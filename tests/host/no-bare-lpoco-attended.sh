#!/bin/sh
# Proves item walleye-procedures-run-own-cli: no procedure, script, workflow or keeper brief of this repository calls a bare `lpoco-attended` (or `lpoco attended`) to run, check or make a procedure; they call bin/attended,
# which runs the lpoco-attended its own steps crate is locked to. Then one procedure walks its steps in a dry run through bin/attended.
#   no-bare-lpoco-attended.sh [--no-run]     (--no-run: only the search, no build)
# Prints `pass|fail|skip walleye-no-bare-lpoco-attended DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-no-bare-lpoco-attended $2"; [ "$1" = pass ]; exit $?; }
cd "$HERE" || say fail "no repository"
bare=$(grep -rnIE '(^|[^/a-z-])(lpoco-attended|lpoco attended) +(run|check|new)( |$|`)' procedures bin build tests .github AGENTS.md README.md 2>/dev/null | grep -v -e 'tests/host/no-bare-lpoco-attended.sh' | head -n5 | tr '\n' '|')
[ -z "$bare" ] || say fail "a bare lpoco-attended call: $bare"
[ "${1:-}" != --no-run ] || say pass "no bare call (the dry run was not asked for)"
[ -x bin/attended ] || say fail "no bin/attended"
command -v cargo >/dev/null && command -v jq >/dev/null && command -v rust-script >/dev/null || say skip "cargo, jq or rust-script not installed: not known yet"
out=$(bin/attended check --help 2>&1) || out=$(bin/attended 2>&1)
case $out in *lpoco*attended*|*usage*|*Usage*|*check*) ;; *) say fail "bin/attended did not run the CLI: $(echo "$out" | head -n2 | tr '\n' '|')" ;; esac
say pass "no bare call, and bin/attended runs the CLI locked at $(cargo metadata --locked --format-version 1 --manifest-path tools/walleye-tries/Cargo.toml | jq -r '.packages[] | select(.name == "lpoco-attended") | .version')"
