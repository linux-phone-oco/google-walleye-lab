#!/bin/sh
# Proves item walleye-tests-never-reach-the-authors-notify (lab-tools decision 2026-11-28-a-test-never-reaches-the-authors-notify-file):
# a check or dry run through bin/attended never reads the author's notify file, and the real run is unchanged. A stub lpoco-attended in
# the runner's cache prints the notify path it was given; a stand-in notify file sits in a stand-in HOME and must not be the one named.
#   tests-never-notify-selftest.sh      Prints `pass|fail|skip walleye-tests-never-notify DETAIL`; exit 0 only on pass.
HERE=$(cd "$(dirname "$0")/../.." && pwd)
say() { echo "$1 walleye-tests-never-notify $2"; [ "$1" = pass ]; exit $?; }
command -v cargo >/dev/null && command -v jq >/dev/null || say skip "cargo or jq not installed: not known yet"
cd "$HERE" || say fail "no repository"
REV=$(cargo metadata --locked --format-version 1 --manifest-path procedures/steps/Cargo.toml | jq -er '.packages[] | select(.name == "lpoco-attended") | .source | sub(".*#"; "")') || say fail "no locked revision"
export CARGO_HOME=${CARGO_HOME:-$HOME/.cargo} RUSTUP_HOME=${RUSTUP_HOME:-$HOME/.rustup}  # the stand-in HOME below must not hide the toolchain
T=$(mktemp -d) || say fail "no temp dir"
trap 'rm -rf "$T"' EXIT
mkdir -p "$T/home/.config/lpoco-attended" "$T/root/$REV/bin"
printf 'server https://ntfy.invalid\ntopic stand-in\nreply_topic stand-in-reply\n' > "$T/home/.config/lpoco-attended/notify"
printf '#!/bin/sh\necho "notify=${LPOCO_ATTENDED_NOTIFY:-unset}"\n' > "$T/root/$REV/bin/lpoco-attended"
chmod +x "$T/root/$REV/bin/lpoco-attended"
run() { HOME="$T/home" ATTENDED_ROOT="$T/root" bin/attended "$@" 2>/dev/null | head -n1; }
case $(run check) in notify=/nonexistent/*) ;; *) say fail "check reads a notify file: $(run check)" ;; esac
case $(run run x --dry-run) in notify=/nonexistent/*) ;; *) say fail "a dry run reads a notify file: $(run run x --dry-run)" ;; esac
case $(LPOCO_ATTENDED_NOTIFY="$T/home/.config/lpoco-attended/notify" run check) in notify=/nonexistent/*) ;; *) say fail "check let a given notify file through" ;; esac
real=$(env -u LPOCO_ATTENDED_NOTIFY HOME="$T/home" ATTENDED_ROOT="$T/root" bin/attended run x 2>/dev/null | head -n1)
[ "$real" = "notify=unset" ] || say fail "the real run was changed: $real"
grep -q 'LPOCO_ATTENDED_NOTIFY' procedures/steps/.cargo/config.toml 2>/dev/null || say fail "procedures/steps/.cargo/config.toml does not pin LPOCO_ATTENDED_NOTIFY"
grep -q 'LPOCO_ATTENDED_NOTIFY' .github/workflows/check.yml || say fail "the CI workflow does not pin LPOCO_ATTENDED_NOTIFY"
say pass "check and dry run name no notify file, the real run is unchanged, cargo test and CI pin it"
