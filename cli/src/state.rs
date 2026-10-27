//! Loom definitions and `.sixloom/state.json`.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;

pub const STATE: &str = ".sixloom/state.json";

pub struct Loom {
    pub id: &'static str,
    pub title: &'static str,
    pub skill: &'static str,
    pub reviewer: &'static str,
    /// Looms that must be approved before this one starts.
    pub deps: &'static [&'static str],
    /// The context pack: the only files this loom reads. Changes here make it stale.
    pub inputs: &'static [&'static str],
    /// What the loom produces and commits.
    pub outputs: &'static [&'static str],
}

/// Product track first (spec → design → preview), then the engineering track
/// derives data, system and API from the approved preview.
///
/// Paths are tokens resolved from sixloom.toml, so the layout follows the project's stack:
/// `{content}` docs.content_dir · `{ui}` every interface folder · `{ui:api}` / `{ui:mocks}` /
/// `{ui:preview}` each interface's API client, mocks and preview guide · `{svc}` every service
/// folder · `{rules}` the agents' instruction files.
pub const LOOMS: &[Loom] = &[
    Loom {
        id: "spec",
        title: "Spec",
        skill: "sxl-specify",
        reviewer: "product",
        deps: &[],
        inputs: &[],
        outputs: &["spec/SPEC.md"],
    },
    Loom {
        id: "design",
        title: "UI design",
        skill: "sxl-design",
        reviewer: "product",
        deps: &["spec"],
        inputs: &["spec/SPEC.md"],
        outputs: &["design/"],
    },
    Loom {
        id: "preview",
        title: "Preview",
        skill: "sxl-preview",
        reviewer: "product",
        deps: &["design"],
        inputs: &["spec/SPEC.md", "design/"],
        outputs: &["{ui}"],
    },
    Loom {
        id: "data",
        title: "Data model",
        skill: "sxl-data-model",
        reviewer: "engineer",
        deps: &["preview"],
        inputs: &[
            "spec/SPEC.md",
            "design/SCREENS.md",
            "{ui:api}",
            "{ui:mocks}",
        ],
        outputs: &["{content}/architecture/data-model.mdx"],
    },
    Loom {
        id: "system",
        title: "System design",
        skill: "sxl-system-design",
        reviewer: "engineer",
        deps: &["data"],
        inputs: &["spec/SPEC.md", "{content}/architecture/data-model.mdx"],
        // The loom records the pattern and services in sixloom.toml, which rewrites the rules.
        outputs: &[
            "{content}/architecture/system-design.mdx",
            "docs/diagrams/",
            "sixloom.toml",
            "{rules}",
        ],
    },
    Loom {
        id: "api",
        title: "API & docs",
        skill: "sxl-api-docs",
        reviewer: "engineer",
        deps: &["system"],
        inputs: &[
            "spec/SPEC.md",
            "design/SCREENS.md",
            "{content}/architecture/data-model.mdx",
            "{content}/architecture/system-design.mdx",
            "{ui:api}",
            "{ui:mocks}",
        ],
        outputs: &["contracts/openapi.yaml", "docs/"],
    },
    Loom {
        id: "tests",
        title: "Test suites",
        skill: "sxl-test-suites",
        reviewer: "engineer",
        deps: &["api"],
        inputs: &[
            "spec/SPEC.md",
            "design/SCREENS.md",
            "contracts/openapi.yaml",
            "{ui:preview}",
        ],
        outputs: &[
            "tests/",
            "{svc}",
            ".github/workflows/",
            "justfile",
            ".sixloom/tests.lock",
        ],
    },
    Loom {
        id: "build",
        title: "Build",
        skill: "sxl-build",
        reviewer: "agent",
        deps: &["tests"],
        inputs: &[
            "spec/SPEC.md",
            "contracts/openapi.yaml",
            ".sixloom/tests.lock",
        ],
        outputs: &["{ui}", "{svc}", ".sixloom/runs/"],
    },
    Loom {
        id: "validate",
        title: "Validate",
        skill: "sxl-validate",
        reviewer: "product",
        deps: &["build"],
        inputs: &[
            "spec/SPEC.md",
            "contracts/openapi.yaml",
            ".sixloom/tests.lock",
            "{ui:preview}",
        ],
        outputs: &[".sixloom/runs/"],
    },
];

