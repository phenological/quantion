#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
assets="$(cd "${1:?usage: check-r-prebuilt.sh <folder with libquantion-<platform>.<ext> and SHA256SUMS.txt>}" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

(cd "$root" && git ls-files -z wrappers/r | tar --null -T - -cf -) | tar -xf - -C "$work"
mv "$work/wrappers/r" "$work/r"
mkdir -p "$work/lib"

r_lib="$work/lib"
if command -v cygpath >/dev/null 2>&1; then
  r_lib="$(cygpath -m "$r_lib")"
  assets="$(cygpath -m "$assets")"
fi

echo "installing the R package the way a user does: no cargo, prebuilt library"
QUANTION_PREBUILT_DIR="$assets" R CMD INSTALL -l "$r_lib" "$work/r" 2>&1 | tee "$work/install.log"
if ! grep -q "cargo is not needed" "$work/install.log"; then
  echo "❌ the install did not use the prebuilt library"
  exit 1
fi
if ! grep -q "DONE (quantion)" "$work/install.log"; then
  echo "❌ the R package failed to install from the prebuilt library"
  exit 1
fi

fixture="$root/core/tests/fixtures/srm.ion"
if command -v cygpath >/dev/null 2>&1; then
  fixture="$(cygpath -m "$fixture")"
fi
R_LIBS="$r_lib" Rscript -e "
library(quantion)
file <- parse_ion('$fixture')
peaks <- find_transitions(file, data.frame(id = 'alanine', precursor = 89.04768, product = 89.04768, rt = 2.385, range = 0.1))
stopifnot(nrow(peaks) == 1, peaks\$intensity > 0)
cat('quantion', as.character(packageVersion('quantion')), 'loads and runs from the prebuilt library\n')
"
