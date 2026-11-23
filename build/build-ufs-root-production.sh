#!/bin/sh
# The stage 2 PRODUCTION image from a live list's int/ tag (backlog walleye-stage2-production-image, walleye-next-image-from-an-int-tag): the kernel is built from a checkout of exactly the tag, never from a tree by hand.
#   build-ufs-root-production.sh TAG OUT_DIR
# TAG is a pushed int/walleye/ufs-on/v7.3/N (rhizoid assemble ufs-on --lists kernel/topics.toml --repo CLONE makes or reuses it); OUT_DIR is named NOT-TO-BOOT-... . Steps: the tag is on the fork's origin and in the clone; a detached checkout of it
# in $AGENT_BUILD; build/build-kernel.sh on it (the diagnostic and UFS-root fragments, as for every UFS-on image); build/build-ufs-root-image.sh with BOOTED_SLOT=yes PRODUCTION_INIT=yes KERNEL_TAG=TAG (its checks decide; it writes nothing when one fails);
# the checkout is removed. Environment: KERNEL_CLONE (default ~/Projects/linux-phone-oco/linux), WALLEYE_BUILD (default ~/Projects/walleye-build: the matched image, busybox-arm64, dtbo.img), ROOT_PART (default system_a: Android is on slot b).
set -eu
TAG=${1:?usage: build-ufs-root-production.sh TAG OUT_DIR}
OUT=${2:?usage: build-ufs-root-production.sh TAG OUT_DIR}
HERE=$(cd "$(dirname "$0")/.." && pwd)
CLONE=${KERNEL_CLONE:-$HOME/Projects/linux-phone-oco/linux}
WB=${WALLEYE_BUILD:-$HOME/Projects/walleye-build}
BUILD=${AGENT_BUILD:?AGENT_BUILD is not set: run from a session started by agent-env}
case $TAG in int/walleye/ufs-on/*) ;; *) echo "build-ufs-root-production: $TAG is not an int/walleye/ufs-on/... tag (the production image has UFS on)" >&2; exit 1 ;; esac
REMOTE=$(git -C "$CLONE" ls-remote origin "refs/tags/$TAG^{}" | cut -f1)
git -C "$CLONE" rev-parse --verify -q "$TAG^{commit}" >/dev/null || { echo "build-ufs-root-production: the clone $CLONE has no $TAG" >&2; exit 1; }
[ "$REMOTE" = "$(git -C "$CLONE" rev-parse "$TAG^{commit}")" ] || { echo "build-ufs-root-production: origin's $TAG is [$REMOTE], the clone's is not that commit (a moved or unpushed tag)" >&2; exit 1; }
MATCHED=$(ls "$WB"/NOT-TO-BOOT-ufs-on-images-ufs-match/NOT-TO-BOOT-ufs-on-ufs-match.img)
N=$(printf '%s' "$TAG" | tr '/' '-')
TREE=$BUILD/walleye-production-tree-$N
KOUT=$BUILD/walleye-production-kernel-$N
[ ! -e "$TREE" ] || { echo "build-ufs-root-production: $TREE exists (a build of this tag is running or was cut short): git -C $CLONE worktree remove --force $TREE" >&2; exit 1; }
git -C "$CLONE" worktree add --detach "$TREE" "$TAG" >/dev/null
trap 'git -C "$CLONE" worktree remove --force "$TREE"' EXIT
mkdir -p "$KOUT"
EXTRA_CONFIG="$HERE/build/walleye-diag.config $HERE/build/walleye-ufs-root.config" "$HERE/build/build-kernel.sh" "$TREE" "$KOUT"
BOOTED_SLOT=yes PRODUCTION_INIT=yes KERNEL_TAG=$TAG ROOT_PART=${ROOT_PART:-system_a} DTBO=${DTBO:-$WB/control/dtbo.img} \
  "$HERE/build/build-ufs-root-image.sh" "$MATCHED" "$TREE" "$KOUT" "$WB" "$OUT"