pub fn loom(id: &str) -> Result<&'static Loom> {
    LOOMS.iter().find(|l| l.id == id).ok_or_else(|| {
        let ids: Vec<_> = LOOMS.iter().map(|l| l.id).collect();
        anyhow!("unknown loom '{id}', expected one of: {}", ids.join(", "))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Pending,
    InProgress,
    Review,
    Approved,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoomState {
    #[serde(default)]
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    /// Hash of each input path when the loom was approved; drives stale detection.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub inputs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    #[serde(default)]
    pub looms: BTreeMap<String, LoomState>,
}

impl Default for State {
    fn default() -> Self {
        State {
            version: 1,
            looms: BTreeMap::new(),
        }
    }
}

/// What the dashboard and `status` show: stored status plus derived conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Blocked,
    Ready,
    InProgress,
    Review,
    Approved,
    Stale,
}

impl View {
    pub fn label(self) -> &'static str {
        match self {
            View::Blocked => "blocked",
            View::Ready => "ready",
            View::InProgress => "in progress",
            View::Review => "in review",
            View::Approved => "approved",
            View::Stale => "stale",
        }
    }
    pub fn glyph(self) -> &'static str {
        match self {
            View::Blocked => "·",
            View::Ready => "○",
            View::InProgress => "◐",
            View::Review => "◑",
            View::Approved => "●",
            View::Stale => "!",
        }
    }
}

impl State {
    pub fn load(root: &Path) -> Result<State> {
        let p = root.join(STATE);
        if !p.exists() {
            return Ok(State::default());
        }
        let text = fs::read_to_string(&p)?;
        serde_json::from_str(&text).with_context(|| format!("parsing {}", p.display()))
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let p = root.join(STATE);
        fs::create_dir_all(p.parent().unwrap())?;
        fs::write(p, serde_json::to_string_pretty(self)? + "\n")?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> LoomState {
        self.looms.get(id).cloned().unwrap_or_default()
    }

    pub fn entry(&mut self, id: &str) -> &mut LoomState {
        self.looms.entry(id.to_string()).or_default()
    }

    pub fn deps_approved(&self, l: &Loom) -> Vec<&'static str> {
        l.deps
            .iter()
            .copied()
            .filter(|d| self.get(d).status != Status::Approved)
            .collect()
    }

    pub fn view(&self, root: &Path, cfg: &Config, l: &Loom) -> View {
        let s = self.get(l.id);
        match s.status {
            Status::Approved if !changed_inputs(root, cfg, l, self).is_empty() => View::Stale,
            Status::Approved => View::Approved,
            Status::Review => View::Review,
            Status::InProgress => View::InProgress,
            Status::Pending if self.deps_approved(l).is_empty() => View::Ready,
            Status::Pending => View::Blocked,
        }
    }
}

/// Resolve a loom path token into project paths (zero or more) using sixloom.toml.
pub fn expand(p: &str, cfg: &Config) -> Vec<String> {
    let ui = &cfg.interfaces;
    let dir = |s: &str| format!("{}/", s.trim_end_matches('/'));
    match p {
        "{ui}" => ui.iter().map(|i| dir(&i.path)).collect(),
        "{ui:api}" => ui.iter().map(|i| i.api_client_path()).collect(),
        "{ui:mocks}" => ui.iter().map(|i| i.mocks_path()).collect(),
        "{ui:preview}" => ui.iter().map(|i| i.preview_doc()).collect(),
        "{svc}" => cfg.services.iter().map(|s| dir(&s.path)).collect(),
        "{rules}" => crate::harness::targets(&cfg.agents.harnesses)
            .map(|(_, files)| files.into_iter().map(String::from).collect())
            .unwrap_or_default(),
        _ => vec![p.replace("{content}", &cfg.docs.content_dir)],
    }
}

pub fn expand_all(paths: &[&str], cfg: &Config) -> Vec<String> {
    paths.iter().flat_map(|p| expand(p, cfg)).collect()
}

