//! Installing skills into harness directories and into an other-skill bank.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

use crate::catalog::{self, Catalog, Skill};
use crate::config::Config;
use crate::harness;
use crate::state::{hex, now};

/// What sixloom installed, so updates remove only its own skills.
const MANIFEST: &str = ".sixloom/skills.json";
pub const BANK: &str = ".otherskill/bank";
pub const OTHERSKILL_CONFIG: &str = "otherskill.config.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub dirs: Vec<String>,
    pub skills: Vec<String>,
    pub bank: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Report {
    pub native: Vec<String>,
    pub dirs: Vec<String>,
    pub banked: Vec<String>,
    pub removed: Vec<String>,
}

pub fn selected<'a>(cat: &'a Catalog, cfg: &Config) -> Vec<&'a Skill> {
    let c = cfg.choices();
    cat.skill
        .iter()
        .filter(|s| s.matches(&c) || cfg.skills.extra.contains(&s.name))
        .collect()
}

pub fn sync(root: &Path, cfg: &Config) -> Result<Report> {
    let cat = catalog::load()?;
    let chosen = selected(&cat, cfg);
    let (dirs, _) = harness::targets(&cfg.agents.harnesses)?;
    let old: Manifest = fs::read_to_string(root.join(MANIFEST))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    let mut report = Report::default();

    // Remove skills we installed before that are no longer wanted.
    for d in &old.dirs {
        for s in &old.skills {
            let keep = dirs.contains(&d.as_str()) && chosen.iter().any(|c| &c.name == s);
            let p = root.join(d).join(s);
            if !keep && p.join("SKILL.md").is_file() {
                fs::remove_dir_all(&p)?;
                report.removed.push(format!("{d}/{s}"));
            }
        }
    }
    for d in &dirs {
        for s in &chosen {
            let dest = root.join(d).join(&s.name);
            if dest.exists() && !old.skills.contains(&s.name) {
                bail!(
                    "{} already exists and was not installed by sixloom; rename it or remove it",
                    dest.display()
                );
            }
            catalog::extract(s.dir()?, &dest)?;
        }
    }
    report.native = chosen.iter().map(|s| s.name.clone()).collect();
    report.dirs = dirs.iter().map(|d| d.to_string()).collect();

    let banked = if cfg.skills.bank {
        let rest: Vec<&Skill> = cat
            .skill
            .iter()
            .filter(|s| !report.native.contains(&s.name))
            .collect();
        write_bank(root, &cat, &rest, &old.bank)?
    } else {
        vec![]
    };
    report.banked = banked.clone();

    let m = Manifest {
        dirs: report.dirs.clone(),
        skills: report.native.clone(),
        bank: banked,
    };
    fs::create_dir_all(root.join(".sixloom"))?;
    fs::write(
        root.join(MANIFEST),
        serde_json::to_string_pretty(&m)? + "\n",
    )?;
    Ok(report)
}

/// The other-skill bank layout: `<bank>/<category>/<slug>--<logicalId[..16]>/` with `.otherskill.json`.
fn write_bank(
    root: &Path,
    cat: &Catalog,
    skills: &[&Skill],
    old: &[String],
) -> Result<Vec<String>> {
    for rel in old {
        let p = root.join(rel);
        if p.join(".otherskill.json").is_file() {
            fs::remove_dir_all(&p)?;
        }
    }
    let mut out = vec![];
    for s in skills {
        let source = match &s.upstream {
            Some(u) => u.origin(),
            None => format!("sixloom:packs/{}/skills/{}", s.pack, s.name),
        };
        let logical = hex(&Sha256::digest(source.as_bytes()));
        let rel = format!("{BANK}/{}/{}--{}", s.pack, s.name, &logical[..16]);
        let dest = root.join(&rel);
        catalog::extract(s.dir()?, &dest)?;
        let meta = serde_json::json!({
            "version": 1,
            "logicalId": logical,
            "source": source,
            "revision": s.upstream.as_ref().and_then(|u| u.rev.clone()),
            "category": s.pack,
            "installedAt": now(),
        });
        fs::write(
            dest.join(".otherskill.json"),
            serde_json::to_string_pretty(&meta)? + "\n",
        )?;
        out.push(rel);
    }
    write_otherskill_config(root, cat)?;
    Ok(out)
}

fn write_otherskill_config(root: &Path, cat: &Catalog) -> Result<()> {
    let p = root.join(OTHERSKILL_CONFIG);
    let mut cfg: serde_json::Value = match fs::read_to_string(&p) {
        Ok(t) => serde_json::from_str(&t).with_context(|| format!("parsing {}", p.display()))?,
        Err(_) => serde_json::json!({
            "bankPath": BANK,
            "model": "inception/mercury-decide:free",
            "scope": "project",
            "categories": [],
        }),
    };
    let cats = cfg["categories"]
        .as_array_mut()
        .context("otherskill categories must be an array")?;
    for pack in &cat.pack {
        if !cats.iter().any(|c| c["id"] == pack.id.as_str()) {
            cats.push(serde_json::json!({ "id": pack.id, "description": pack.description }));
        }
    }
    fs::write(&p, serde_json::to_string_pretty(&cfg)? + "\n")?;
    Ok(())
}

/// Copy a user's own skill (a directory with SKILL.md) into every harness dir.
pub fn add_local(root: &Path, cfg: &Config, src: &Path) -> Result<String> {
    let md = fs::read_to_string(src.join("SKILL.md"))
        .with_context(|| format!("{} has no SKILL.md", src.display()))?;
    let name =
        catalog::frontmatter_field(&md, "name").context("SKILL.md frontmatter needs a name")?;
    if catalog::frontmatter_field(&md, "description").is_none() {
        bail!("SKILL.md frontmatter needs a description");
    }
    let (dirs, _) = harness::targets(&cfg.agents.harnesses)?;
    for d in dirs {
        copy_dir(src, &root.join(d).join(&name))?;
    }
    Ok(name)
}

fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        if e.file_name() == ".git" {
            continue;
        }
        let to = dest.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &to)?;
        } else {
            fs::copy(e.path(), to)?;
        }
    }
    Ok(())
}
