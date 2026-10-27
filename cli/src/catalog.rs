//! Embedded skill packs and the catalog that says which skill installs when.

use anyhow::{anyhow, Context, Result};
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub static PACKS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../packs");
pub static TEMPLATES: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../templates");

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub pack: Vec<Pack>,
    pub skill: Vec<Skill>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Pack {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Skill {
    pub name: String,
    pub pack: String,
    #[serde(default)]
    pub when: When,
    pub upstream: Option<Upstream>,
    pub ported_from: Option<Upstream>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Upstream {
    pub repo: Option<String>,
    pub rev: Option<String>,
    pub npm: Option<String>,
    pub path: String,
    pub license: String,
}

impl Upstream {
    /// `github:owner/repo/path@rev` or `npm:pkg@version/path`.
    pub fn origin(&self) -> String {
        match (&self.npm, &self.repo) {
            (Some(n), _) => format!("npm:{n}/{}", self.path),
            (None, Some(r)) => format!(
                "github:{r}/{}@{}",
                self.path,
                self.rev.as_deref().unwrap_or("HEAD")
            ),
            _ => self.path.clone(),
        }
    }
    pub fn label(&self) -> String {
        self.repo
            .clone()
            .or_else(|| self.npm.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum When {
    One(String),
    Any(Vec<String>),
}

impl Default for When {
    fn default() -> Self {
        When::Any(vec![])
    }
}

/// The project choices a `when` condition is matched against.
pub struct Choices<'a> {
    pub frontend: &'a str,
    pub ui_kit: &'a str,
    pub docs: &'a str,
    pub design: &'a str,
}

impl Skill {
    pub fn matches(&self, c: &Choices) -> bool {
        let conds: Vec<&str> = match &self.when {
            When::One(s) => vec![s.as_str()],
            When::Any(v) => v.iter().map(String::as_str).collect(),
        };
        conds.iter().any(|cond| match cond.split_once(':') {
            None => *cond == "always",
            Some(("frontend", v)) => v == c.frontend,
            Some(("ui", v)) => v == c.ui_kit,
            Some(("docs", v)) => v == c.docs,
            Some(("design", v)) => v == c.design,
            _ => false,
        })
    }

    pub fn dir(&self) -> Result<&'static Dir<'static>> {
        let p = format!("{}/skills/{}", self.pack, self.name);
        PACKS
            .get_dir(&p)
            .ok_or_else(|| anyhow!("skill {} missing from embedded packs ({p})", self.name))
    }

    pub fn source(&self) -> String {
        match (&self.upstream, &self.ported_from) {
            (Some(u), _) => format!("{} ({})", u.label(), u.license),
            (None, Some(u)) => format!("sixloom, ported from {} ({})", u.label(), u.license),
            _ => "sixloom".into(),
        }
    }

    #[cfg(test)]
    pub fn description(&self) -> String {
        let Ok(dir) = self.dir() else {
            return String::new();
        };
        let Some(md) = dir.get_file(dir.path().join("SKILL.md")) else {
            return String::new();
        };
        frontmatter_field(md.contents_utf8().unwrap_or(""), "description").unwrap_or_default()
    }
}

pub fn load() -> Result<Catalog> {
    let file = PACKS
        .get_file("catalog.toml")
        .ok_or_else(|| anyhow!("embedded catalog.toml missing"))?;
    toml::from_str(file.contents_utf8().unwrap_or_default()).context("parsing packs/catalog.toml")
}

/// Write an embedded directory to `dest`, replacing what is there.
pub fn extract(dir: &Dir, dest: &Path) -> Result<()> {
    if dest.exists() {
        fs::remove_dir_all(dest).with_context(|| format!("removing {}", dest.display()))?;
    }
    write_tree(dir, dir.path(), dest)
}

fn write_tree(dir: &Dir, root: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for f in dir.files() {
        let rel = f.path().strip_prefix(root).unwrap_or(f.path());
        let out = dest.join(rel);
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out, f.contents()).with_context(|| format!("writing {}", out.display()))?;
    }
    for d in dir.dirs() {
        write_tree(d, root, dest)?;
    }
    Ok(())
}

pub fn template(path: &str) -> Result<&'static str> {
    TEMPLATES
        .get_file(path)
        .and_then(|f| f.contents_utf8())
        .ok_or_else(|| anyhow!("embedded template {path} missing"))
}

pub fn pack_file(path: &str) -> Result<&'static str> {
    PACKS
        .get_file(path)
        .and_then(|f| f.contents_utf8())
        .ok_or_else(|| anyhow!("embedded pack file {path} missing"))
}

/// Read a scalar field from YAML frontmatter, handling `>-` folded blocks.
pub fn frontmatter_field(md: &str, key: &str) -> Option<String> {
    let body = md.strip_prefix("---")?;
    let end = body.find("\n---")?;
    let value: serde_yaml::Value = serde_yaml::from_str(&body[..end]).ok()?;
    match value.get(key)? {
        serde_yaml::Value::String(s) => Some(s.trim().to_string()),
        serde_yaml::Value::Sequence(items) => Some(
            items
                .iter()
                .map(|i| {
                    serde_yaml::to_string(i)
                        .unwrap_or_default()
                        .trim()
                        .to_string()
                })
                .collect::<Vec<_>>()
                .join(", "),
        ),
        serde_yaml::Value::Null => None,
        other => Some(serde_yaml::to_string(other).ok()?.trim().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_skills_all_embedded() {
        let cat = load().unwrap();
        for s in &cat.skill {
            let dir = s.dir().unwrap();
            let md = dir.get_file(dir.path().join("SKILL.md")).expect("SKILL.md");
            let name = frontmatter_field(md.contents_utf8().unwrap(), "name").unwrap();
            assert_eq!(name, s.name, "frontmatter name must equal catalog name");
            assert!(!s.description().is_empty(), "{} has a description", s.name);
            assert!(cat.pack.iter().any(|p| p.id == s.pack));
        }
    }

    #[test]
    fn when_matching() {
        let cat = load().unwrap();
        let c = Choices {
            frontend: "vue",
            ui_kit: "none",
            docs: "starlight",
            design: "stitch",
        };
        let names: Vec<_> = cat
            .skill
            .iter()
            .filter(|s| s.matches(&c))
            .map(|s| s.name.as_str())
            .collect();
        assert!(names.contains(&"vue"));
        assert!(names.contains(&"sxl-docs-starlight"));
        assert!(names.contains(&"stitch"));
        assert!(!names.contains(&"nuxt"));
        assert!(!names.contains(&"vercel-react-best-practices"));
        assert!(!names.contains(&"webapp-testing"));
    }
}
