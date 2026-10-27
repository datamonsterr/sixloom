//! Locked acceptance tests: `sixloom tests lock` / `sixloom tests verify`.
//!
//! A test's fingerprint is its title plus the ordered list of assertion matchers
//! with their arguments (`.not.toHaveText("Saved")`). The subject inside `expect(…)`
//! is not part of it, so selector and wait repairs are allowed, while removing,
//! reordering or loosening an assertion changes the fingerprint.

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::state::{files_under, hex, now};
use crate::trace::is_test_source;

pub const LOCK: &str = ".sixloom/tests.lock";
const NOT_MATCHERS: &[&str] = &[
    "toString",
    "toFixed",
    "toJSON",
    "toLocaleString",
    "toLowerCase",
    "toUpperCase",
    "toISOString",
    "toPrecision",
];

pub const LOCKED_DIRS: &[&str] = &["tests/e2e", "tests/contract", "tests/integration"];

#[derive(Debug, Serialize, Deserialize)]
pub struct Lock {
    pub version: u32,
    pub locked_at: String,
    /// `<file>::<test title>` → fingerprint
    pub tests: BTreeMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub struct TestCase {
    pub title: String,
    pub modifier: Option<String>,
    pub fingerprint: String,
    pub assertions: usize,
}

pub fn parse(src: &str) -> Vec<TestCase> {
    let start = Regex::new(
        r#"\b(?:test|it)((?:\.(?:skip|only|fixme|fail|todo|slow|step))*)\s*\(\s*(?:'((?:[^'\\]|\\.)*)'|"((?:[^"\\]|\\.)*)"|`([^`]*)`)"#,
    )
    .unwrap();
    let matcher =
        Regex::new(r"\.\s*(not\s*\.\s*)?((?:to|resolves\.to|rejects\.to)[A-Z]\w*)\s*\(").unwrap();
    let starts: Vec<_> = start.captures_iter(src).collect();
    let mut out = vec![];
    for (i, c) in starts.iter().enumerate() {
        let title = c
            .get(2)
            .or(c.get(3))
            .or(c.get(4))
            .map(|m| m.as_str())
            .unwrap_or_default();
        let m0 = c.get(0).unwrap();
        let end = starts
            .get(i + 1)
            .map(|n| n.get(0).unwrap().start())
            .unwrap_or(src.len());
        let body = &src[m0.end()..end];
        let mods = c.get(1).map(|m| m.as_str()).unwrap_or_default();
        let modifier = ["skip", "only", "fixme", "fail", "todo"]
            .iter()
            .find(|m| mods.contains(&format!(".{m}")))
            .map(|m| m.to_string());
        let mut h = Sha256::new();
        h.update(title.as_bytes());
        let mut n = 0;
        for m in matcher.captures_iter(body) {
            if NOT_MATCHERS.contains(&&m[2]) {
                continue;
            }
            let open = m.get(0).unwrap().end();
            let args = balanced(&body[open..]);
            let norm: String = format!("{}{}({})", m.get(1).map_or("", |_| "not."), &m[2], args)
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            h.update([0]);
            h.update(norm.as_bytes());
            n += 1;
        }
        out.push(TestCase {
            title: title.to_string(),
            modifier,
            fingerprint: hex(&h.finalize())[..16].to_string(),
            assertions: n,
        });
    }
    out
}

/// Text up to the parenthesis that closes an already-open one.
fn balanced(s: &str) -> &str {
    let mut depth = 1;
    let mut quote: Option<char> = None;
    let mut prev = ' ';
    for (i, ch) in s.char_indices() {
        match quote {
            Some(q) if ch == q && prev != '\\' => quote = None,
            Some(_) => {}
            None => match ch {
                '\'' | '"' | '`' => quote = Some(ch),
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return &s[..i];
                    }
                }
                _ => {}
            },
        }
        prev = ch;
    }
    s
}

fn collect(root: &Path) -> Result<BTreeMap<String, (String, TestCase)>> {
    let mut all = BTreeMap::new();
    for dir in LOCKED_DIRS {
        for f in files_under(root, dir) {
            if !is_test_source(&f) {
                continue;
            }
            let src = fs::read_to_string(root.join(&f))?;
            for t in parse(&src) {
                all.insert(format!("{f}::{}", t.title), (f.clone(), t));
            }
        }
    }
    Ok(all)
}

