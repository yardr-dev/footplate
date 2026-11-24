# Arch packaging

Two packages for the AUR, each a directory with a `PKGBUILD`:

- `footplate-bin/`: the signed Linux release from yardr.dev, installed as it is.
- `footplate-git/`: built from the newest commit on GitHub.

## Install here

In either directory:

    makepkg -si

## Versions

`footplate-bin` pins a release and its checksums. `./update` reads
`https://yardr.dev/releases/footplate/latest.json`, checks it against that
release's `checksums.txt`, verifies the minisign signature of that `checksums.txt` against the release key (it refuses without a good one, and needs `minisign` installed), rewrites `pkgver`, the checksums (the polkit action's too, which comes from the tag's repository) and `.SRCINFO`, and
does nothing when already current. Check the result with `makepkg -si`.

`footplate-git` needs no bump: `pkgver()` reads `git describe` at build time, so
any commit gives a new version. Its `.SRCINFO` carries the version of the last
build and may lag; the AUR only needs the files to be valid.

## Publishing

`.github/workflows/aur.yml` runs on a `v*` tag (prereleases, with a `-`, are
skipped) and by hand. On a tag it waits for `latest.json` to name that tag,
about ten minutes at most; run it by hand after `scripts/release.sh` has
published if that was missed. It runs `footplate-bin/update`, checks the
checksums (`makepkg --verifysource`) and that `makepkg --printsrcinfo` equals
`.SRCINFO`, then pushes both packages to the AUR if their files differ from the
AUR's. It refuses to push a `footplate-bin` older than the AUR's: an older
release is signed too, so a `latest.json` that names one would pass `update`.
`scripts/test-aur.sh` runs the push step against local repositories.

The workflow does not commit the bumped `PKGBUILD` back to this repository: the
AUR's copy is the current one, and `./update` in a checkout brings this one up.

### AUR setup

For whoever holds the AUR account: add the account's SSH key to the AUR, and
the private half as the secret `AUR_SSH_PRIVATE_KEY` of a GitHub environment
named `aur` (not a repository secret), whose deployment branches and tags are
limited to `main` and `v*`: the job runs in that environment, so the key
reaches no other ref. Without the secret the workflow validates and ends green
without pushing. The AUR's host keys are `aur_known_hosts`, checked against
the fingerprints on https://aur.archlinux.org; the push trusts no others.

A package not yet on the AUR needs nothing by hand: the workflow's clone of a
new name is an empty repository, and its first push creates the package.
