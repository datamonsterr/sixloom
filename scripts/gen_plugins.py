#!/usr/bin/env python3
"""Generate Claude Code and Codex plugin manifests from packs/catalog.toml.

usage: python3 -I scripts/gen_plugins.py [--check]

Claude Code: one marketplace (.claude-plugin/marketplace.json) whose plugins are
*bundles*: the core plugin plus one plugin per front-end, docs-engine and design
choice, so a team installs only the variants it uses.
Codex: one plugin per pack (packs/<pack>/.codex-plugin/plugin.json) listed in
.agents/plugins/marketplace.json.
--check exits 1 if any generated file is out of date (CI).
"""

import json
import pathlib
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
VERSION = tomllib.loads((ROOT / "cli/Cargo.toml").read_text())["package"]["version"]
AUTHOR = {"name": "datamonsterr", "url": "https://github.com/datamonsterr"}
REPO = "https://github.com/datamonsterr/sixloom"

# Claude bundles: plugin name -> (description, `when` conditions it covers).
BUNDLES = {
    "sixloom": ("The sixloom loom workflow: spec, UI design, preview, data model, system design, API docs, locked test suites, autonomous TDD build, patch and validate.",
                ["always"]),
    "sixloom-web": ("sixloom skills for any web or desktop interface: visual design direction, web interface guidelines, Playwright browser testing.", ["kind:web"]),
    "sixloom-expo": ("sixloom front-end skills for Expo / React Native mobile interfaces (Vercel React Native guide).", ["frontend:expo"]),
    "sixloom-react": ("sixloom front-end skills for React + Vite (Vercel React guides, Vite, Vitest).", ["frontend:react-vite"]),
    "sixloom-nextjs": ("sixloom front-end skills for Next.js (Vercel React and composition guides).", ["frontend:nextjs"]),
    "sixloom-vue": ("sixloom front-end skills for Vue 3 + Vite (antfu's Vue, Vite and Vitest skills).", ["frontend:vue"]),
    "sixloom-nuxt": ("sixloom front-end skills for Nuxt (antfu's Vue, Nuxt and Vitest skills).", ["frontend:nuxt"]),
    "sixloom-svelte": ("sixloom front-end skills for SvelteKit (official Svelte skills).", ["frontend:sveltekit"]),
    "sixloom-shadcn": ("shadcn/ui component guidance for sixloom React front ends.", ["ui:shadcn"]),
    "sixloom-fumadocs": ("sixloom docs site with Fumadocs: MDX, Mermaid and a generated OpenAPI reference.", ["docs:fumadocs"]),
    "sixloom-docusaurus": ("sixloom docs site with Docusaurus: MDX, Mermaid and a generated OpenAPI reference.", ["docs:docusaurus"]),
    "sixloom-starlight": ("sixloom docs site with Astro Starlight: MDX, Mermaid and a generated OpenAPI reference.", ["docs:starlight"]),
    "sixloom-stitch": ("Google Stitch design workflow for the sixloom UI design loom (official stitch CLI skill).", ["design:stitch"]),
}


def conds(skill: dict) -> list[str]:
    w = skill.get("when", [])
    return [w] if isinstance(w, str) else w


