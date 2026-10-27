//! `sixloom init`: scaffold a new project or adopt an existing one.

use anyhow::{bail, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::catalog::template;
use crate::config::{self, Config};
use crate::harness;
use crate::skills::{self, Report};
use crate::state::State;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    New,
    Adopt,
}

pub struct Outcome {
    pub created: Vec<String>,
    pub skills: Report,
}

const BEGIN: &str = "<!-- sixloom:begin";
const END: &str = "<!-- sixloom:end -->";

pub fn run(root: &Path, cfg: &Config, mode: Mode, force: bool) -> Result<Outcome> {
    cfg.validate()?;
    if root.join(config::FILE).exists() && !force {
        bail!(
            "{} already has {}; use `sixloom update` or pass --force",
            root.display(),
            config::FILE
        );
    }
    if mode == Mode::New
        && root.exists()
        && fs::read_dir(root)?.any(|e| e.is_ok_and(|e| e.file_name() != ".git"))
        && !force
    {
        bail!(
            "{} is not empty; use --adopt to add sixloom to an existing project",
            root.display()
        );
    }
    fs::create_dir_all(root)?;
    let mut created = vec![];

    let in_repo = inside_git(root);
    if !in_repo && git(root, &["init", "-q"]) {
        created.push(".git/".into());
    }

    cfg.save(root)?;
    created.push(config::FILE.into());
    if !root.join(crate::state::STATE).exists() {
        State::default().save(root)?;
        created.push(crate::state::STATE.into());
    }
    for d in [
        "spec/changes",
        "design/exports",
        "contracts",
        ".sixloom/runs",
    ] {
        let p = root.join(d);
        if !p.exists() {
            fs::create_dir_all(&p)?;
            fs::write(p.join(".gitkeep"), "")?;
            created.push(format!("{d}/"));
        }
    }
    merge_gitignore(root)?;
    created.push(".gitignore".into());
    for f in write_instructions(root, cfg)? {
        created.push(f);
    }
    let skills = skills::sync(root, cfg)?;
    commit_scaffold(root, cfg, &skills.native)?;
    Ok(Outcome { created, skills })
}

