#!/usr/bin/env bash
# Smoke-test that each coding-agent CLI discovers the skills `sixloom init` installs.
# No model calls and no API keys: every check uses the tool's own offline discovery.
#
#   scripts/smoke-harnesses.sh [path/to/sixloom-binary]
#
# Needs: node/npm (codex, opencode, gemini), uv (kimi), claude (optional).
set -euo pipefail

# Pinned agent CLI versions, so a fresh upstream release cannot break (or silently pass) the run.
# Bump deliberately and re-run; override with e.g. CODEX_VERSION=latest.
CODEX_VERSION=${CODEX_VERSION:-0.164.0}
OPENCODE_VERSION=${OPENCODE_VERSION:-1.19.4}
GEMINI_VERSION=${GEMINI_VERSION:-0.65.0}
KIMI_VERSION=${KIMI_VERSION:-1.52.0}

BIN=${1:-target/debug/sixloom}
BIN=$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")
WORK=$(mktemp -d "${TMPDIR:-/tmp}/sixloom-smoke.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
export HOME="$WORK/home" CODEX_HOME="$WORK/home/.codex"
mkdir -p "$HOME/.gemini" "$CODEX_HOME"

echo "== installing agent CLIs into $WORK/clis"
(cd "$WORK" && mkdir clis && cd clis && npm i -s --no-fund --no-audit \
  "@openai/codex@$CODEX_VERSION" "opencode-ai@$OPENCODE_VERSION" "@google/gemini-cli@$GEMINI_VERSION" >/dev/null)
NPX="$WORK/clis/node_modules/.bin"
# HOME is overridden above, so resolve kimi through uv's tool dir rather than PATH.
KIMI_PY="$(uv tool dir)/kimi-cli/bin/python"
[ -x "$KIMI_PY" ] || uv tool install -q "kimi-cli==$KIMI_VERSION"
[ -x "$KIMI_PY" ] || { echo "kimi-cli python not found at $KIMI_PY"; exit 1; }

P="$WORK/demo"
"$BIN" init "$P" --yes --agents claude,codex,opencode,gemini,kimi --frontend react-vite --docs fumadocs --design claude >/dev/null
EXPECT=$(find "$P/.agents/skills" -mindepth 2 -maxdepth 2 -name SKILL.md | wc -l | tr -d ' ')
echo "== sixloom installed $EXPECT skills into .agents/skills (and .claude/skills)"
cd "$P"
fail=0
check() { # name count
  if [ "$2" -ge "$EXPECT" ]; then echo "  ok   $1 discovers $2 skills"; else echo "  FAIL $1 discovers $2/$EXPECT"; fail=1; fi
}

# Each CLI writes to a file first: a CLI that fails to start is reported, not silently skipped.
run() { # name file cmd...
  local name=$1 out=$2; shift 2
  if ! timeout 120 "$@" > "$out" 2> "$out.err"; then
    echo "  FAIL $name did not run: $(grep -v WARNING "$out.err" | grep -m1 -E '^[A-Za-z]*Error:|^error|fatal' || tail -1 "$out.err")"
    fail=1
    return 1
  fi
}

if run codex "$WORK/codex.out" "$NPX/codex" debug prompt-input hi; then
  n=$(grep -oE '(sxl-[a-z-]+|sixloom|grilling|diagnosing-bugs|frontend-design|web-design-guidelines|playwright-cli|vercel-[a-z-]+|vite|vitest|shadcn-ui):' "$WORK/codex.out" | sort -u | wc -l | tr -d ' ')
  check codex "$n"
fi

# opencode truncates large output written to a pipe, so it also goes through a file.
if run opencode "$WORK/opencode.out" "$NPX/opencode" debug skill; then
  n=$(grep -oE '"name": "[^"]+"' "$WORK/opencode.out" | grep -vc customize-opencode || true)
  check opencode "$n"
fi

echo "{\"$P\": \"TRUST_FOLDER\"}" > "$HOME/.gemini/trustedFolders.json"
if run gemini "$WORK/gemini.out" "$NPX/gemini" skills list; then
  n=$(grep -c '\[Enabled\]' "$WORK/gemini.out" || true)
  check gemini "$n"
fi

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
