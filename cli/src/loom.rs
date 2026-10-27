//! Loom lifecycle: start (with context pack), end (machine checks + commit), approve, next, done.

use anyhow::{bail, Context, Result};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::change;
use crate::config::Config;
use crate::harness;
use crate::lock;
use crate::state::{self, expand, files_under, loom, now, Loom, State, Status, View, LOOMS};
use crate::trace;

pub const CONTEXT_DIR: &str = ".sixloom/context";

pub struct Pack {
    pub path: String,
    pub files: Vec<(String, usize)>,
    pub tokens: usize,
}

/// Rough token estimate (≈4 characters per token), as repomix's fallback does.
pub fn tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// Write the loom's context pack: every input file in one XML document, in the
/// layout popularised by repomix (https://github.com/yamadashy/repomix): a summary,
/// the directory structure, then `<file path="…">` entries. A fresh agent session
/// reads this one file instead of rediscovering the project.
pub fn write_pack(root: &Path, cfg: &Config, l: &Loom) -> Result<Pack> {
    let files: Vec<String> = l
        .inputs
        .iter()
        .flat_map(|p| files_under(root, &expand(p, cfg)))
        .collect();
    let mut body = String::new();
    let mut listed = vec![];
    for f in &files {
        let Ok(text) = fs::read_to_string(root.join(f)) else {
            continue;
        };
        let t = tokens(&text);
        listed.push((f.clone(), t));
        let _ = write!(
            body,
            "<file path=\"{f}\">\n{}\n</file>\n\n",
            text.trim_end()
        );
    }
    let total: usize = listed.iter().map(|(_, t)| t).sum();
    let mut out = String::new();
    let _ = write!(
        out,
        "<file_summary>\nsixloom context pack for the `{id}` loom ({title}), generated {at}.\n\
         Skill: {skill}. Reviewer: {rev}. These {n} files (~{total} tokens) are the loom's only inputs;\n\
         read them instead of exploring the repository. Regenerate with `sixloom loom start {id}`.\n</file_summary>\n\n\
         <directory_structure>\n",
        id = l.id,
        title = l.title,
        at = now(),
        skill = l.skill,
        rev = l.reviewer,
        n = listed.len(),
    );
    for (f, t) in &listed {
        let _ = writeln!(out, "{f}  (~{t} tokens)");
    }
    out.push_str("</directory_structure>\n\n<files>\n");
    out.push_str(&body);
    out.push_str("</files>\n");
    let rel = format!("{CONTEXT_DIR}/{}.xml", l.id);
    fs::create_dir_all(root.join(CONTEXT_DIR))?;
    fs::write(root.join(&rel), out)?;
    Ok(Pack {
        path: rel,
        files: listed,
        tokens: total,
    })
}

pub fn start(root: &Path, cfg: &Config, id: &str, force: bool) -> Result<Pack> {
    let l = loom(id)?;
    let mut st = State::load(root)?;
    let missing = st.deps_approved(l);
    if !missing.is_empty() && !force {
        bail!(
            "`{id}` needs {} approved first (or pass --force)",
            missing.join(", ")
        );
    }
    let pack = write_pack(root, cfg, l)?;
    let e = st.entry(id);
    if e.status != Status::Approved || force {
        e.status = Status::InProgress;
    }
    e.started_at = Some(now());
    st.save(root)?;
    Ok(pack)
}

/// Machine checks a loom must pass before it can be ended. Returns problems.
pub fn check(root: &Path, cfg: &Config, id: &str) -> Result<Vec<String>> {
    let l = loom(id)?;
    let mut p = vec![];
    let exists = |rel: &str| !files_under(root, &expand(rel, cfg)).is_empty();
    let read = |rel: &str| fs::read_to_string(root.join(expand(rel, cfg))).unwrap_or_default();
    for o in l.outputs {
        if !exists(o)
            && !matches!(
                *o,
                "docs/diagrams/" | ".sixloom/runs/" | "justfile" | ".github/workflows/"
            )
        {
            p.push(format!("missing output {}", expand(o, cfg)));
        }
    }
    let trace_gaps = |pred: &dyn Fn(&str) -> bool| -> Vec<String> {
        trace::build(root, &cfg.docs.content_dir, None)
            .map(|t| t.gaps.into_iter().filter(|g| pred(g)).collect())
            .unwrap_or_else(|e| vec![e.to_string()])
    };
    match id {
        "spec" if exists("spec/SPEC.md") => p.extend(trace::load_spec(root)?.problems),
        "design" => {
            for f in ["design/SCREENS.md", "design/DESIGN.md"] {
                if !exists(f) {
                    p.push(format!("missing {f}"));
                }
            }
            p.extend(trace_gaps(&|g| {
                g.contains("not on any screen") || g.contains("not defined")
            }));
        }
        "preview" if !exists("apps/web/PREVIEW.md") => {
            p.push("missing apps/web/PREVIEW.md (screen × state URL table)".into())
        }
        "data" if !read("{content}/architecture/data-model.mdx").contains("erDiagram") => {
            p.push("data-model.mdx has no Mermaid erDiagram".into())
        }
        "system" if !read("{content}/architecture/system-design.mdx").contains("```mermaid") => {
            p.push("system-design.mdx has no Mermaid diagram".into())
        }
        "api" => {
            let text = read("contracts/openapi.yaml");
            match serde_yaml::from_str::<serde_yaml::Value>(&text) {
                Ok(v) if v.get("openapi").is_some() && v.get("paths").is_some() => {}
                Ok(_) => p.push("contracts/openapi.yaml lacks `openapi` or `paths`".into()),
                Err(e) => p.push(format!("contracts/openapi.yaml does not parse: {e}")),
            }
            p.extend(trace_gaps(&|g| {
                g.starts_with("operation") || g.contains("not defined")
            }));
        }
        "tests" | "build" => {
            match lock::verify(root) {
                Ok(v) => p.extend(v),
                Err(e) => p.push(e.to_string()),
            }
            p.extend(trace_gaps(&|g| g.contains("has no test")));
        }
        "validate" if latest_validate(root).is_none() => {
            p.push("no .sixloom/runs/validate-*.md report".into());
        }
        _ => {}
    }
    Ok(p)
}