fn git(root: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn inside_git(root: &Path) -> bool {
    git(root, &["rev-parse", "--is-inside-work-tree"])
}

/// Commit only the files sixloom wrote, leaving anything else the user has staged alone.
fn commit_scaffold(root: &Path, cfg: &Config, installed: &[String]) -> Result<()> {
    if !inside_git(root) {
        return Ok(());
    }
    let (dirs, files) = harness::targets(&cfg.agents.harnesses)?;
    let mut paths: Vec<String> = [
        config::FILE,
        ".sixloom",
        "spec",
        "design",
        "contracts",
        ".gitignore",
    ]
    .iter()
    .map(|s| s.to_string())
    // Only the skills sixloom installed: the user's own skills in the same dirs stay theirs.
    .chain(
        dirs.iter()
            .flat_map(|d| installed.iter().map(move |s| format!("{d}/{s}"))),
    )
    .chain(files.iter().map(|f| f.to_string()))
    .collect();
    if cfg.skills.bank {
        paths.extend([
            ".otherskill".to_string(),
            crate::skills::OTHERSKILL_CONFIG.to_string(),
        ]);
    }
    paths.retain(|p| root.join(p).exists());
    let mut add = vec!["add", "--"];
    add.extend(paths.iter().map(String::as_str));
    if !git(root, &add) {
        return Ok(());
    }
    let mut commit = vec!["commit", "-q", "-m", "sxl(init): add sixloom", "--"];
    commit.extend(paths.iter().map(String::as_str));
    // No identity configured or nothing to commit: leave the files staged.
    git(root, &commit);
    Ok(())
}

/// Write or refresh the managed block in every harness instruction file.
pub fn write_instructions(root: &Path, cfg: &Config) -> Result<Vec<String>> {
    let (dirs, files) = harness::targets(&cfg.agents.harnesses)?;
    let block = template("project/agents-block.md")?
        .replace("{{content_dir}}", &cfg.docs.content_dir)
        .replace("{{frontend}}", &cfg.frontend.framework)
        .replace("{{ui_kit}}", &cfg.frontend.ui_kit)
        .replace("{{docs}}", &cfg.docs.engine)
        .replace("{{design}}", &cfg.design.tool)
        .replace(
            "{{skills_dirs}}",
            &dirs
                .iter()
                .map(|d| format!("`{d}/`"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    let mut written = vec![];
    for f in files {
        let p = root.join(f);
        let old = fs::read_to_string(&p).unwrap_or_default();
        let new = match (old.find(BEGIN), old.find(END)) {
            (Some(a), Some(b)) if b > a => {
                format!("{}{}{}", &old[..a], block.trim_end(), &old[b + END.len()..])
            }
            _ if old.trim().is_empty() => block.clone(),
            _ => format!("{}\n\n{}", old.trim_end(), block),
        };
        fs::write(&p, new)?;
        written.push(f.to_string());
    }
    Ok(written)
}

fn merge_gitignore(root: &Path) -> Result<()> {
    let p = root.join(".gitignore");
    let mut text = fs::read_to_string(&p).unwrap_or_default();
    for line in template("project/gitignore.txt")?.lines() {
        if !text.lines().any(|l| l.trim() == line.trim()) {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(line);
            text.push('\n');
        }
    }
    fs::write(p, text)?;
    Ok(())
}

pub fn default_config(name: &str) -> Config {
    Config {
        project: config::Project {
            name: name.to_string(),
        },
        agents: config::Agents {
            harnesses: vec!["claude".into(), "codex".into()],
        },
        frontend: config::Frontend {
            framework: "react-vite".into(),
            ui_kit: "shadcn".into(),
        },
        docs: config::Docs {
            engine: "fumadocs".into(),
            content_dir: config::content_dir_for("fumadocs").into(),
        },
        design: config::Design {
            tool: "claude".into(),
            fidelity: "key".into(),
        },
        build: Default::default(),
        validate: Default::default(),
        skills: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_new_project_installs_selected_skills() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("app");
        let mut cfg = default_config("app");
        cfg.agents.harnesses = vec!["claude".into(), "codex".into(), "kimi".into()];
        cfg.skills.bank = true;
        let out = run(&root, &cfg, Mode::New, false).unwrap();
        assert_eq!(out.skills.dirs, vec![".claude/skills", ".agents/skills"]);
        for d in [".claude/skills", ".agents/skills"] {
            assert!(root.join(d).join("sxl-specify/SPEC.template.md").is_file());
            assert!(root
                .join(d)
                .join("vercel-react-best-practices/SKILL.md")
                .is_file());
            assert!(root.join(d).join("sxl-docs-fumadocs/SKILL.md").is_file());
            assert!(!root.join(d).join("nuxt").exists());
        }
        let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(agents.contains("front end **react-vite**"));
        assert!(root.join("CLAUDE.md").is_file());
        // Unselected variants land in the other-skill bank.
        let bank = fs::read_dir(root.join(".otherskill/bank/frontend"))
            .unwrap()
            .count();
        assert!(bank >= 4);
        let osc: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(root.join("otherskill.config.json")).unwrap())
                .unwrap();
        assert_eq!(osc["bankPath"], ".otherskill/bank");
        assert!(
            run(&root, &cfg, Mode::New, false).is_err(),
            "second init refuses"
        );
    }

    #[test]
    fn update_switches_variants_and_keeps_user_content() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("AGENTS.md"), "# My rules\n\nKeep this.\n").unwrap();
        fs::create_dir_all(root.join(".agents/skills/my-own")).unwrap();
        fs::write(
            root.join(".agents/skills/my-own/SKILL.md"),
            "---\nname: my-own\ndescription: x\n---\n",
        )
        .unwrap();
        let mut cfg = default_config("x");
        cfg.agents.harnesses = vec!["codex".into()];
        run(root, &cfg, Mode::Adopt, false).unwrap();
        cfg.frontend.framework = "vue".into();
        cfg.frontend.ui_kit = "none".into();
        cfg.save(root).unwrap();
        let rep = skills::sync(root, &cfg).unwrap();
        assert!(rep
            .removed
            .iter()
            .any(|r| r.ends_with("vercel-react-best-practices")));
        assert!(root.join(".agents/skills/vue/SKILL.md").is_file());
        assert!(root.join(".agents/skills/my-own/SKILL.md").is_file());
        write_instructions(root, &cfg).unwrap();
        let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
        assert!(agents.starts_with("# My rules\n\nKeep this."));
        assert_eq!(agents.matches("sixloom:begin").count(), 1);
        assert!(agents.contains("front end **vue**"));
    }

    #[test]
    fn adopt_commits_only_sixloom_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let g = |args: &[&str]| assert!(git(root, args), "git {args:?}");
        g(&["init", "-q"]);
        g(&["config", "user.email", "t@t.io"]);
        g(&["config", "user.name", "T"]);
        fs::write(root.join("app.js"), "1").unwrap();
        g(&["add", "-A"]);
        g(&["commit", "-qm", "legacy"]);
        fs::write(root.join("app.js"), "2").unwrap();
        fs::create_dir_all(root.join(".claude/skills/mine")).unwrap();
        fs::write(
            root.join(".claude/skills/mine/SKILL.md"),
            "---\nname: mine\ndescription: x\n---\n",
        )
        .unwrap();
        let mut cfg = default_config("x");
        cfg.agents.harnesses = vec!["claude".into()];
        run(root, &cfg, Mode::Adopt, false).unwrap();
        let out = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output()
            .unwrap();
        let status = String::from_utf8_lossy(&out.stdout);
        assert!(
            status.contains(" M app.js"),
            "user edits stay uncommitted: {status}"
        );
        assert!(
            status.contains("?? .claude/skills/mine/"),
            "user skill stays untracked: {status}"
        );
        assert_eq!(
            status.lines().count(),
            2,
            "everything sixloom wrote is committed: {status}"
        );
    }
}
