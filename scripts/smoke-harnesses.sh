#!/usr/bin/env bash
# Smoke-test that each coding-agent CLI discovers the skills `sixloom init` installs.
# No model calls and no API keys: every check uses the tool's own offline discovery.
#
#   scripts/smoke-harnesses.sh [path/to/sixloom-binary]
#
# Needs: node/npm (codex, opencode, gemini), uv (kimi), claude (optional).
set -euo pipefail

BIN=${1:-target/debug/sixloom}
BIN=$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")
WORK=$(mktemp -d "${TMPDIR:-/tmp}/sixloom-smoke.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
export HOME="$WORK/home" CODEX_HOME="$WORK/home/.codex"
mkdir -p "$HOME/.gemini" "$CODEX_HOME"

echo "== installing agent CLIs into $WORK/clis"
(cd "$WORK" && mkdir clis && cd clis && npm i -s --no-fund --no-audit @openai/codex opencode-ai @google/gemini-cli >/dev/null)
NPX="$WORK/clis/node_modules/.bin"
command -v kimi >/dev/null || uv tool install -q kimi-cli
KIMI_PY="$(dirname "$(readlink -f "$(command -v kimi)")")/python"

P="$WORK/demo"
"$BIN" init "$P" --yes --agents claude,codex,opencode,gemini,kimi --frontend react-vite --docs fumadocs --design claude >/dev/null
EXPECT=$(find "$P/.agents/skills" -mindepth 2 -maxdepth 2 -name SKILL.md | wc -l | tr -d ' ')
echo "== sixloom installed $EXPECT skills into .agents/skills (and .claude/skills)"
cd "$P"
fail=0
check() { # name count
  if [ "$2" -ge "$EXPECT" ]; then echo "  ok   $1 discovers $2 skills"; else echo "  FAIL $1 discovers $2/$EXPECT"; fail=1; fi
}

n=$(timeout 120 "$NPX/codex" debug prompt-input hi 2>/dev/null | grep -oE '(sxl-[a-z-]+|sixloom|grilling|diagnosing-bugs|frontend-design|web-design-guidelines|playwright-cli|vercel-[a-z-]+|vite|vitest|shadcn-ui):' | sort -u | wc -l | tr -d ' ')
check codex "$n"

n=$(timeout 120 "$NPX/opencode" debug skill 2>/dev/null | grep -c '"location": "/' || true)
check opencode "$n"

echo "{\"$P\": \"TRUST_FOLDER\"}" > "$HOME/.gemini/trustedFolders.json"
n=$(timeout 120 "$NPX/gemini" skills list 2>/dev/null | grep -c '\[Enabled\]' || true)
check gemini "$n"

n=$("$KIMI_PY" - "$P" <<'EOF'
import asyncio, sys
from kaos.path import KaosPath
from kimi_cli.skill import resolve_skills_roots, discover_skills_from_roots
async def main():
    roots = await resolve_skills_roots(KaosPath.unsafe_from_local_path(sys.argv[1]))
    print(sum(1 for s in await discover_skills_from_roots(roots) if s.scope == "project"))
asyncio.run(main())
EOF
)
check kimi "$n"

if command -v claude >/dev/null; then
  claude plugin validate "$OLDPWD" >/dev/null && echo "  ok   claude plugin marketplace validates" || { echo "  FAIL claude plugin validate"; fail=1; }
  n=$(find "$P/.claude/skills" -mindepth 2 -maxdepth 2 -name SKILL.md | wc -l | tr -d ' ')
  check "claude (.claude/skills)" "$n"
fi
exit $fail
