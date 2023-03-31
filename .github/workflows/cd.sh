#!/usr/bin/env bash

# cd.sh
# =====
# `rustracer` continuos deployment workflow (`cd.yml`) helper script,
# it assemble, checksum and (co)sign artifacts for releases

set -xe

PLATFORMS=("x86_64-unknown-linux-gnu" "x86_64-unknown-linux-musl")
BASENAME="rustracer-${GITHUB_REF_NAME}"
CHECKSUM="rustracer-${GITHUB_REF_NAME}_checksums_sha256.txt"

_assemble() {
   for PLATFORM in "${PLATFORMS[@]}"
   do
      mkdir -pv "${BASENAME}-${PLATFORM}/bin"
      cp -v "target/${PLATFORM}/release/rustracer" \
         "${BASENAME}-${PLATFORM}/bin/rustracer"
      cp -v {README.md,LICENSE} "${BASENAME}-${PLATFORM}/"
      tar czfv "${BASENAME}-${PLATFORM}.tar.gz" "${BASENAME}-${PLATFORM}"
   done
   sha256sum ./*.tar.gz > "${CHECKSUM}"
}

_cosign() {
   for PLATFORM in "${PLATFORMS[@]}"
   do
      cosign sign-blob -y "${BASENAME}-${PLATFORM}.tar.gz" \
         --output-signature "${BASENAME}-${PLATFORM}.tar.gz-keyless.sig" \
         --output-certificate "${BASENAME}-${PLATFORM}.tar.gz-keyless.pem"
   done
   cosign sign-blob -y "${CHECKSUM}" \
      --output-signature "${CHECKSUM}-keyless.sig" \
      --output-certificate "${CHECKSUM}-keyless.pem"
}

case $1 in
   assemble) _assemble;;
   cosign)   _cosign;;
   *)        echo "cd.sh (assemble|cosign)";;
esac
