//! Coding-agent harnesses: where each one discovers project skills and instructions.
//!
//! Paths were checked against each tool's shipped code (see docs/HARNESSES.md);
//! `verified` says whether sixloom's own smoke test covers the harness.

use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy)]
pub struct Harness {
    pub id: &'static str,
    pub name: &'static str,
    /// Project-relative directory the harness scans for `<skill>/SKILL.md`.
    pub skills_dir: &'static str,
    /// Project instruction file the harness loads every session.
    pub instructions: &'static str,
    /// Binary to look for on PATH (`sixloom doctor`).
    pub binary: &'static str,
    /// How a user invokes a skill by name in this harness.
    pub invoke: &'static str,
    pub verified: bool,
}

pub const ALL: &[Harness] = &[
    Harness {
        id: "claude",
        name: "Claude Code",
        skills_dir: ".claude/skills",
        instructions: "CLAUDE.md",
        binary: "claude",
        invoke: "/{skill}",
        verified: true,
    },
    Harness {
        id: "codex",
        name: "OpenAI Codex CLI",
        skills_dir: ".agents/skills",
        instructions: "AGENTS.md",
        binary: "codex",
        invoke: "${skill}",
        verified: true,
    },
    Harness {
        id: "opencode",
        name: "OpenCode",
        skills_dir: ".agents/skills",
        instructions: "AGENTS.md",
        binary: "opencode",
        invoke: "use the {skill} skill",
        verified: true,
    },
    Harness {
        id: "gemini",
        name: "Gemini CLI",
        skills_dir: ".agents/skills",
        instructions: "GEMINI.md",
        binary: "gemini",
        invoke: "use the {skill} skill",
        verified: true,
    },
    Harness {
        id: "kimi",
        name: "Kimi Code CLI",
        skills_dir: ".agents/skills",
        instructions: "AGENTS.md",
        binary: "kimi",
        invoke: "/skill:{skill}",
        verified: true,
    },
    Harness {
        id: "antigravity",
        name: "Google Antigravity",
        skills_dir: ".agents/skills",
        instructions: "AGENTS.md",
        binary: "agy",
        invoke: "use the {skill} skill",
        verified: false,
    },
    Harness {
        id: "cursor",
        name: "Cursor",
        skills_dir: ".cursor/skills",
        instructions: "AGENTS.md",
        binary: "cursor-agent",
        invoke: "/{skill}",
        verified: false,
    },
    Harness {
        id: "copilot",
        name: "GitHub Copilot CLI",
        skills_dir: ".github/skills",
        instructions: "AGENTS.md",
        binary: "copilot",
        invoke: "use the {skill} skill",
        verified: false,
    },
    Harness {
        id: "amp",
        name: "Amp",
        skills_dir: ".agents/skills",
        instructions: "AGENTS.md",
        binary: "amp",
        invoke: "use the {skill} skill",
        verified: false,
    },
];

pub fn get(id: &str) -> Result<&'static Harness> {
    match ALL.iter().find(|h| h.id == id) {
        Some(h) => Ok(h),
        None => {
            let ids: Vec<_> = ALL.iter().map(|h| h.id).collect();
            bail!("unknown agent '{id}', expected one of: {}", ids.join(", "))
        }
    }
}

impl Harness {
    pub fn invocation(&self, skill: &str) -> String {
        self.invoke.replace("{skill}", skill)
    }

    pub fn on_path(&self) -> bool {
        which(self.binary)
    }
}

pub fn which(bin: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| {
        let p = dir.join(bin);
        p.is_file() || p.with_extension("exe").is_file() || p.with_extension("cmd").is_file()
    })
}

/// Distinct skill directories and instruction files for a set of harnesses.
pub fn targets(ids: &[String]) -> Result<(Vec<&'static str>, Vec<&'static str>)> {
    let mut dirs = vec![];
    let mut files = vec![];
    for id in ids {
        let h = get(id)?;
        if !dirs.contains(&h.skills_dir) {
            dirs.push(h.skills_dir);
        }
        if !files.contains(&h.instructions) {
            files.push(h.instructions);
        }
    }
    Ok((dirs, files))
}
