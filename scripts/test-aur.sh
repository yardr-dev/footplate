#!/bin/sh
# The AUR workflow's push step, run as written in .github/workflows/aur.yml
# against local repositories in place of the AUR's: footplate-bin is pushed
# over an older AUR version and into a new, empty one, and an AUR version
# newer than the one here is refused and left as it was (an older release's
# checksums.txt is signed too, so a latest.json naming one passes update).
# Needs vercmp, which pacman brings: the workflow runs on Arch.
set -eu

root=$(CDPATH= cd "$(dirname "$0")/.." && pwd)
if ! command -v vercmp >/dev/null 2>&1; then
    echo "test-aur: skipped, needs vercmp (pacman)"
    exit 0
fi
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# The step's script, the block under its `run: |`, out of its indent.
awk '
/- name: Push to the AUR/ { step = 1 }
step && /^ *run: \|$/ { run = 1; next }
run {
    if ($0 ~ /^[[:space:]]*$/) { print ""; next }
    match($0, /^ */)
    if (!ind) ind = RLENGTH
    if (RLENGTH < ind) exit
    print substr($0, ind + 1)
}' "$root/.github/workflows/aur.yml" \
    | sed "s|ssh://aur@aur.archlinux.org/|file://$tmp/aur/|" > "$tmp/step.sh"
grep -q 'file://' "$tmp/step.sh" || { echo "test-aur: no push step found" >&2; exit 1; }

# Runs the step against an AUR whose footplate-bin is at $1 (none: empty).
run() {
    rm -rf "$tmp/aur" "$tmp/repo" "$tmp/home"
    mkdir -p "$tmp/aur" "$tmp/home"
    for pkg in footplate-bin footplate-git; do
        mkdir -p "$tmp/repo/packaging/arch/$pkg"
        cp "$root/packaging/arch/$pkg/PKGBUILD" "$root/packaging/arch/$pkg/.SRCINFO" "$tmp/repo/packaging/arch/$pkg/"
        git init -q --bare "$tmp/aur/$pkg.git"
        [ "$pkg" = footplate-bin ] && [ "$1" = none ] && continue
        w=$tmp/seed-$pkg
        rm -rf "$w"
        git init -q "$w"
        cp "$root/packaging/arch/$pkg/PKGBUILD" "$root/packaging/arch/$pkg/.SRCINFO" "$w/"
        if [ "$pkg" = footplate-bin ]; then
            sed -i.bak "s/pkgver = .*/pkgver = $1/" "$w/.SRCINFO" && rm "$w/.SRCINFO.bak"
        fi
        git -C "$w" add PKGBUILD .SRCINFO
        git -C "$w" -c user.name=t -c user.email=t@t commit -qm seed
        git -C "$w" push -q "$tmp/aur/$pkg.git" HEAD:master
    done
    (cd "$tmp/repo" && env -i HOME="$tmp/home" PATH="$PATH" AUR_SSH_PRIVATE_KEY=stand-in \
        bash -eo pipefail "$tmp/step.sh") >"$tmp/out" 2>&1
}
aur_ver() {
    git --git-dir="$tmp/aur/footplate-bin.git" show master:.SRCINFO | awk '$1 == "pkgver" { print $3 }'
}
here=$(awk '$1 == "pkgver" { print $3 }' "$root/packaging/arch/footplate-bin/.SRCINFO")

run 0.0.1 || { cat "$tmp/out"; echo "test-aur: push over an older AUR failed" >&2; exit 1; }
[ "$(aur_ver)" = "$here" ] || { echo "test-aur: older AUR not brought to $here" >&2; exit 1; }

run none || { cat "$tmp/out"; echo "test-aur: push to a new AUR package failed" >&2; exit 1; }
[ "$(aur_ver)" = "$here" ] || { echo "test-aur: new AUR package not at $here" >&2; exit 1; }

if run 999.0.0; then
    cat "$tmp/out"
    echo "test-aur: pushed $here over the AUR's newer 999.0.0" >&2
    exit 1
fi
grep -q 'refusing to push it back' "$tmp/out" || { cat "$tmp/out"; echo "test-aur: refused, but not for the version" >&2; exit 1; }
[ "$(aur_ver)" = 999.0.0 ] || { echo "test-aur: the AUR's newer version was changed" >&2; exit 1; }

echo "test-aur: ok"
