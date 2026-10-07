#!/usr/bin/env bash
# =============================================================================
#  build-armhf-sysroot.sh — build an armhf sysroot for cross-compiling armv7
# -----------------------------------------------------------------------------
#  This runs INSIDE an ubuntu:noble container, not on the runner. It writes the
#  result to /out/sysroot.
#
#  Why a container at all: installing armhf packages directly on an x86_64
#  runner via multiarch resolves and unpacks them — 326 packages, 521 MB — but
#  nothing ever executes them. dpkg then needs to run armhf maintainer scripts,
#  pkg-config needs armhf `glib-compile-schemas` and `ldconfig`, and all of them
#  die on "Exec format error". A sysroot does not fix that, because the problem
#  is execution, not paths. Inside a container with qemu-user-static and binfmt
#  registered, dpkg genuinely runs armhf code, so schemas and caches get built
#  correctly and what we copy out is a sysroot that works.
#
#  This is not the hand-built sysroot the old comment claimed was needed —
#  Ubuntu builds webkit2gtk for armhf; we install it and keep the libraries.
# =============================================================================
set -euo pipefail

# armhf lives on ports.ubuntu.com, not archive.ubuntu.com. Restricting the
# default list to amd64 keeps apt from requesting armhf from a mirror that 404s
# it — those 404s are tolerated during update and then fail the install.
{
  echo 'deb http://ports.ubuntu.com/ubuntu-ports noble main restricted universe multiverse'
  echo 'deb http://ports.ubuntu.com/ubuntu-ports noble-updates main restricted universe multiverse'
  echo 'deb http://ports.ubuntu.com/ubuntu-ports noble-security main restricted universe multiverse'
} > /etc/apt/sources.list.d/armhf.list
sed -i 's/^deb /deb [arch=amd64] /' /etc/apt/sources.list

apt-get update
# binfmt must be registered before dpkg runs anything armhf, or the first
# maintainer script dies on "Exec format error".
apt-get install -y --no-install-recommends qemu-user-static
update-binfmts --enable armhf || true

dpkg --add-architecture armhf
apt-get update
apt-get install -y --no-install-recommends \
  libwebkit2gtk-4.1-dev:armhf \
  libjavascriptcoregtk-4.1-dev:armhf \
  libgtk-3-dev:armhf \
  libappindicator3-dev:armhf \
  librsvg2-dev:armhf \
  libssl-dev:armhf

# Keep only what the linker needs: libraries, headers, pkgconfig and runtime
# data. The armhf apt and libc6-dev would add hundreds of megabytes and pull in
# an armhf dynamic loader, which we do not want at link time — the cross gcc
# brings its own.
mkdir -p /out/sysroot
dpkg-query -W -f='${Package} ${Architecture}\n' \
  | awk '$2 == "armhf" { print $1 }' \
  | xargs -r dpkg-query -L \
  | grep -E '^/usr/(lib|include|share)' \
  | while read -r f; do
      # Symlinks and directories are included; a dangling one is not an error.
      [ -e "$f" ] && cp -a --parents "$f" /out/sysroot 2>/dev/null || true
    done

count="$(find /out/sysroot -type f | wc -l)"
echo "sysroot files: $count"
if [ "$count" -lt 100 ]; then
  echo "sysroot looks empty ($count files) — pkg-config would find nothing" >&2
  exit 1
fi
