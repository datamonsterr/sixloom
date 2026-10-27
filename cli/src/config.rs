//! `sixloom.toml`: the project's framework choices.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const FILE: &str = "sixloom.toml";

pub const FRONTENDS: &[(&str, &str)] = &[
    ("react-vite", "React + Vite (SPA)"),
    ("nextjs", "Next.js (React, app router)"),
    ("vue", "Vue 3 + Vite"),
    ("nuxt", "Nuxt"),
    ("sveltekit", "SvelteKit"),
];
pub const UI_KITS: &[(&str, &str)] = &[
    ("shadcn", "shadcn/ui (React only)"),
    ("none", "Plain CSS / Tailwind from DESIGN.md tokens"),
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
    pub frontend: Frontend,
    pub docs: Docs,
    pub design: Design,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub validate: Validate,
    #[serde(default)]
    pub skills: Skills,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agents {
    pub harnesses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontend {
    pub framework: String,
    pub ui_kit: String,
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
        check_choice("frontend", &self.frontend.framework, FRONTENDS)?;
        check_choice("ui kit", &self.frontend.ui_kit, UI_KITS)?;
        check_choice("docs engine", &self.docs.engine, DOCS)?;
        check_choice("design tool", &self.design.tool, DESIGNS)?;
        for h in &self.agents.harnesses {
            crate::harness::get(h)?;
        }
        Ok(())
    }

    pub fn choices(&self) -> crate::catalog::Choices<'_> {
        crate::catalog::Choices {
            frontend: &self.frontend.framework,
            ui_kit: &self.frontend.ui_kit,
            docs: &self.docs.engine,
            design: &self.design.tool,
        }
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
