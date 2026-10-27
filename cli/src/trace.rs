//! Spec parsing and the feature → screen → operation → test trace matrix.
//!
//! IDs never change product behavior. They live only in:
//! - documents: an `sxl:` marker, e.g. `<sub>sxl: F-003.AC1 · F-003.AC2</sub>` (visible, so docs search finds it)
//! - OpenAPI operation `description`s: a final `sxl: F-003.AC1` line (rendered by the API reference)
//! - the front end: `data-sxl="F-003.AC1"` attributes (inspect in devtools; stripped in production builds)
//! - test titles: `test('F-003.AC1 …')`

use anyhow::{Context, Result};
use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::state::files_under;

#[derive(Debug, Clone, Serialize)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub priority: String,
    pub status: String,
    pub acs: Vec<Ac>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Ac {
    pub id: String,
    pub text: String,
    pub removed: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct Spec {
    pub features: Vec<Feature>,
    pub problems: Vec<String>,
}

pub fn parse_spec(text: &str) -> Spec {
    let feat = Regex::new(r"^#{2,4}\s+(F-\d{3,})\s+(.+?)\s*$").unwrap();
    let ac = Regex::new(r"\*\*(F-\d{3,}\.AC\d+)\*\*\s*(.*)$").unwrap();
    let field = Regex::new(r"\*\*(Priority|Status)\*\*:\s*([A-Za-z-]+)").unwrap();
    let mut spec = Spec::default();
    let mut seen = BTreeSet::new();
    for line in text.lines() {
        if let Some(c) = feat.captures(line) {
            let id = c[1].to_string();
            if !seen.insert(id.clone()) {
                spec.problems.push(format!("duplicate feature ID {id}"));
            }
            spec.features.push(Feature {
                id,
                name: c[2].to_string(),
                priority: "must".into(),
                status: "planned".into(),
                acs: vec![],
            });
            continue;
        }
        let Some(f) = spec.features.last_mut() else {
            continue;
        };
        if let Some(c) = field.captures(line) {
            let v = c[2].to_lowercase();
            if &c[1] == "Priority" {
                f.priority = v
            } else {
                f.status = v
            }
        }
        if let Some(c) = ac.captures(line) {
            let id = c[1].to_string();
            if !id.starts_with(&format!("{}.", f.id)) {
                spec.problems.push(format!("{id} is listed under {}", f.id));
            }
            if !seen.insert(id.clone()) {
                spec.problems.push(format!("duplicate AC ID {id}"));
            }
            let removed = line
                .trim_start()
                .trim_start_matches(['-', '*', ' '])
                .starts_with("~~");
            let text = c[2].trim().trim_end_matches("~~").trim().to_string();
            if text.is_empty() && !removed {
                spec.problems.push(format!("{id} has no text"));
            }
            f.acs.push(Ac { id, text, removed });
        }
    }
    for f in &spec.features {
        if f.status != "dropped" && !f.acs.iter().any(|a| !a.removed) {
            spec.problems
                .push(format!("{} has no acceptance criteria", f.id));
        }
        if !["must", "should", "could"].contains(&f.priority.as_str()) {
            spec.problems
                .push(format!("{} has unknown priority '{}'", f.id, f.priority));
        }
    }
    let placeholder = Regex::new(r"<[A-Z][a-z][^<>\n]{0,40}>|Given …|when …").unwrap();
    let unfilled: Vec<_> = text
        .lines()
        .filter(|l| !l.trim_start().starts_with("<!--"))
        .filter_map(|l| placeholder.find(l).map(|m| m.as_str().to_string()))
        .collect();
    if !unfilled.is_empty() {
        spec.problems.push(format!(
            "unfilled template placeholders: {}",
            unfilled.join(", ")
        ));
    }
    if spec.features.is_empty() {
        spec.problems
            .push("no features found (expected headings like `### F-001 Name`)".into());
    }
    spec
}

pub fn load_spec(root: &Path) -> Result<Spec> {
    let p = root.join("spec/SPEC.md");
    let text = fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
    Ok(parse_spec(&text))
}

#[derive(Debug, Serialize)]
pub struct Row {
    pub ac: String,
    pub feature: String,
    pub priority: String,
    pub screens: Vec<String>,
    /// Front-end files with a matching `data-sxl` attribute.
    pub ui: Vec<String>,
    pub operations: Vec<String>,
    /// Doc pages (data model, system design, …) with a matching `sxl:` marker.
    pub docs: Vec<String>,
    pub tests: Vec<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct Trace {
    pub rows: Vec<Row>,
    /// IDs cited by artifacts that the spec does not define.
    pub orphans: Vec<String>,
    /// Operations in the contract with no `sxl:` marker.
    pub untraced_operations: Vec<String>,
    pub gaps: Vec<String>,
}

/// Source files an interface may carry trace markers in.
const UI_EXTS: &[&str] = &[
    ".ts", ".tsx", ".js", ".jsx", ".mjs", ".vue", ".svelte", ".astro", ".html", ".rs", ".go",
    ".py", ".dart", ".kt", ".swift",
];

pub fn build(root: &Path, cfg: &Config, filter: Option<&str>) -> Result<Trace> {
    let content_dir = cfg.docs.content_dir.as_str();
    let spec = load_spec(root)?;
    let ac_re = Regex::new(r"F-\d{3,}\.AC\d+").unwrap();
    let screens = screen_refs(root, &ac_re);
    let (ops, untraced) = operation_refs(root);
    let tests = test_refs(root, &cfg.tests.locked);
    let docs = marker_refs(root, content_dir, &[".md", ".mdx"], &marker_re());
    // Web interfaces mark elements with data-sxl; TUI, CLI and native code use an `sxl:` comment.
    let mut ui: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for i in &cfg.interfaces {
        for re in [attr_re(), marker_re()] {
            for (id, files) in marker_refs(root, &i.path, UI_EXTS, &re) {
                let v = ui.entry(id).or_default();
                v.extend(files);
                v.sort();
                v.dedup();
            }
        }
    }

    let known: BTreeSet<String> = spec
        .features
        .iter()
        .flat_map(|f| f.acs.iter().map(|a| a.id.clone()))
        .collect();
    let mut trace = Trace {
        untraced_operations: untraced,
        ..Default::default()
    };
    let features: BTreeSet<String> = spec.features.iter().map(|f| f.id.clone()).collect();
    for id in screens
        .keys()
        .chain(ops.keys())
        .chain(tests.keys())
        .chain(docs.keys())
        .chain(ui.keys())
    {
        let defined = known.contains(id) || features.contains(id);
        if !defined && !trace.orphans.contains(id) {
            trace.orphans.push(id.clone());
        }
    }
    for f in &spec.features {
        if f.status == "dropped" {
            continue;
        }
        for a in f.acs.iter().filter(|a| !a.removed) {
            if filter.is_some_and(|q| !a.id.starts_with(q)) {
                continue;
            }
            let row = Row {
                ac: a.id.clone(),
                feature: f.id.clone(),
                priority: f.priority.clone(),
                screens: screens.get(&a.id).cloned().unwrap_or_default(),
                ui: with_feature(&ui, &a.id, &f.id),
                operations: ops.get(&a.id).cloned().unwrap_or_default(),
                docs: with_feature(&docs, &a.id, &f.id),
                tests: tests.get(&a.id).cloned().unwrap_or_default(),
            };
            if row.tests.is_empty() && f.priority == "must" && root.join("tests").is_dir() {
                trace.gaps.push(format!("{} (must) has no test", a.id));
            }
            if row.screens.is_empty() && root.join("design/SCREENS.md").is_file() {
                trace.gaps.push(format!("{} is not on any screen", a.id));
            }
            trace.rows.push(row);
        }
    }
    for o in &trace.orphans {
        trace
            .gaps
            .push(format!("{o} is cited but not defined in SPEC.md"));
    }
    for o in &trace.untraced_operations {
        trace.gaps.push(format!(
            "operation {o} has no `sxl:` marker in its description"
        ));
    }
    Ok(trace)
}

/// AC → screen IDs, from `## S-xx` sections of design/SCREENS.md.
fn screen_refs(root: &Path, ac_re: &Regex) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let Ok(text) = fs::read_to_string(root.join("design/SCREENS.md")) else {
        return out;
    };
    let head = Regex::new(r"^#{2,3}\s+(S-\d+)").unwrap();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if let Some(c) = head.captures(line) {
            current = Some(c[1].to_string());
            continue;
        }
        let Some(s) = &current else { continue };
        for m in ac_re.find_iter(line) {
            let v = out.entry(m.as_str().to_string()).or_default();
            if !v.contains(s) {
                v.push(s.clone());
            }
        }
    }
    out
}