pub fn end(
    root: &Path,
    cfg: &Config,
    id: &str,
    message: Option<&str>,
    commit: bool,
    skip_checks: bool,
) -> Result<Option<String>> {
    let l = loom(id)?;
    if !skip_checks {
        let problems = check(root, cfg, id)?;
        if !problems.is_empty() {
            bail!(
                "`{id}` is not ready to end:\n  - {}",
                problems.join("\n  - ")
            );
        }
    }
    let mut st = State::load(root)?;
    let e = st.entry(id);
    e.status = Status::Review;
    e.ended_at = Some(now());
    st.save(root)?;
    if !commit {
        return Ok(None);
    }
    let mut paths: Vec<String> = l
        .outputs
        .iter()
        .map(|o| expand(o, cfg))
        .filter(|o| root.join(o).exists())
        .collect();
    paths.extend(["spec".into(), ".sixloom".into()]);
    paths.retain(|p| root.join(p).exists());
    git(
        root,
        &[
            &["add", "--"][..],
            &paths.iter().map(String::as_str).collect::<Vec<_>>(),
        ]
        .concat(),
    )?;
    let msg = format!("sxl({id}): {}", message.unwrap_or(l.title));
    let staged = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(root)
        .status()?;
    if staged.success() {
        return Ok(None);
    }
    git(root, &["commit", "-q", "-m", &msg])?;
    Ok(Some(msg))
}

pub fn approve(root: &Path, cfg: &Config, id: &str) -> Result<String> {
    let l = loom(id)?;
    let mut st = State::load(root)?;
    if st.get(id).status == Status::Pending {
        bail!("`{id}` has not been started");
    }
    let by = author(root);
    let e = st.entry(id);
    e.status = Status::Approved;
    e.approved_at = Some(now());
    e.approved_by = Some(by.clone());
    e.inputs = state::input_hashes(root, cfg, l);
    st.save(root)?;
    Ok(by)
}

pub fn approve_change(root: &Path, id: &str) -> Result<change::Change> {
    let by = author(root);
    let c = change::set_status(root, id, "accepted", Some(&by))?;
    change::render_log(root)?;
    Ok(c)
}

pub fn author(root: &Path) -> String {
    let get = |k: &str| {
        Command::new("git")
            .args(["config", k])
            .current_dir(root)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
    };
    match (get("user.name"), get("user.email")) {
        (Some(n), Some(e)) => format!("{n} <{e}>"),
        (Some(n), None) => n,
        _ => std::env::var("USER").unwrap_or_else(|_| "unknown".into()),
    }
}

fn git(root: &Path, args: &[&str]) -> Result<()> {
    let s = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .context("running git")?;
    if !s.success() {
        bail!("git {} failed", args.join(" "));
    }
    Ok(())
}

/// The looms to work on now, in pipeline order: stale first, then in progress, then ready.
pub fn next(root: &Path, cfg: &Config) -> Result<Vec<(&'static Loom, View)>> {
    let st = State::load(root)?;
    let mut v: Vec<_> = LOOMS.iter().map(|l| (l, st.view(root, cfg, l))).collect();
    v.retain(|(_, s)| {
        matches!(
            s,
            View::Stale | View::InProgress | View::Review | View::Ready
        )
    });
    v.sort_by_key(|(_, s)| match s {
        View::Stale => 0,
        View::Review => 1,
        View::InProgress => 2,
        _ => 3,
    });
    Ok(v)
}

pub fn invocations(cfg: &Config, skill: &str) -> Vec<String> {
    cfg.agents
        .harnesses
        .iter()
        .filter_map(|h| harness::get(h).ok())
        .map(|h| format!("{:<12} {}", h.id, h.invocation(skill)))
        .collect()
}

pub fn latest_validate(root: &Path) -> Option<String> {
    let mut v: Vec<String> = files_under(root, ".sixloom/runs")
        .into_iter()
        .filter(|f| f.contains("/validate-") && f.ends_with(".md"))
        .collect();
    v.sort();
    v.pop()
}

