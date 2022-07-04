#!/usr/bin/env bash

# install.sh
# ==========
# end-user `rustracer` installation script:
#  * downloads particular release (default `latest+musl`)
#  * downloads related checksum and signatures
#  * controls related  checksum and signatures
#  * installs inside given $PREFIX (default `~/.local`)

set -e

os=$(uname -s)
arch=$(uname -m)
libc=${1:-musl}
version=${2:-latest}
platform="${arch}-unknown-${os,,}-${libc}"
PREFIX=${PREFIX-$HOME/.local}

_die() {
   printf '[Error] %s\n' "$@" >&2
   exit 1
}
_info() {
   printf '[Info] %s\n' "$@"
}

_check_deps() {
   local deps
   deps=(curl jq openssl)
   for dep in "${deps[@]}"
   do
      type "$dep" >/dev/null 2>&1 || {
         _die "Cannot find ${dep} in your \$PATH" \
              "Install rustracer manually or install ${dep}"
      }
   done
}

_download_release() {
   local checksums_uri
   local rustracer_uri
   local repo="https://github.com/andros21/rustracer"
   local repo_api="https://api.github.com/repos/andros21/rustracer"
   if [ "${version}" == "latest" ]
   then
      checksums_uri=$(curl -sL "${repo_api}/releases/latest" \
         | jq -r ".assets[] | select(.name | test(\"_checksums_sha256.txt\$\")) | .browser_download_url")
      rustracer_uri=$(curl -sL "${repo_api}/releases/latest" \
         | jq -r ".assets[] | select(.name | test(\"${platform}.tar.gz\$\")) | .browser_download_url")
   else
      curl -sL "${repo_api}/releases" \
         | jq -r ".[] | select(.tag_name | test(\"${version}\$\"))" \
         | grep -qF "${version}" \
         || _die "Invalid rustracer version ${version}" \
                 "see available at ${repo}/releases"
      checksums_uri="${repo}/releases/download/${version}/rustracer-${version}_checksums_sha256.txt"
      rustracer_uri=$(curl -sL "${repo_api}/releases" \
         | jq -r ".[] | select(.tag_name | test(\"${version}\$\"))" \
         | jq -r ".assets[] | select(.name | test(\"${platform}.tar.gz\$\")) | .browser_download_url")
   fi
   [[ -n "${rustracer_uri}" ]] \
      || _die "Invalid rustracer platform ${platform}" \
              "see available at ${repo}/releases"
   checksums="$(basename "${checksums_uri}")"
   rustracer="$(basename "${rustracer_uri}")"
   _info "Downloading rustracer ${version} checksums"
   [[ -f "${checksums}" ]]             || curl -sSJOL "${checksums_uri}"
   [[ -f "${checksums}-keyless.pem" ]] || curl -sSJOL "${checksums_uri}-keyless.pem"
   [[ -f "${checksums}-keyless.sig" ]] || curl -sSJOL "${checksums_uri}-keyless.sig"
   _info "Downloading rustracer ${version} for platform ${platform}"
   [[ -f "${rustracer}" ]]             || curl -sSJOL "${rustracer_uri}"
   [[ -f "${rustracer}-keyless.pem" ]] || curl -sSJOL "${rustracer_uri}-keyless.pem"
   [[ -f "${rustracer}-keyless.sig" ]] || curl -sSJOL "${rustracer_uri}-keyless.sig"
}

_verify_release() {
   _info "Verifying   rustracer ${version} checksums signature"
   openssl dgst -sha256 -out /dev/null \
      -verify <(base64 -d "${checksums}-keyless.pem" | openssl x509 -pubkey -noout 2>/dev/null) \
      -signature <(base64 -d "${checksums}-keyless.sig") \
      "${checksums}" && rm -f "${checksums}-keyless".{sig,pem}
   _info "Verifying   rustracer ${version} for platform ${platform} checksum"
   sha256sum -c --quiet --ignore-missing "${checksums}" && rm -f "${checksums}"
   _info "Verifying   rustracer ${version} for platform ${platform} signature"
   openssl dgst -sha256 -out /dev/null \
      -verify <(base64 -d "${rustracer}-keyless.pem" | openssl x509 -pubkey -noout 2>/dev/null) \
      -signature <(base64 -d "${rustracer}-keyless.sig") \
      "${rustracer}" && rm -f "${rustracer}-keyless".{sig,pem}
}

_install_release() {
   _info "Installing  rustracer ${version} for platform ${platform}"
   tar xzf "${rustracer}" && rm -f "${rustracer}"
   install -Dm755 "${rustracer%.tar.gz}/bin/rustracer" "${PREFIX}/bin/rustracer" \
      && _info "rustracer was installed successfully to ${PREFIX}/bin/rustracer"
}

_check_deps
_download_release
_verify_release
_install_release
if type rustracer >/dev/null 2>&1
then
   printf "\nRun 'rustracer --help' to get started\n"
else
   printf "\nManually add %s/bin to your \$PATH\n" "${PREFIX}"
   printf "Run 'rustracer --help' to get started\n"
fi