pub fn lock(root: &Path) -> Result<(usize, Vec<String>)> {
    let all = collect(root)?;
    let mut warnings = vec![];
    let mut tests = BTreeMap::new();
    for (key, (_, t)) in &all {
        if let Some(m) = &t.modifier {
            bail!("cannot lock '{key}': it is marked .{m}");
        }
        if t.assertions == 0 {
            warnings.push(format!("{key} has no assertions"));
        }
        tests.insert(key.clone(), t.fingerprint.clone());
    }
    if tests.is_empty() {
        bail!("no tests found under {}", LOCKED_DIRS.join(", "));
    }
    let lock = Lock {
        version: 1,
        locked_at: now(),
        tests,
    };
    fs::create_dir_all(root.join(".sixloom"))?;
    fs::write(root.join(LOCK), serde_json::to_string_pretty(&lock)? + "\n")?;
    Ok((lock.tests.len(), warnings))
}

pub fn verify(root: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(root.join(LOCK))
        .with_context(|| format!("no {LOCK}; run `sixloom tests lock`"))?;
    let lock: Lock = serde_json::from_str(&text)?;
    let all = collect(root)?;
    let mut problems = vec![];
    for (key, fp) in &lock.tests {
        match all.get(key) {
            None => problems.push(format!("locked test removed or renamed: {key}")),
            Some((_, t)) if t.modifier.is_some() => problems.push(format!(
                "locked test disabled with .{}: {key}",
                t.modifier.as_deref().unwrap()
            )),
            Some((_, t)) if &t.fingerprint != fp => {
                problems.push(format!("assertions changed in locked test: {key}"))
            }
            _ => {}
        }
    }
    for (key, (_, t)) in &all {
        if let (false, Some(m)) = (lock.tests.contains_key(key), &t.modifier) {
            if m == "only" {
                problems.push(format!("test.only left in: {key}"));
            }
        }
    }
    Ok(problems)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORIG: &str = r#"
test('F-001.AC1 shows projects', async ({ page }) => {
  await page.goto('/projects');
  await expect(page.getByTestId('S-02.row')).toHaveCount(3);
  await expect(page.getByRole('heading')).toHaveText("Projects (3)");
});
test('F-001.AC2 empty', async ({ page }) => {
  await expect(page.locator('.empty')).toBeVisible();
});
"#;

    #[test]
    fn selector_repair_keeps_fingerprint() {
        let repaired = ORIG
            .replace(
                "page.getByTestId('S-02.row')",
                "page.locator('[data-testid=\"S-02.row\"]')",
            )
            .replace(
                "await page.goto('/projects');",
                "await page.goto('/projects');\n  await page.waitForLoadState();",
            );
        assert_eq!(parse(ORIG), parse(&repaired));
    }

    #[test]
    fn weakened_assertion_changes_fingerprint() {
        let weak = ORIG.replace("toHaveText(\"Projects (3)\")", "toHaveText(/Projects/)");
        assert_ne!(parse(ORIG)[0].fingerprint, parse(&weak)[0].fingerprint);
        let removed = ORIG.replace(
            "  await expect(page.getByTestId('S-02.row')).toHaveCount(3);\n",
            "",
        );
        assert_ne!(parse(ORIG)[0].fingerprint, parse(&removed)[0].fingerprint);
        let negated = ORIG.replace(").toBeVisible()", ").not.toBeVisible()");
        assert_ne!(parse(ORIG)[1].fingerprint, parse(&negated)[1].fingerprint);
    }

    #[test]
    fn detects_modifiers() {
        let skipped = ORIG.replace("test('F-001.AC2", "test.skip('F-001.AC2");
        assert_eq!(parse(&skipped)[1].modifier.as_deref(), Some("skip"));
        assert_eq!(parse(ORIG)[0].assertions, 2);
    }

    #[test]
    fn lock_then_verify() {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();
        fs::create_dir_all(r.join("tests/e2e")).unwrap();
        fs::write(r.join("tests/e2e/f001.spec.ts"), ORIG).unwrap();
        assert_eq!(lock(r).unwrap().0, 2);
        assert!(verify(r).unwrap().is_empty());
        fs::write(
            r.join("tests/e2e/f001.spec.ts"),
            ORIG.replace("toHaveCount(3)", "toHaveCount(2)"),
        )
        .unwrap();
        assert_eq!(verify(r).unwrap().len(), 1);
        fs::write(
            r.join("tests/e2e/f001.spec.ts"),
            ORIG.replace("test('F-001.AC2", "test.skip('F-001.AC2"),
        )
        .unwrap();
        assert!(verify(r).unwrap()[0].contains("disabled"));
    }
}
