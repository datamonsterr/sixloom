//! `sixloom.toml`: the project's framework choices.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const FILE: &str = "sixloom.toml";

/// Interface kinds and the frameworks sixloom knows for each (`kind`, `framework`, label).
/// An interface is anything a user touches: a web app, a TUI, a CLI, a mobile or desktop app.
pub const INTERFACES: &[(&str, &str, &str)] = &[
    ("web", "react-vite", "React + Vite (SPA)"),
    ("web", "nextjs", "Next.js (React, app router)"),
    ("web", "vue", "Vue 3 + Vite"),
    ("web", "nuxt", "Nuxt"),
    ("web", "sveltekit", "SvelteKit"),
    ("tui", "ratatui", "Ratatui (Rust)"),
    ("tui", "bubbletea", "Bubble Tea (Go)"),
    ("tui", "textual", "Textual (Python)"),
    ("tui", "ink", "Ink (Node, React for terminals)"),
    ("cli", "clap", "clap (Rust)"),
    ("cli", "cobra", "Cobra (Go)"),
    ("cli", "typer", "Typer (Python)"),
    ("cli", "commander", "Commander (Node)"),
    ("mobile", "expo", "Expo (React Native)"),
    ("mobile", "flutter", "Flutter"),
    ("desktop", "tauri", "Tauri"),
    ("desktop", "electron", "Electron"),
];
pub const KINDS: &[&str] = &["web", "tui", "cli", "mobile", "desktop"];
/// Architecture patterns the system-design loom chooses from.
pub const PATTERNS: &[(&str, &str)] = &[
    (
        "undecided",
        "Not chosen yet: the system-design loom decides",
    ),
    ("monolith", "One deployable backend"),
    (
        "modular-monolith",
        "One deployable, strict internal modules",
    ),
    ("microservices", "Several independently deployed services"),
    ("serverless", "Functions behind a gateway"),
    ("local-only", "No backend: the interface owns its data"),
];
pub const SERVICE_KINDS: &[&str] = &[
    "http-api", "worker", "gateway", "library", "database", "other",
];
pub const UI_KITS: &[(&str, &str)] = &[
    ("shadcn", "shadcn/ui (React web only)"),
    ("none", "Plain styling from DESIGN.md tokens"),
];
pub const DOCS: &[(&str, &str)] = &[
    ("fumadocs", "Fumadocs (Next.js, GitBook-like)"),
    ("docusaurus", "Docusaurus (most widely used)"),
    ("starlight", "Starlight (Astro, lightest)"),
];
pub const DESIGNS: &[(&str, &str)] = &[
    ("claude", "Claude Design / HTML comps"),
    ("stitch", "Google Stitch (needs Stitch MCP)"),
    ("none", "No comps: inventory + preview only"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub project: Project,
    pub agents: Agents,
    /// Every front end the product has. Chosen at `init`; edit with `sixloom layout interface …`.
    #[serde(default)]
    pub interfaces: Vec<Interface>,
    /// Architecture pattern. Decided by the system-design loom.
    #[serde(default)]
    pub system: System,
    /// Backend services and their folders. Recorded by the system-design loom.
    #[serde(default)]
    pub services: Vec<Service>,
    pub docs: Docs,
    pub design: Design,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub validate: Validate,
    #[serde(default)]
    pub skills: Skills,
    #[serde(default)]
    pub tests: Tests,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tests {
    /// Folders whose tests are locked acceptance tests (any language sixloom parses:
    /// JS/TS, Rust, Python, Go). A Rust TUI might add `crates/tui/tests/acceptance`.
    pub locked: Vec<String>,
}

impl Default for Tests {
    fn default() -> Self {
        Tests {
            locked: crate::lock::LOCKED_DIRS
                .iter()
                .map(|d| d.to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agents {
    pub harnesses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Interface {
    /// Short name used in commands and docs, e.g. `web`, `tui`, `admin`.
    pub id: String,
    /// web | tui | cli | mobile | desktop
    pub kind: String,
    pub framework: String,
    #[serde(default = "none")]
    pub ui_kit: String,
    /// Folder of this interface, relative to the project root.
    pub path: String,
    /// The only module that talks to the backend (relative to `path`).
    #[serde(default = "default_api_client")]
    pub api_client: String,
    /// Mock data and scenarios used by the preview (relative to `path`).
    #[serde(default = "default_mocks")]
    pub mocks: String,
}

fn none() -> String {
    "none".into()
}
fn default_api_client() -> String {
    "src/api".into()
}
fn default_mocks() -> String {
    "src/mocks".into()
}

impl Interface {
    pub fn new(id: &str, kind: &str, framework: &str, path: Option<&str>) -> Interface {
        Interface {
            id: id.into(),
            kind: kind.into(),
            framework: framework.into(),
            ui_kit: if framework == "react-vite" || framework == "nextjs" {
                "shadcn".into()
            } else {
                none()
            },
            path: path
                .map(String::from)
                .unwrap_or_else(|| format!("apps/{id}")),
            api_client: default_api_client(),
            mocks: default_mocks(),
        }
    }
    fn join(&self, rel: &str) -> String {
        format!(
            "{}/{}",
            self.path.trim_end_matches('/'),
            rel.trim_start_matches("./")
        )
    }
    pub fn api_client_path(&self) -> String {
        self.join(&self.api_client)
    }
    pub fn mocks_path(&self) -> String {
        self.join(&self.mocks)
    }
    /// The reviewer's guide: every screen × state and how to reach it.
    pub fn preview_doc(&self) -> String {
        self.join("PREVIEW.md")
    }
    pub fn label(&self) -> String {
        format!("{} ({} · {})", self.id, self.kind, self.framework)
    }
}

/// Parse `kind:framework[:path]`, or just a known framework name.
pub fn parse_interface(spec: &str, taken: &[Interface]) -> Result<Interface> {
    let parts: Vec<&str> = spec.splitn(3, ':').collect();
    let (kind, framework, path) = match parts.as_slice() {
        [fw] => {
            let Some((k, _, _)) = INTERFACES.iter().find(|(_, f, _)| f == fw) else {
                bail!("unknown framework '{fw}'; use kind:framework (see `sixloom layout --help`)")
            };
            (*k, *fw, None)
        }
        [k, fw] => (*k, *fw, None),
        [k, fw, p] => (*k, *fw, Some(*p)),
        _ => unreachable!(),
    };
    if !KINDS.contains(&kind) {
        bail!(
            "unknown interface kind '{kind}', expected one of: {}",
            KINDS.join(", ")
        );
    }
    let mut id = kind.to_string();
    let mut n = 2;
    while taken.iter().any(|i| i.id == id) {
        id = format!("{kind}{n}");
        n += 1;
    }
    Ok(Interface::new(&id, kind, framework, path))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    pub pattern: String,
}

impl Default for System {
    fn default() -> Self {
        System {
            pattern: "undecided".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    pub id: String,
    /// http-api | worker | gateway | library | database | other
    pub kind: String,
    /// Language/framework, free text chosen in the ADRs, e.g. "hono-node", "axum", "fastapi".
    pub stack: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Docs {
    pub engine: String,
    pub content_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Design {
    pub tool: String,
    pub fidelity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    pub max_parallel: u32,
    pub max_attempts_per_slice: u32,
    pub pr: bool,
}

impl Default for Build {
    fn default() -> Self {
        Build {
            max_parallel: 3,
            max_attempts_per_slice: 3,
            pr: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validate {
    pub visual_tolerance: f64,
}

impl Default for Validate {
    fn default() -> Self {
        Validate {
            visual_tolerance: 0.005,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Skills {
    /// Install the non-selected packs into an other-skill bank.
    #[serde(default)]
    pub bank: bool,
    /// Extra skill names (from the catalog) to install natively.
    #[serde(default)]
    pub extra: Vec<String>,
}

pub fn content_dir_for(engine: &str) -> &'static str {
    match engine {
        "docusaurus" => "docs/docs",
        "starlight" => "docs/src/content/docs",
        _ => "docs/content/docs",
    }
}

pub fn check_choice(kind: &str, value: &str, options: &[(&str, &str)]) -> Result<()> {
    if options.iter().any(|(id, _)| *id == value) {
        return Ok(());
    }
    let ids: Vec<_> = options.iter().map(|(id, _)| *id).collect();
    bail!(
        "unknown {kind} '{value}', expected one of: {}",
        ids.join(", ")
    )
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        let mut ids = std::collections::BTreeSet::new();
        let mut paths = std::collections::BTreeSet::new();
        for i in &self.interfaces {
            if !KINDS.contains(&i.kind.as_str()) {
                bail!(
                    "interface '{}': unknown kind '{}', expected one of: {}",
                    i.id,
                    i.kind,
                    KINDS.join(", ")
                );
            }
            // Unknown frameworks are allowed (any stack works); known ones get matching skills.
            check_choice("ui kit", &i.ui_kit, UI_KITS)?;
            if !ids.insert(i.id.clone()) {
                bail!("duplicate interface id '{}'", i.id);
            }
            if !paths.insert(i.path.trim_end_matches('/').to_string()) {
                bail!("two components share the folder '{}'", i.path);
            }
        }
        for s in &self.services {
            if !SERVICE_KINDS.contains(&s.kind.as_str()) {
                bail!(
                    "service '{}': unknown kind '{}', expected one of: {}",
                    s.id,
                    s.kind,
                    SERVICE_KINDS.join(", ")
                );
            }
            if !ids.insert(s.id.clone()) {
                bail!("duplicate component id '{}'", s.id);
            }
            if !paths.insert(s.path.trim_end_matches('/').to_string()) {
                bail!("two components share the folder '{}'", s.path);
            }
        }
        check_choice("architecture pattern", &self.system.pattern, PATTERNS)?;
        check_choice("docs engine", &self.docs.engine, DOCS)?;
        check_choice("design tool", &self.design.tool, DESIGNS)?;
        for h in &self.agents.harnesses {
            crate::harness::get(h)?;
        }
        Ok(())
    }

    pub fn choices(&self) -> crate::catalog::Choices<'_> {
        crate::catalog::Choices {
            frameworks: self
                .interfaces
                .iter()
                .map(|i| i.framework.as_str())
                .collect(),
            kinds: self.interfaces.iter().map(|i| i.kind.as_str()).collect(),
            ui_kits: self.interfaces.iter().map(|i| i.ui_kit.as_str()).collect(),
            docs: &self.docs.engine,
            design: &self.design.tool,
        }
    }

    /// One line per component, for status output and the agents rules block.
    pub fn layout_lines(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .interfaces
            .iter()
            .map(|i| format!("`{}/` interface **{}** ({} · {}): API client `{}`, mocks `{}`, preview guide `{}`", i.path.trim_end_matches('/'), i.id, i.kind, i.framework, i.api_client_path(), i.mocks_path(), i.preview_doc()))
            .collect();
        if self.services.is_empty() {
            out.push(match self.system.pattern.as_str() {
                "local-only" => {
                    "no backend services: the interfaces own their data (local-only)".into()
                }
                _ => "services: not decided yet (the system-design loom records them)".into(),
            });
        }
        for s in &self.services {
            out.push(format!(
                "`{}/` service **{}** ({} · {})",
                s.path.trim_end_matches('/'),
                s.id,
                s.kind,
                s.stack
            ));
        }
        out.push(format!(
            "locked acceptance tests: {}",
            self.tests
                .locked
                .iter()
                .map(|d| format!("`{d}/`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        out
    }

    pub fn load(root: &Path) -> Result<Config> {
        let path = root.join(FILE);
        let text = fs::read_to_string(&path).with_context(|| {
            format!("no {FILE} in {}; run `sixloom init` first", root.display())
        })?;
        let cfg: Config =
            toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let body = toml::to_string_pretty(self)?;
        let text = format!(
            "# sixloom project configuration. Docs: https://github.com/datamonsterr/sixloom\n\n{body}"
        );
        fs::write(root.join(FILE), text)?;
        Ok(())
    }
}

/// Walk up from `start` to the directory holding `sixloom.toml`.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|d| d.join(FILE).is_file())
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interfaces_parse_validate_and_roundtrip() {
        let mut cfg = crate::init::default_config("x");
        let tui = parse_interface("tui:ratatui:crates/tui", &cfg.interfaces).unwrap();
        assert_eq!((tui.id.as_str(), tui.path.as_str()), ("tui", "crates/tui"));
        assert_eq!(tui.preview_doc(), "crates/tui/PREVIEW.md");
        cfg.interfaces.push(tui);
        // A second web interface gets a unique id and folder; a bare framework name infers its kind.
        let web2 = parse_interface("nextjs", &cfg.interfaces).unwrap();
        assert_eq!(
            (web2.id.as_str(), web2.kind.as_str(), web2.path.as_str()),
            ("web2", "web", "apps/web2")
        );
        assert!(parse_interface("kiosk:qt", &cfg.interfaces).is_err());
        cfg.services.push(Service {
            id: "api".into(),
            kind: "http-api".into(),
            stack: "axum".into(),
            path: "services/api".into(),
        });
        cfg.validate().unwrap();
        let text = toml::to_string_pretty(&cfg).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back.interfaces, cfg.interfaces);
        assert_eq!(back.services, cfg.services);
        // Two components may not share a folder.
        cfg.services.push(Service {
            id: "worker".into(),
            kind: "worker".into(),
            stack: "axum".into(),
            path: "crates/tui/".into(),
        });
        assert!(cfg
            .validate()
            .unwrap_err()
            .to_string()
            .contains("share the folder"));
    }
}