def render() -> dict[pathlib.Path, str]:
    cat = tomllib.loads((ROOT / "packs/catalog.toml").read_text())
    out: dict[pathlib.Path, str] = {}
    plugins = []
    for name, (desc, wanted) in BUNDLES.items():
        skills = [f"./packs/{s['pack']}/skills/{s['name']}" for s in cat["skill"]
                  if set(conds(s)) & set(wanted)]
        plugins.append({
            "name": name,
            "source": "./",
            "strict": False,
            "description": desc,
            "version": VERSION,
            "author": AUTHOR,
            "homepage": REPO,
            "license": "MIT",
            "category": "development",
            "keywords": ["sixloom", "spec-driven", "tdd", "agents"],
            "skills": skills,
        })
    out[ROOT / ".claude-plugin/marketplace.json"] = json.dumps({
        "name": "sixloom",
        "owner": AUTHOR,
        "metadata": {"description": "sixloom: spec → preview → API docs → autonomous TDD build, as skills for coding agents.",
                     "version": VERSION},
        "plugins": plugins,
    }, indent=2) + "\n"

    codex = []
    for pack in cat["pack"]:
        pid = f"sixloom-{pack['id']}" if pack["id"] != "core" else "sixloom"
        out[ROOT / f"packs/{pack['id']}/.codex-plugin/plugin.json"] = json.dumps({
            "name": pid,
            "version": VERSION,
            "description": pack["description"],
            "author": AUTHOR,
            "homepage": REPO,
            "repository": REPO,
            "license": "MIT",
            "keywords": ["sixloom", pack["id"]],
            "skills": "./skills/",
            "interface": {
                "displayName": pack["title"] if pack["id"] != "core" else "sixloom",
                "shortDescription": pack["description"][:80],
                "developerName": "datamonsterr",
                "category": "Coding",
                "capabilities": ["Interactive", "Write"],
                "websiteURL": REPO,
                "defaultPrompt": ["sixloom init", "sixloom status", "sixloom next"] if pack["id"] == "core" else [],
            },
        }, indent=2) + "\n"
        codex.append({
            "name": pid,
            "source": {"source": "local", "path": f"./packs/{pack['id']}"},
            "policy": {"installation": "AVAILABLE", "authentication": "ON_INSTALL"},
            "category": "Coding",
        })
    out[ROOT / "docs/SKILLS.md"] = skills_doc(cat)
    out[ROOT / ".agents/plugins/marketplace.json"] = json.dumps({
        "name": "sixloom",
        "interface": {"displayName": "sixloom"},
        "plugins": codex,
    }, indent=2) + "\n"
    return out


def skills_doc(cat: dict) -> str:
    def desc(s: dict) -> str:
        md = (ROOT / f"packs/{s['pack']}/skills/{s['name']}/SKILL.md")
        if not md.exists():
            return ""
        text = md.read_text().split("---")[1]
        lines = text.splitlines()
        for i, line in enumerate(lines):
            if line.startswith("description:"):
                v = line.split(":", 1)[1].strip()
                if v in (">-", ">", "|", "|-"):
                    v = " ".join(x.strip() for x in lines[i + 1:] if x.startswith("  "))
                return v.strip("'\"").split(". ")[0].rstrip(".") + "."
        return ""

    def src(s: dict) -> str:
        if "upstream" in s:
            u = s["upstream"]
            if "npm" in u:
                return f"[`{u['npm']}`](https://www.npmjs.com/package/{u['npm'].rsplit('@', 1)[0]}) · {u['license']}"
            return f"[{u['repo']}](https://github.com/{u['repo']}/tree/{u['rev']}/{u['path']}) · {u['license']}"
        if "ported_from" in s:
            u = s["ported_from"]
            return f"sixloom, ported from [{u['repo']}](https://github.com/{u['repo']}/tree/{u['rev']}/{u['path']}) · {u['license']}"
        return "sixloom · MIT"

    def when(s: dict) -> str:
        c = conds(s)
        return ", ".join(f"`{x}`" for x in c) if c else "bank only"

    lines = ["# Skill packs", "",
             "<!-- Generated by scripts/gen_plugins.py from packs/catalog.toml. Do not edit. -->", "",
             "Pack IDs are also [other-skill](https://github.com/datamonsterr/other-skill) category IDs. "
             "`Installed when` refers to the choices in `sixloom.toml`; the rest go to the bank with `sixloom init --bank`.",
             "Upstream skills are vendored verbatim at a pinned revision (`scripts/vendor_skills.py`) with their license; "
             "the only edits are listed in the catalog's `patch` field.", ""]
    for pack in cat["pack"]:
        lines += [f"## {pack['title']} (`{pack['id']}`)", "", pack["description"] + ".", "",
                  "| Skill | What it does | Installed when | Source |", "|---|---|---|---|"]
        for s in cat["skill"]:
            if s["pack"] == pack["id"]:
                lines.append(f"| `{s['name']}` | {desc(s)} | {when(s)} | {src(s)} |")
        lines.append("")
    return "\n".join(lines)


def main() -> int:
    check = "--check" in sys.argv
    stale = []
    for path, text in render().items():
        if check:
            if not path.exists() or path.read_text() != text:
                stale.append(str(path.relative_to(ROOT)))
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        print(f"wrote {path.relative_to(ROOT)}")
    if stale:
        print("out of date (run scripts/gen_plugins.py): " + ", ".join(stale), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
