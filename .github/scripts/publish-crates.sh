#!/usr/bin/env bash
# Publishes the crates of a release (RELEASING.md, section 3): compmd-markdown first, since compmd depends on it by
# version, then compmd, then tauri-plugin-compmd. A version crates.io has already is left as it is: the fork's version moves only when its code
# changes, and a release that failed halfway can run again.
#
#   .github/scripts/publish-crates.sh            # publishes, with CARGO_REGISTRY_TOKEN set
#   .github/scripts/publish-crates.sh --dry-run  # says what it would publish, and publishes nothing
set -euo pipefail
# The repository's root, wherever it is run from.
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

dry_run=false
[ "${1:-}" = "--dry-run" ] && dry_run=true

# tauri-plugin-compmd's package holds the frames it embeds, which the release job builds (`cargo make tauri-frames`):
# without them, nothing is published.
if [ ! -f tauri/frames/frames.json ]; then
  echo "::error::tauri/frames/ has no build: run cargo make tauri-frames first"
  exit 1
fi

# The version of the package `$1` in the manifest `$2`, read without a lock file: the fork, `markdown-rs/`, is a
# workspace of its own, whose checkout has none, and `cargo pkgid` needs one.
version_of() {
  cargo metadata --no-deps --format-version 1 --manifest-path "$2" | jq -r --arg name "$1" '.packages[] | select(.name == $name) | .version'
}

# Publishes the package `$1` of the manifest `$2`, with `cargo publish`'s further arguments after them.
publish() {
  local version
  version="$(version_of "$1" "$2")"
  if [ -z "$version" ]; then
    echo "::error::no version for $1 in $2"
    exit 1
  fi
  if curl -sf -A "comp-markdown/compmd release workflow" "https://crates.io/api/v1/crates/$1/$version" > /dev/null; then
    echo "$1@$version is on crates.io already"
  elif [ "$dry_run" = true ]; then
    echo "$1@$version would be published"
  else
    cargo publish --manifest-path "$2" -p "$1" "${@:3}"
  fi
}

publish compmd-markdown markdown-rs/Cargo.toml
publish compmd Cargo.toml
# Not built again here: building Tauri on Linux needs WebKitGTK, and the Tauri check built and tested this commit
# before the release went on (release.yml).
publish tauri-plugin-compmd tauri/Cargo.toml --no-verify
