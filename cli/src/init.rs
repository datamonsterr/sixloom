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

    if !root.join(".git").exists() && Command::new("git").arg("--version").output().is_ok() {
        let ok = Command::new("git")
            .args(["init", "-q"])
            .current_dir(root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            created.push(".git/".into());
        }
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
    Ok(Outcome { created, skills })
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
}