pub struct DoneCheck {
    pub label: &'static str,
    pub ok: bool,
    pub detail: String,
}

/// D-13: the machine-checkable conditions for "done". The human confirms the rest.
pub fn done_checks(root: &Path, cfg: &Config) -> Result<Vec<DoneCheck>> {
    let st = State::load(root)?;
    let mut out = vec![];
    let views: Vec<_> = LOOMS
        .iter()
        .filter(|l| l.id != "validate")
        .map(|l| (l.id, st.view(root, cfg, l)))
        .collect();
    let open: Vec<_> = views
        .iter()
        .filter(|(_, v)| *v != View::Approved)
        .map(|(id, v)| format!("{id}: {}", v.label()))
        .collect();
    out.push(DoneCheck {
        label: "Every loom approved and none stale",
        ok: open.is_empty(),
        detail: open.join(", "),
    });

    let lockp = lock::verify(root).unwrap_or_else(|e| vec![e.to_string()]);
    out.push(DoneCheck {
        label: "Locked tests intact (sixloom tests verify)",
        ok: lockp.is_empty(),
        detail: lockp.join("; "),
    });

    let gaps = trace::build(root, &cfg.docs.content_dir, None)
        .map(|t| t.gaps)
        .unwrap_or_else(|e| vec![e.to_string()]);
    out.push(DoneCheck {
        label: "Trace complete: every AC on a screen, every must AC tested",
        ok: gaps.is_empty(),
        detail: gaps.join("; "),
    });

    let pending: Vec<_> = change::list(root)?
        .into_iter()
        .filter(|c| !["built", "rejected"].contains(&c.status.as_str()))
        .map(|c| format!("{} {}", c.id, c.status))
        .collect();
    out.push(DoneCheck {
        label: "Changelog stack empty (all changes built or rejected)",
        ok: pending.is_empty(),
        detail: pending.join(", "),
    });

    let report = latest_validate(root);
    let pass = report
        .as_ref()
        .and_then(|r| fs::read_to_string(root.join(r)).ok())
        .and_then(|t| crate::catalog::frontmatter_field(&t, "result"))
        .is_some_and(|r| r.eq_ignore_ascii_case("pass"));
    out.push(DoneCheck {
        label: "Latest validate report says result: pass",
        ok: pass,
        detail: report.unwrap_or_else(|| "no report".into()),
    });
    Ok(out)
}

/// Human sign-off after the checks pass; recorded on the validate loom.
pub fn mark_done(root: &Path, cfg: &Config) -> Result<String> {
    let failing: Vec<_> = done_checks(root, cfg)?
        .into_iter()
        .filter(|c| !c.ok)
        .map(|c| c.label)
        .collect();
    if !failing.is_empty() {
        bail!("not done yet:\n  - {}", failing.join("\n  - "));
    }
    approve(root, cfg, "validate")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init;

    #[test]
    fn lifecycle_and_stale_detection() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut cfg = init::default_config("t");
        cfg.agents.harnesses = vec!["codex".into()];
        init::run(root, &cfg, init::Mode::Adopt, false).unwrap();
        assert!(
            start(root, &cfg, "design", false).is_err(),
            "design needs spec"
        );
        start(root, &cfg, "spec", false).unwrap();
        assert!(
            end(root, &cfg, "spec", None, false, false).is_err(),
            "no SPEC.md yet"
        );
        fs::write(
            root.join("spec/SPEC.md"),
            "### F-001 A\n- **Priority**: must\n- **F-001.AC1** Given x, then y.\n",
        )
        .unwrap();
        end(root, &cfg, "spec", None, false, false).unwrap();
        approve(root, &cfg, "spec").unwrap();
        let pack = start(root, &cfg, "design", false).unwrap();
        assert_eq!(pack.files.len(), 1);
        let xml = fs::read_to_string(root.join(&pack.path)).unwrap();
        assert!(xml.contains("<file path=\"spec/SPEC.md\">"));
        fs::create_dir_all(root.join("design")).unwrap();
        fs::write(root.join("design/DESIGN.md"), "# d").unwrap();
        fs::write(
            root.join("design/SCREENS.md"),
            "## S-01 A\n| x | F-001.AC1 |\n",
        )
        .unwrap();
        end(root, &cfg, "design", None, false, false).unwrap();
        approve(root, &cfg, "design").unwrap();
        let st = State::load(root).unwrap();
        assert_eq!(st.view(root, &cfg, loom("design").unwrap()), View::Approved);
        fs::write(
            root.join("spec/SPEC.md"),
            "### F-001 A\n- **F-001.AC1** Given x, then z.\n",
        )
        .unwrap();
        assert_eq!(st.view(root, &cfg, loom("design").unwrap()), View::Stale);
        assert_eq!(
            state::changed_inputs(root, &cfg, loom("design").unwrap(), &st),
            vec!["spec/SPEC.md"]
        );
        assert_eq!(next(root, &cfg).unwrap()[0].0.id, "design");
        assert!(!done_checks(root, &cfg).unwrap()[0].ok);
    }
}
