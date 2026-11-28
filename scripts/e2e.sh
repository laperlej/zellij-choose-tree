#!/usr/bin/env bash
# e2e.sh: drive a real zellij session to test the plugin, with no human at the keyboard.
#
#   scripts/e2e.sh [worktree-dir]
#
# Builds the wasm from <worktree-dir> (default: the repo this script lives in), starts a
# throwaway zellij session, loads the plugin once per case with a generated layout, grants
# the permission prompt, then reads the rendered pane back with `zellij action dump-screen`
# and asserts on it. Exit status is 0 only if every case passes.
set -o errexit
set -o nounset
set -o pipefail

declare -r REPO="$(cd "${1:-$(dirname "${BASH_SOURCE[0]}")/..}" && pwd)"
declare -r SESSION="zct-e2e-$$"
declare -r WORK="$(mktemp -d)"
declare -r WASM="${REPO}/target/wasm32-wasip1/debug/zellij-choose-tree.wasm"

# Build through rustup when it is present: a distro or Homebrew rustc earlier in PATH may
# lack the wasm32-wasip1 std, which surfaces as a confusing "can't find crate for `core`".
if command -v rustup >/dev/null 2>&1; then
    cargo_build() { rustup run stable cargo build --target wasm32-wasip1; }
else
    cargo_build() { cargo build --target wasm32-wasip1; }
fi

fail_count=0

cleanup() {
    zellij delete-session --force "${SESSION}" >/dev/null 2>&1 || true
    rm -rf "${WORK}"
}
trap cleanup EXIT

# Wait until the pane contains $1, dumping repeatedly. Returns 1 on timeout.
await_screen() {
    local needle="$1" deadline=$((SECONDS + 15))
    while (( SECONDS < deadline )); do
        zellij -s "${SESSION}" action dump-screen "${WORK}/screen" >/dev/null 2>&1 || true
        grep -qF "${needle}" "${WORK}/screen" 2>/dev/null && return 0
        sleep 0.5
    done
    return 1
}

# load_plugin <kdl-config-body>
load_plugin() {
    local body="$1" layout="${WORK}/layout-$RANDOM.kdl"
    cat > "${layout}" <<EOF
layout {
    pane {
        plugin location="file:${WASM}" {
            ${body}
        }
    }
}
EOF
    zellij -s "${SESSION}" action new-tab --layout "${layout}"
    # The plugin requests five permissions; the prompt only appears the first time.
    if await_screen 'Allow? (y/n)'; then
        zellij -s "${SESSION}" action write-chars 'y'
    fi
    await_screen "(0) ${SESSION}" || { echo "    plugin never rendered" >&2; return 1; }
}

tab_lines() { grep -cE '^[[:space:]]+- \([0-9A-Z]+\) ' "${WORK}/screen" || true; }

# check <name> <kdl-body> <expect-tabs: yes|no>
check() {
    local name="$1" body="$2" expect="$3"
    printf '  %-34s' "${name}"
    if ! load_plugin "${body}"; then printf 'FAIL (no render)\n'; fail_count=$((fail_count+1)); return; fi
    local n; n="$(tab_lines)"
    if [[ "${expect}" == yes && "${n}" -gt 0 ]] || [[ "${expect}" == no && "${n}" -eq 0 ]]; then
        printf 'ok    (%s tab lines)\n' "${n}"
    else
        printf 'FAIL  (%s tab lines, expected %s)\n' "${n}" "${expect}"
        sed 's/^/      | /' "${WORK}/screen" >&2
        fail_count=$((fail_count+1))
    fi
}

echo "repo:    ${REPO}"
echo "building wasm32-wasip1 ..."
( cd "${REPO}" && cargo_build ) >/dev/null
[[ -f "${WASM}" ]] || { echo "no wasm artifact at ${WASM}" >&2; exit 1; }

echo "starting session ${SESSION} ..."
TERM=xterm-256color script -q /dev/null zellij --session "${SESSION}" >"${WORK}/boot.log" 2>&1 &
for _ in {1..30}; do zellij list-sessions 2>/dev/null | grep -q "${SESSION}" && break; sleep 0.5; done
zellij list-sessions 2>/dev/null | grep -q "${SESSION}" || { echo "session never started" >&2; exit 1; }

echo "cases:"
check "expanded true  -> tabs shown"  'expanded true'  yes
check "expanded false -> collapsed"   'expanded false' no
check "expanded absent-> collapsed"   ''               no

echo
if (( fail_count == 0 )); then echo "all cases passed"; else echo "${fail_count} case(s) failed"; fi
exit $(( fail_count > 0 ))
