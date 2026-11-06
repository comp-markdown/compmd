#!/bin/bash
# In the container of docker/Dockerfile: checks out a commit of the repository mounted at /git into /work, with the
# markdown-rs submodule, installs the packages, and runs a command there, `cargo make ci` by default (docs/ci.md,
# decision 12).
#
# Usage: compmd-linux [--ref <commit>] [site | command...]
#   --ref <commit>  the commit to check out: a hash, tag or branch of the mounted repository; HEAD by default
#   site            the site's build and check, as the Pages workflow runs them
set -euo pipefail

ref=HEAD
if [ "${1:-}" = "--ref" ]; then
  ref="$2"
  shift 2
fi

# The mounted repository belongs to the host's user. As a submodule's or worktree's git folder, its config may name a
# work tree on the host, so it is read with one that exists here.
git config --global --add safe.directory '*'
host() { git --git-dir=/git --work-tree=/ "$@"; }

# Checks out `commit` in `folder` from the objects of the repository `source`, without copying them.
checkout() {
  local source="$1" folder="$2" commit="$3"
  git init -q "$folder"
  echo "$source/objects" > "$folder/.git/objects/info/alternates"
  git -C "$folder" -c advice.detachedHead=false checkout -q --force "$commit"
}

commit="$(host rev-parse "$ref^{commit}")"
checkout /git /work "$commit"
# The site's build reads the spec's released versions from their tags; their objects come with the alternates.
host for-each-ref --format='create %(refname) %(objectname)' refs/tags | git -C /work update-ref --stdin
checkout /git/modules/markdown-rs /work/markdown-rs "$(git -C /work rev-parse "HEAD:markdown-rs")"
echo "compmd-linux: $(git -C /work log -1 --format='%h %s')"

cd /work
pnpm install --frozen-lockfile --store-dir /cache/pnpm
if [ "$#" -eq 0 ]; then
  # CI's browser tests run in Chrome and in Playwright's WebKit.
  pnpm --dir sandbox exec playwright install --with-deps webkit
  set -- cargo make ci
elif [ "$1" = site ]; then
  # .github/workflows/pages.yml: the timing is measured on a contributor's machine.
  pnpm -r build
  set -- pnpm --dir website check --no-timing
fi
exec "$@"
