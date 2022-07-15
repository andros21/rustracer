#!/usr/bin/env bash

# cd.sh
# =====
# `rustracer` continuos deployment workflow (`cd.yml`) helper script,
# it assemble, checksum and (co)sign artifacts for releases

set -e

GNU="rustracer-${GITHUB_REF_NAME}-x86_64-unknown-linux-gnu"
MUSL="rustracer-${GITHUB_REF_NAME}-x86_64-unknown-linux-musl"
SUM="rustracer-${GITHUB_REF_NAME}_checksums_sha256.txt"

_assemble() {
   mkdir -pv "${GNU}/bin"
   mkdir -pv "${MUSL}/bin"
   cp -v target/release/rustracer \
      "${GNU}/bin/rustracer"
   cp -v target/x86_64-unknown-linux-musl/release/rustracer \
      "${MUSL}/bin/rustracer"
   cp -v {README.md,LICENSE} "${GNU}/"
   cp -v {README.md,LICENSE} "${MUSL}/"
   tar czfv "${GNU}.tar.gz" "${GNU}"
   tar czfv "${MUSL}.tar.gz" "${MUSL}"
   sha256sum ./*.tar.gz > "${SUM}"
}

_cosign() {
   for blob in "${GNU}.tar.gz" "${MUSL}.tar.gz" "${SUM}"
   do
      cosign sign-blob "${blob}" \
         --output-signature "${blob}-keyless.sig" \
         --output-certificate "${blob}-keyless.pem"
   done
}

case $1 in
   assemble) _assemble;;
   cosign)   _cosign;;
   *)        echo "cd.sh (assemble|cosign)";;
esac