/// AC → operationIds via the `sxl:` marker in each operation's description.
fn operation_refs(root: &Path) -> (BTreeMap<String, Vec<String>>, Vec<String>) {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut untraced = vec![];
    let Ok(text) = fs::read_to_string(root.join("contracts/openapi.yaml")) else {
        return (out, untraced);
    };
    let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&text) else {
        return (out, untraced);
    };
    let Some(paths) = doc.get("paths").and_then(|p| p.as_mapping()) else {
        return (out, untraced);
    };
    for (path, item) in paths {
        let Some(item) = item.as_mapping() else {
            continue;
        };
        for (method, op) in item {
            let m = method.as_str().unwrap_or_default();
            if !["get", "post", "put", "patch", "delete", "head", "options"].contains(&m) {
                continue;
            }
            let name = op
                .get("operationId")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| {
                    format!("{} {}", m.to_uppercase(), path.as_str().unwrap_or("?"))
                });
            let desc = op
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let acs = ids_in_markers(desc, &marker_re());
            if acs.is_empty() {
                untraced.push(name.clone());
            }
            for ac in acs {
                out.entry(ac).or_default().push(name.clone());
            }
        }
    }
    (out, untraced)
}

/// AC → test files, from AC IDs in test titles under tests/.
/// AC → test files, from AC IDs in test titles: `test('F-001.AC1 …')` in JS/TS, or
/// `f001_ac1_…` / `TestF001AC1…` names in Rust, Python and Go.
fn test_refs(root: &Path, dirs: &[String]) -> BTreeMap<String, Vec<String>> {
    let id = Regex::new(r"(?i)F[-_]?(\d{3,})[._-]?AC[-_]?(\d+)").unwrap();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut scan: Vec<&str> = vec!["tests"];
    scan.extend(dirs.iter().map(String::as_str));
    let mut files: Vec<String> = scan.iter().flat_map(|d| files_under(root, d)).collect();
    files.sort();
    files.dedup();
    for f in files {
        if !is_test_source(&f) {
            continue;
        }
        let Ok(text) = fs::read_to_string(root.join(&f)) else {
            continue;
        };
        for t in crate::lock::parse_file(&f, &text) {
            for c in id.captures_iter(&t.title) {
                let ac = format!("F-{}.AC{}", &c[1], &c[2]);
                let v = out.entry(ac).or_default();
                if !v.contains(&f) {
                    v.push(f.clone());
                }
            }
        }
    }
    out
}

