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
check="$work/check.R"
cat > "$check" <<'EOF'
step <- function(text) {
  cat("step:", text, "\n", file = stderr())
  flush(stderr())
}
fixture <- commandArgs(trailingOnly = TRUE)[1]
step("library(quantion)")
library(quantion)
step("parse_ion")
file <- parse_ion(fixture)
step("get_transition")
trace <- get_transition(file, 0)
stopifnot(length(trace$x) > 0)
step("find_transitions")
peaks <- find_transitions(file, data.frame(id = "alanine", precursor = 89.04768, product = 89.04768, rt = 2.385, range = 0.1))
stopifnot(nrow(peaks) == 1, peaks$intensity > 0)
cat("quantion", as.character(packageVersion("quantion")), "loads and runs from the prebuilt library\n")
EOF
if command -v cygpath >/dev/null 2>&1; then
  fixture="$(cygpath -m "$fixture")"
  check="$(cygpath -m "$check")"
fi
R_LIBS="$r_lib" Rscript "$check" "$fixture"
