//! Locked acceptance tests: `sixloom tests lock` / `sixloom tests verify`.
//!
//! A test's fingerprint is its title plus the ordered list of its assertions.
//! JS/TS (Playwright, Vitest, Jest): matchers with their arguments
//! (`.not.toHaveText("Saved")`); the subject inside `expect(…)` is not part of it, so
//! selector and wait repairs are allowed. Rust (`assert*!`, snapshot macros), Python
//! (`assert …`, Playwright `to_*`) and Go (testify, `t.Error*`/`t.Fatal*`): each full
//! assertion. Removing, reordering or loosening an assertion changes the fingerprint,
//! and skip markers (`.skip`, `#[ignore]`, `@pytest.mark.skip`, `t.Skip`) count as disabling.

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

/// Default locked folders; projects can change them in sixloom.toml `[tests] locked`.
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

/// Parse the tests in one source file, by language.
pub fn parse_file(path: &str, src: &str) -> Vec<TestCase> {
    match path.rsplit('.').next().unwrap_or_default() {
        "rs" => parse_rust(src),
        "py" => parse_python(src),
        "go" => parse_go(src),
        _ => parse(src),
    }
}

/// Tests found by a language's start pattern: (title, header incl. attributes, body).
fn cases_by_start<'a>(
    src: &'a str,
    start: &Regex,
    title_group: usize,
) -> Vec<(String, &'a str, &'a str)> {
    let starts: Vec<_> = start.captures_iter(src).collect();
    starts
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let m0 = c.get(0).unwrap();
            let end = starts
                .get(i + 1)
                .map(|n| n.get(0).unwrap().start())
                .unwrap_or(src.len());
            (c[title_group].to_string(), m0.as_str(), &src[m0.end()..end])
        })
        .collect()
}

/// Fingerprint the assertions matched by `re` (full call with balanced arguments).
fn fingerprint(title: &str, skipped: bool, modifier: &str, body: &str, re: &Regex) -> TestCase {
    let mut h = Sha256::new();
    h.update(title.as_bytes());
    let mut n = 0;
    for m in re.find_iter(body) {
        let call = m.as_str();
        let text = if call.ends_with('(') {
            format!("{call}{})", balanced(&body[m.end()..]))
        } else {
            call.to_string()
        };
        h.update([0]);
        h.update(
            text.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .as_bytes(),
        );
        n += 1;
    }
    TestCase {
        title: title.to_string(),
        modifier: skipped.then(|| modifier.to_string()),
        fingerprint: hex(&h.finalize())[..16].to_string(),
        assertions: n,
    }
}

fn parse_rust(src: &str) -> Vec<TestCase> {
    let start = Regex::new(
        r"(?m)((?:^[ \t]*#\[[^\]]*\][ \t]*\n)+)^[ \t]*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(",
    )
    .unwrap();
    let assert = Regex::new(
        r"\b(?:assert(?:_eq|_ne|_matches)?|debug_assert\w*|(?:insta::)?assert_\w*snapshot)!\s*\(",
    )
    .unwrap();
    cases_by_start(src, &start, 2)
        .into_iter()
        // Only functions carrying a test attribute (#[test], #[tokio::test], …).
        .filter(|(_, head, _)| {
            head.lines()
                .any(|l| l.trim_start().starts_with("#[") && l.contains("test"))
        })
        .map(|(title, head, body)| {
            fingerprint(&title, head.contains("#[ignore"), "ignore", body, &assert)
        })
        .collect()
}

fn parse_python(src: &str) -> Vec<TestCase> {
    let start = Regex::new(r"(?m)((?:^[ \t]*@[^\n]*\n)*)^[ \t]*(?:async\s+)?def\s+(test_\w+)\s*\(")
        .unwrap();
    let assert = Regex::new(r"(?m)^[ \t]*assert\s[^\n]+|\.(?:not_)?to_\w+\s*\(").unwrap();
    cases_by_start(src, &start, 2)
        .into_iter()
        .map(|(title, head, body)| {
            let skipped = head.contains(".skip") || head.contains(".xfail");
            fingerprint(&title, skipped, "skip", body, &assert)
        })
        .collect()
}

fn parse_go(src: &str) -> Vec<TestCase> {
    let start = Regex::new(r"(?m)^func\s+(Test\w+)\s*\(\s*\w+\s+\*testing\.T\s*\)").unwrap();
    let assert =
        Regex::new(r"\b(?:assert|require)\.\w+\s*\(|\bt\.(?:Errorf?|Fatalf?)\s*\(").unwrap();
    cases_by_start(src, &start, 1)
        .into_iter()
        .map(|(title, _, body)| fingerprint(&title, body.contains("t.Skip"), "Skip", body, &assert))
        .collect()
}

