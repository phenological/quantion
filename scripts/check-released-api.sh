#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

version="$(sed -n 's/^version = "\(.*\)"/\1/p' core/Cargo.toml | head -1)"
[ -n "$version" ] || { echo "no version in core/Cargo.toml"; exit 1; }
tag="v$version"

if ! git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null; then
  echo "$tag is not released yet; the wrappers will download it once it is"
  exit 0
fi

git fetch --quiet --depth=1 origin "refs/tags/$tag:refs/tags/$tag"
if ! git diff --quiet "$tag" -- core/include/quantion.h; then
  echo "core/include/quantion.h changed since $tag, but the version is still $version"
  echo "wrappers installed without cargo would download $tag and miss the new functions; bump the version"
  exit 1
fi
echo "the API matches the released $tag"