fn marker_re() -> Regex {
    Regex::new(r"sxl:\s*([A-Z0-9.,·/ \t-]+)").unwrap()
}

fn attr_re() -> Regex {
    Regex::new(r#"data-sxl\s*=\s*["'{]([^"'}]+)"#).unwrap()
}

/// Feature or AC IDs inside every capture of `re`.
fn ids_in_markers(text: &str, re: &Regex) -> Vec<String> {
    let id = Regex::new(r"F-\d{3,}(?:\.AC\d+)?").unwrap();
    let mut out = vec![];
    for c in re.captures_iter(text) {
        for m in id.find_iter(&c[1]) {
            if !out.iter().any(|x| x == m.as_str()) {
                out.push(m.as_str().to_string());
            }
        }
    }
    out
}

/// ID → files under `dir` whose text carries the ID in a marker.
fn marker_refs(root: &Path, dir: &str, exts: &[&str], re: &Regex) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for f in files_under(root, dir) {
        if !exts.iter().any(|e| f.ends_with(e)) {
            continue;
        }
        let Ok(text) = fs::read_to_string(root.join(&f)) else {
            continue;
        };
        if is_generated(&f, &text) {
            continue;
        }
        for id in ids_in_markers(&text, re) {
            out.entry(id).or_default().push(f.clone());
        }
    }
    out
}

/// Generated code (API types, clients) copies the contract's `sxl:` lines; it is not evidence.
fn is_generated(path: &str, text: &str) -> bool {
    path.ends_with(".d.ts")
        || text.lines().take(8).any(|l| {
            let l = l.to_ascii_lowercase();
            l.contains("auto-generated") || l.contains("@generated") || l.contains("code generated")
        })
}

/// Files citing the AC itself or its whole feature.
fn with_feature(map: &BTreeMap<String, Vec<String>>, ac: &str, feature: &str) -> Vec<String> {
    let mut v: Vec<String> = map
        .get(ac)
        .into_iter()
        .chain(map.get(feature))
        .flatten()
        .cloned()
        .collect();
    v.sort();
    v.dedup();
    v
}