/// JS/TS tests: `test('…', …)` / `it('…', …)`.
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

fn collect(root: &Path, dirs: &[String]) -> Result<BTreeMap<String, (String, TestCase)>> {
    let mut all = BTreeMap::new();
    for dir in dirs {
        for f in files_under(root, dir) {
            if !is_test_source(&f) {
                continue;
            }
            let src = fs::read_to_string(root.join(&f))?;
            for t in parse_file(&f, &src) {
                all.insert(format!("{f}::{}", t.title), (f.clone(), t));
            }
        }
    }
    Ok(all)
}

pub fn lock(root: &Path, dirs: &[String]) -> Result<(usize, Vec<String>)> {
    let all = collect(root, dirs)?;
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
        bail!("no tests found under {}", dirs.join(", "));
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

pub fn verify(root: &Path, dirs: &[String]) -> Result<Vec<String>> {
    let text = fs::read_to_string(root.join(LOCK))
        .with_context(|| format!("no {LOCK}; run `sixloom tests lock`"))?;
    let lock: Lock = serde_json::from_str(&text)?;
    let all = collect(root, dirs)?;
    let mut problems = vec![];
    for (key, fp) in &lock.tests {
        match all.get(key) {
            None => problems.push(format!("locked test removed or renamed: {key}")),
            Some((_, t)) if t.modifier.is_some() => problems.push(format!(
                "locked test disabled ({}): {key}",
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

    fn dirs() -> Vec<String> {
        LOCKED_DIRS.iter().map(|d| d.to_string()).collect()
    }

    #[test]
    fn rust_python_go_tests_are_fingerprinted() {
        let rs = "#[test]\nfn f001_ac1_shows_tasks() {\n    let b = render();\n    assert_eq!(line(&b, 2), \"Buy milk\");\n}\n\nfn helper() {}\n\n#[test]\n#[ignore]\nfn f001_ac2_empty() { assert!(empty()); }\n";
        let t = parse_file("tests/e2e/tui/list.rs", rs);
        assert_eq!(t.len(), 2, "{t:?}");
        assert_eq!(
            (
                t[0].title.as_str(),
                t[0].assertions,
                t[0].modifier.as_deref()
            ),
            ("f001_ac1_shows_tasks", 1, None)
        );
        assert_eq!(t[1].modifier.as_deref(), Some("ignore"));
        let weaker = rs.replace("assert_eq!(line(&b, 2), \"Buy milk\");", "assert!(true);");
        assert_ne!(
            parse_file("a.rs", rs)[0].fingerprint,
            parse_file("a.rs", &weaker)[0].fingerprint
        );

        let py = "@pytest.mark.skip\ndef test_f002_ac1_add():\n    assert app.rows() == ['x']\n\nasync def test_f002_ac2_blank(pilot):\n    await pilot.press('enter')\n    assert pilot.app.error == 'Title is required'\n";
        let t = parse_file("tests/e2e/tui/test_add.py", py);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].modifier.as_deref(), Some("skip"));
        assert_eq!(
            (t[1].title.as_str(), t[1].assertions),
            ("test_f002_ac2_blank", 1)
        );

        let go = "func TestF001AC1List(t *testing.T) {\n\trequire.Equal(t, 2, len(rows))\n}\nfunc TestF001AC2Empty(t *testing.T) {\n\tt.Skip(\"later\")\n}\n";
        let t = parse_file("tests/e2e/cli/list_test.go", go);
        assert_eq!(
            (t[0].assertions, t[1].modifier.as_deref()),
            (1, Some("Skip"))
        );
    }

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
        assert_eq!(lock(r, &dirs()).unwrap().0, 2);
        assert!(verify(r, &dirs()).unwrap().is_empty());
        fs::write(
            r.join("tests/e2e/f001.spec.ts"),
            ORIG.replace("toHaveCount(3)", "toHaveCount(2)"),
        )
        .unwrap();
        assert_eq!(verify(r, &dirs()).unwrap().len(), 1);
        fs::write(
            r.join("tests/e2e/f001.spec.ts"),
            ORIG.replace("test('F-001.AC2", "test.skip('F-001.AC2"),
        )
        .unwrap();
        assert!(verify(r, &dirs()).unwrap()[0].contains("disabled"));
    }
}
