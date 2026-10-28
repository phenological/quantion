#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

version="${1:-}"
[ -n "$version" ] || { echo "usage: make publish_R VERSION=X.Y.Z   (after make publish VERSION=X.Y.Z)"; exit 1; }
version="${version#v}"
tag="v$version"

git fetch --quiet --tags origin
git rev-parse --verify --quiet "refs/tags/$tag" >/dev/null || {
  echo "tag $tag does not exist, run make publish VERSION=$version first"; exit 1
}

tagged_version="$(git show "$tag:wrappers/r/DESCRIPTION" | sed -n 's/^Version: *//p' | head -1)"
[ "$tagged_version" = "$version" ] || { echo "$tag has R version '$tagged_version', not $version"; exit 1; }

git cat-file -e "$tag:scripts/build-r-package.sh" 2>/dev/null || {
  echo "$tag has no scripts/build-r-package.sh, so it can't be built for CRAN"; exit 1
}

repository="$(git remote get-url origin | sed -E 's#^.*github\.com[:/]##; s#\.git$##')"

gh workflow run publish-r.yml --repo "$repository" --ref main -f tag="$tag"
echo "started the Publish R workflow for $tag: https://github.com/$repository/actions/workflows/publish-r.yml"
echo "when it passes, download quantion_$version.tar.gz from the $tag release and submit it at https://cran.r-project.org/submit.html"