pub fn is_test_source(f: &str) -> bool {
    [".ts", ".tsx", ".js", ".mjs", ".cjs", ".py", ".rs", ".go"]
        .iter()
        .any(|e| f.ends_with(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "\
### F-001 Sign in
- **Priority**: must
**Acceptance criteria**
- **F-001.AC1** Given a user, when they sign in, then they see projects.
- ~~**F-001.AC2** Old rule.~~
### F-002 Export
- **Priority**: could
- **Status**: dropped
### F-003 Empty
- **Priority**: should
";

    #[test]
    fn parses_features_and_problems() {
        let s = parse_spec(SPEC);
        assert_eq!(s.features.len(), 3);
        assert_eq!(s.features[0].acs.len(), 2);
        assert!(s.features[0].acs[1].removed);
        assert_eq!(s.features[1].status, "dropped");
        assert_eq!(
            s.problems,
            vec!["F-003 has no acceptance criteria".to_string()]
        );
    }

    #[test]
    fn unfilled_template_is_rejected() {
        let tpl = crate::catalog::pack_file("core/skills/sxl-specify/SPEC.template.md").unwrap();
        let s = parse_spec(tpl);
        assert!(s
            .problems
            .iter()
            .any(|p| p.starts_with("unfilled template placeholders")));
        assert_eq!(s.features.len(), 2);
    }

    #[test]
    fn traces_across_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();
        fs::create_dir_all(r.join("spec")).unwrap();
        fs::create_dir_all(r.join("design")).unwrap();
        fs::create_dir_all(r.join("contracts")).unwrap();
        fs::create_dir_all(r.join("tests/e2e")).unwrap();
        fs::write(r.join("spec/SPEC.md"), SPEC).unwrap();
        fs::write(
            r.join("design/SCREENS.md"),
            "## S-01 Sign in\n| btn | F-001.AC1 |\n",
        )
        .unwrap();
        fs::write(
            r.join("contracts/openapi.yaml"),
            "paths:\n  /session:\n    post:\n      operationId: signIn\n      description: |\n        Start a session.\n\n        sxl: F-001.AC1\n  /x:\n    get:\n      operationId: orphanOp\n",
        )
        .unwrap();
        fs::write(
            r.join("tests/e2e/a.spec.ts"),
            "test('F-001.AC1 signs in', async () => {});\ntest('F-009.AC1 ghost', () => {})",
        )
        .unwrap();
        fs::create_dir_all(r.join("docs/content/docs/architecture")).unwrap();
        fs::write(
            r.join("docs/content/docs/architecture/data-model.mdx"),
            "## User\n<sub>sxl: F-001</sub>\n",
        )
        .unwrap();
        fs::create_dir_all(r.join("apps/web/src")).unwrap();
        fs::write(
            r.join("apps/web/src/SignIn.tsx"),
            "<form data-sxl=\"F-001.AC1\">",
        )
        .unwrap();
        let mut cfg = crate::init::default_config("t");
        cfg.interfaces.push(crate::config::Interface::new(
            "tui",
            "tui",
            "ratatui",
            Some("crates/tui"),
        ));
        fs::create_dir_all(r.join("crates/tui/src")).unwrap();
        fs::write(
            r.join("crates/tui/src/list.rs"),
            "// sxl: F-001.AC1\nfn draw() {}\n",
        )
        .unwrap();
        // A Rust acceptance test names its AC in the function name.
        fs::create_dir_all(r.join("tests/e2e/tui")).unwrap();
        fs::write(
            r.join("tests/e2e/tui/list.rs"),
            "#[test]\nfn f001_ac1_lists() {\n    assert!(true);\n}\n",
        )
        .unwrap();
        let t = build(r, &cfg, None).unwrap();
        assert_eq!(t.rows.len(), 1);
        assert_eq!(t.rows[0].screens, vec!["S-01"]);
        assert_eq!(t.rows[0].operations, vec!["signIn"]);
        assert_eq!(
            t.rows[0].docs,
            vec!["docs/content/docs/architecture/data-model.mdx"]
        );
        assert_eq!(
            t.rows[0].ui,
            vec!["apps/web/src/SignIn.tsx", "crates/tui/src/list.rs"]
        );
        assert_eq!(
            t.rows[0].tests,
            vec!["tests/e2e/a.spec.ts", "tests/e2e/tui/list.rs"]
        );
        assert_eq!(t.orphans, vec!["F-009.AC1"]);
        assert_eq!(t.untraced_operations, vec!["orphanOp"]);
    }
}
