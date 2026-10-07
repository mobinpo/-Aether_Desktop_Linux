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

# armhf lives on ports.ubuntu.com, not archive.ubuntu.com — and only there.
# هر دو طرف باید برچسب داشته باشد: `ports` نباید amd64 بدهد (۴۰۴ می‌دهد) و
# `archive` هم نباید armhf بخواهد (۴۰۴ می‌دهد). برچسبِ یک‌طرفه کافی نیست.
#
# فهرستِ پیش‌فرض کامل جایگزین می‌شود، نه وصله‌اش. دلیلش قالب است: ubuntu:nole
# از deb822 (`ubuntu.sources`) استفاده می‌کند و `sed` روی `sources.list` آن
# بی‌اثر است — یعنی پیش‌فرض‌ها همچنان armhf می‌خواستند و همین‌طور ۴۰۴ دادند.
# این اسکریپت در یک چک‌اوت تازه اجرا می‌شود، پس چیزی برای از دست دادن نیست.
rm -f /etc/apt/sources.list.d/ubuntu.sources /etc/apt/sources.list
cat > /etc/apt/sources.list <<'EOF'
deb [arch=amd64] http://archive.ubuntu.com/ubuntu noble main restricted universe multiverse
deb [arch=amd64] http://archive.ubuntu.com/ubuntu noble-updates main restricted universe multiverse
deb [arch=amd64] http://security.ubuntu.com/ubuntu noble-security main restricted universe multiverse
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports noble main restricted universe multiverse
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports noble-updates main restricted universe multiverse
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports noble-security main restricted universe multiverse
EOF

apt-get update
apt-get install -y --no-install-recommends qemu-user-static

# binfmt ثبت می‌شود تا dpkg بتواند اسکریپت‌های armhf را اجرا کند. این کار را
# `tonistiigi/binfmt` می‌کند — یک بار، بیرون از کانتینر.
#
# چرا دستی نه: نوشتنِ ورودیِ binfmt باینری است (بایت‌های 0x7F و 0xFF)، و
# `printf` در dash آن‌ها را نمی‌نویسد ("Invalid argument"). تستِ محلی نشان داد
# نام‌های `qemu-arm` و حتی طول‌های مختلفِ mask رفتار یکسانی ندارند و
# بازخوردِ کرنل فقط EINVAL است. این ابزار همان کار را در یک خط می‌کند و
# تست‌شده است.
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
# data. The armhf apt itself would add hundreds of megabytes and pull in an armhf
# dpkg we never want to run.
mkdir -p /out/sysroot
dpkg-query -W -f='${Package} ${Architecture}\n' \
  | awk '$2 == "armhf" { print $1 }' \
  | xargs -r dpkg-query -L \
  | grep -E '^/(usr/(lib|include|share)|lib|etc)/' \
  | while read -r f; do
      # Symlinks and directories are included; a dangling one is not an error.
      [ -e "$f" ] && cp -a --parents "$f" /out/sysroot 2>/dev/null || true
    done

# `opensslconf.h` در include عمومی نیست: openssl آن را در مسیرِ مخصوصِ معماری
# می‌گذارد (`/usr/include/arm-linux-gnueabihf/openssl/` — با بستهٔ armhf تست شد).
# فیلترِ بالا آن را کپی می‌کند، ولی `openssl-sys` دنبالِ
# `<include>/openssl/opensslconf.h` می‌گردد. پس یک کپیِ دیگر همان‌جا می‌گذاریم
# تا هر دو مسیر جواب بدهد.
if [ -f /out/sysroot/usr/include/arm-linux-gnueabihf/openssl/opensslconf.h ]; then
  cp -f /out/sysroot/usr/include/arm-linux-gnueabihf/openssl/opensslconf.h \
        /out/sysroot/usr/include/openssl/opensslconf.h
  echo "opensslconf.h also staged at usr/include/openssl/"
else
  echo "ERROR: opensslconf.h missing — openssl-sys will fail on the host header" >&2
  exit 1
fi

count="$(find /out/sysroot -type f | wc -l)"
echo "sysroot files: $count"
if [ "$count" -lt 100 ]; then
  echo "sysroot looks empty ($count files) — pkg-config would find nothing" >&2
  exit 1
fi
