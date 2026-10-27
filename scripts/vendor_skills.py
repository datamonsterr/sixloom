#!/usr/bin/env python3
"""Vendor upstream skills listed in packs/catalog.toml at their pinned revisions.

usage: python3 -I scripts/vendor_skills.py [--cache DIR] [--only NAME ...] [--check]

--check exits 1 if any vendored skill differs from its pinned upstream (used in CI).
Upstream content is copied verbatim; the only edits are the ones in a skill's `patch`
list, of which the supported kind is `rename <old> -> <new>` (frontmatter name).
"""

import argparse
import filecmp
import json
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
SKIP = {".git", "node_modules", "GENERATION.md"}
SKIP_SUFFIX = (".zip",)
LICENSE_RE = re.compile(r"^(LICENSE|LICENCE|COPYING|NOTICE)(\..*)?$", re.I)


def fetch(repo: str, rev: str, cache: pathlib.Path) -> pathlib.Path:
    dest = cache / f"{repo.replace('/', '__')}@{rev[:12]}"
    if dest.exists():
        return dest
    tmp = dest.with_suffix(".tmp")
    shutil.rmtree(tmp, ignore_errors=True)
    subprocess.run(["git", "init", "-q", str(tmp)], check=True)
    subprocess.run(["git", "-C", str(tmp), "fetch", "-q", "--depth", "1",
                    f"https://github.com/{repo}.git", rev], check=True)
    subprocess.run(["git", "-C", str(tmp), "checkout", "-q", "FETCH_HEAD"], check=True)
    tmp.rename(dest)
    return dest


def fetch_npm(spec: str, cache: pathlib.Path) -> pathlib.Path:
    dest = cache / ("npm__" + spec.replace("/", "__").replace("@", "_"))
    if dest.exists():
        return dest
    tmp = dest.with_suffix(".tmp")
    shutil.rmtree(tmp, ignore_errors=True)
    tmp.mkdir(parents=True)
    out = subprocess.run(["npm", "pack", spec, "--silent", "--pack-destination", str(tmp)],
                         check=True, capture_output=True, text=True).stdout.strip().splitlines()[-1]
    subprocess.run(["tar", "xzf", str(tmp / out), "-C", str(tmp)], check=True)
    (tmp / out).unlink()
    (tmp / "package").rename(dest)
    shutil.rmtree(tmp)
    return dest


def ignore(_dir: str, names: list[str]) -> set[str]:
    return {n for n in names if n in SKIP or n.endswith(SKIP_SUFFIX)}


def apply_patches(skill_dir: pathlib.Path, patches: list[str]) -> None:
    for p in patches:
        m = re.match(r"rename (\S+) -> (\S+)", p)
        if not m:
            continue
        md = skill_dir / "SKILL.md"
        text = md.read_text()
        text = re.sub(rf"^name:\s*['\"]?{re.escape(m[1])}['\"]?\s*$",
                      f"name: {m[2]}", text, count=1, flags=re.M)
        md.write_text(text)


def build(skill: dict, cache: pathlib.Path, out: pathlib.Path) -> None:
    up = skill["upstream"]
    if "npm" in up:
        repo_dir = fetch_npm(up["npm"], cache)
    else:
        repo_dir = fetch(up["repo"], up["rev"], cache)
    src = repo_dir / up["path"]
    shutil.copytree(src, out, ignore=ignore)
    if not any(LICENSE_RE.match(p.name) for p in out.iterdir()):
        lic = next((p for p in sorted(repo_dir.iterdir()) if LICENSE_RE.match(p.name)), None)
        if lic:
            shutil.copy2(lic, out / "LICENSE")
        else:
            (out / "LICENSE").write_text(
                f"{up['license']} (declared by upstream {origin(up)}; no license file in repository)\n")
    apply_patches(out, skill.get("patch", []))
    (out / ".upstream.json").write_text(json.dumps(
        {"source": origin(up), "path": up["path"],
         "license": up["license"], "patch": skill.get("patch", [])}, indent=2) + "\n")


def origin(up: dict) -> str:
    return f"npm:{up['npm']}" if "npm" in up else f"github:{up['repo']}@{up['rev']}"


def same_tree(a: pathlib.Path, b: pathlib.Path) -> bool:
    cmp = filecmp.dircmp(a, b)
    if cmp.left_only or cmp.right_only or cmp.diff_files or cmp.funny_files:
        return False
    return all(same_tree(a / d, b / d) for d in cmp.common_dirs)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cache", type=pathlib.Path,
                    default=pathlib.Path(tempfile.gettempdir()) / "sixloom-vendor")
    ap.add_argument("--only", nargs="*")
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    args.cache.mkdir(parents=True, exist_ok=True)

    catalog = tomllib.loads((ROOT / "packs/catalog.toml").read_text())
    drift = []
    for skill in catalog["skill"]:
        if "upstream" not in skill or (args.only and skill["name"] not in args.only):
            continue
        target = ROOT / "packs" / skill["pack"] / "skills" / skill["name"]
        with tempfile.TemporaryDirectory() as tmp:
            staged = pathlib.Path(tmp) / skill["name"]
            build(skill, args.cache, staged)
            if args.check:
                if not target.exists() or not same_tree(staged, target):
                    drift.append(skill["name"])
                continue
            shutil.rmtree(target, ignore_errors=True)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copytree(staged, target)
            print(f"vendored {skill['pack']}/{skill['name']} <- {origin(skill['upstream'])}")
    if drift:
        print("drift from pinned upstream: " + ", ".join(drift), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