const IGNORED: &[&str] = &[
    ".git",
    "node_modules",
    ".next",
    "dist",
    "build",
    ".astro",
    ".docusaurus",
    ".turbo",
    ".preview-shots",
    "test-results",
    "playwright-report",
    "target",
    ".source",
    ".cache",
];

/// Files under a project path (file or directory), sorted, skipping build output.
pub fn files_under(root: &Path, rel: &str) -> Vec<String> {
    let base = root.join(rel.trim_end_matches('/'));
    if base.is_file() {
        return vec![rel.trim_end_matches('/').to_string()];
    }
    if !base.is_dir() {
        return vec![];
    }
    let mut out: Vec<String> = WalkDir::new(&base)
        .into_iter()
        .filter_entry(|e| !IGNORED.contains(&e.file_name().to_string_lossy().as_ref()))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            e.path()
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
        })
        .collect();
    out.sort();
    out
}

pub fn hash_paths(root: &Path, paths: &[String]) -> String {
    let mut h = Sha256::new();
    for rel in paths {
        for f in files_under(root, rel) {
            h.update(f.as_bytes());
            h.update([0]);
            if let Ok(bytes) = fs::read(root.join(&f)) {
                h.update(Sha256::digest(&bytes));
            }
        }
    }
    hex(&h.finalize())
}

pub const SPEC: &str = "spec/SPEC.md";

/// Hash of each input path. The spec is hashed per feature section
/// (`spec/SPEC.md#F-001`, plus `#intro` for everything before the first feature),
/// so stale looms can say which feature changed.
pub fn input_hashes(root: &Path, cfg: &Config, l: &Loom) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for p in expand_all(l.inputs, cfg) {
        if p == SPEC {
            if let Ok(text) = fs::read_to_string(root.join(SPEC)) {
                out.extend(spec_sections(&text));
                continue;
            }
        }
        let h = hash_paths(root, std::slice::from_ref(&p));
        out.insert(p, h);
    }
    out
}

fn spec_sections(text: &str) -> Vec<(String, String)> {
    let head = regex::Regex::new(r"^#{2,4}\s+(F-\d{3,})\b").unwrap();
    let mut sections: Vec<(String, String)> = vec![("intro".into(), String::new())];
    for line in text.lines() {
        if let Some(c) = head.captures(line) {
            sections.push((c[1].to_string(), String::new()));
        } else if line.starts_with("## ") && sections.len() > 1 {
            // A level-2 heading after the features (Key flows, Rules, …) belongs to the tail.
            sections.push(("rest".into(), String::new()));
        }
        let cur = sections.last_mut().unwrap();
        cur.1.push_str(line.trim_end());
        cur.1.push('\n');
    }
    let mut merged: BTreeMap<String, String> = BTreeMap::new();
    for (k, body) in sections {
        merged.entry(k).or_default().push_str(&body);
    }
    merged
        .into_iter()
        .map(|(k, body)| (format!("{SPEC}#{k}"), hex(&Sha256::digest(body.as_bytes()))))
        .collect()
}

/// Inputs whose content changed since the loom was approved.
pub fn changed_inputs(root: &Path, cfg: &Config, l: &Loom, st: &State) -> Vec<String> {
    let recorded = st.get(l.id).inputs;
    if recorded.is_empty() {
        return vec![];
    }
    // Keys are compared without a trailing slash, so `src/api` and `src/api/` are one input.
    let norm = |m: BTreeMap<String, String>| -> BTreeMap<String, String> {
        m.into_iter()
            .map(|(k, v)| (k.trim_end_matches('/').to_string(), v))
            .collect()
    };
    let recorded = norm(recorded);
    let now = norm(input_hashes(root, cfg, l));
    let mut changed: Vec<String> = now
        .iter()
        .filter(|(p, h)| recorded.get(*p) != Some(*h))
        .map(|(p, _)| p.clone())
        .collect();
    // A recorded section that disappeared (feature removed) is a change too.
    changed.extend(recorded.keys().filter(|k| !now.contains_key(*k)).cloned());
    changed.sort();
    changed.dedup();
    changed
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
