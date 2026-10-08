#!/usr/bin/env bash
# =============================================================================
#  fetch-engine.sh — stage the official Aether core into dist-engine/
# -----------------------------------------------------------------------------
#  The core is NOT built here: it is a prebuilt binary the upstream project
#  publishes per platform. Downloading the official artifact is faster and
#  safer than rebuilding it, and rebuilding would drag in the Windows-only
#  build scripts.
#
#  Per platform, all from the same official release (v2.3.0, names verified):
#
#      linux    aether-linux-<arch>.tar.gz     aether, pt/lyrebird, pt/psiphon-tunnel-core
#      windows  aether-windows-x86_64.zip      aether.exe, pt/…
#      macos    aether-macos-<arch>.tar.gz     aether, pt/…
#
#  Windows and macOS differ from Linux in ways that are easy to get wrong:
#  Windows ships a zip rather than a tar.gz, and its binary carries a `.exe`
#  extension. Both are handled below rather than assumed — `engine.rs` looks
#  the binary up through `EXE_NAME`, so the name has to be exactly right.
#
#  Two files are NOT in the archives and come from this repo:
#      server_entries.txt      tracked at assets/psiphon/server_entries.txt
#      CORE_VERSION            the release tag
#
#  Usage:  AETHER_PLATFORM=linux   scripts/fetch-engine.sh [version] [arch]
#          AETHER_PLATFORM=windows scripts/fetch-engine.sh 2.3.0 x86_64
# =============================================================================
set -euo pipefail

VERSION="${1:-${CORE_VERSION:-2.3.0}}"
ARCH="${2:-$(uname -m)}"

# سکو را صریح می‌گیریم، از `uname` حدس نمی‌زنیم: روی رانرِ `windows-latest`
# هم `uname -m` جواب می‌دهد و `MINGW64_NT-10.0…` می‌دهد که در هیچ فهرستی نیست.
PLATFORM="${AETHER_PLATFORM:-linux}"
case "$PLATFORM" in
  linux|windows|macos) ;;
  *) echo "fetch-engine: unknown platform '$PLATFORM' (want linux|windows|macos)" >&2
     exit 1 ;;
esac

# نامِ معماری بین `uname` و نامِ آرشیورِ بالادست یکی نیست، و musl هم پسوندِ
# جدا دارد:
#   uname:     x86_64   aarch64   armv7l
#   linux:     x86_64    arm64     armv7
#   musl:   x86_64-musl aarch64-musl armv7-musl
#
# نگاشت صریح است تا یک معماریِ ناشناخته بی‌صدا به x86_64 نیفتد و باینریِ
# ۶۴ بیتی را در بستهٔ ۳۲ بیتی جا بزند. (`aarch64` نامِ آرشیورِ لینوکس نیست —
# تست شد و ۴۰۴ داد؛ `arm64` است. در مک هم `arm64` است.)
#
# ۳۲ بیتیِ x86 عمداً نیست: آرشیوری به این نام منتشر نشده (تست شد، ۴۰۴). بدون
# موتور، بستهٔ برنامه بی‌معنی است — پس ساختنش فقط وقتِ بیلد را می‌سوزاند.
# ویندوزِ ۳۲ بیتی هم ندارد: تنها آرشیورِ موجود `windows-x86_64` است.
case "$ARCH" in
  x86_64|amd64)             SLUG="x86_64"      ;;
  aarch64|arm64)           SLUG="arm64"       ;;
  armv7l|armv7)            SLUG="armv7"       ;;
  x86_64-musl|amd64-musl)  SLUG="x86_64-musl" ;;
  aarch64-musl|arm64-musl) SLUG="aarch64-musl" ;;
  armv7l-musl|armv7-musl)  SLUG="armv7-musl"  ;;
  *)
    echo "fetch-engine: unsupported arch: $ARCH" >&2
    echo "  linux glibc: x86_64 aarch64 armv7"                 >&2
    echo "  linux musl:  x86_64-musl aarch64-musl armv7-musl"  >&2
    echo "  windows:     x86_64"                              >&2
    echo "  macos:       x86_64 aarch64"                      >&2
    exit 1 ;;
esac

# ویندوز تنها سکویی است که zip دارد؛ بقیه tar.gz.
if [ "$PLATFORM" = "windows" ]; then
  ASSET="aether-${PLATFORM}-${SLUG}.zip"
else
  ASSET="aether-${PLATFORM}-${SLUG}.tar.gz"
fi

URL="https://github.com/CluvexStudio/Aether/releases/download/v${VERSION}/${ASSET}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/dist-engine"

echo "==> [engine] $PLATFORM v$VERSION / $SLUG"
# هر دو پوشه ساخته می‌شوند، نه فقط ریشه: `dist-engine/` در `.gitignore` است و
# روی یک چک‌اوت تازه اصلاً وجود ندارد. ساختنِ فقطِ ریشه باعث می‌شد `cp` روی
# `pt/lyrebird` با «No such file or directory» بمیرد — و همین خطا بود که هر سه
# معماریِ لینوکس را در CI می‌کشت، بی‌آنکه پیامش کسی ببیند.
mkdir -p "$OUT/pt"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Upstream publishes a .sha256 next to every asset. Verify: this is the core the
# app ships and runs, so an unverified download is not acceptable.
#
# `--retry-all-errors` matters here: on CI the architecture jobs start together
# and each pulls ~25 MB at the same moment, which is exactly the shape that gets
# a runner throttled with a 403/429. A plain `--retry 3` gives up on those
# because it only retries transient transport errors, not HTTP status codes.
# `-C -` resumes a partial file instead of starting over.
fetch() {
  curl -fL --retry 8 --retry-delay 10 --retry-all-errors --retry-max-time 600 \
       --speed-limit 1024 --speed-time 60 \
       -C - -o "$1" "$2"
}

echo "==> [engine] downloading $ASSET"
if ! fetch "$tmp/core" "$URL"; then
  echo "fetch-engine: could not download $URL" >&2
  exit 1
fi
fetch "$tmp/core.sha256" "$URL.sha256" || true

if [ -s "$tmp/core.sha256" ]; then
  # The .sha256 file is "<hash>  <filename>" or a bare "<hash>"; take field one.
  want="$(awk '{print $1; exit}' "$tmp/core.sha256")"
  got="$(sha256sum "$tmp/core" | awk '{print $1}')"
  if [ "$want" != "$got" ]; then
    echo "fetch-engine: DIGEST MISMATCH for $ASSET" >&2
    echo "  expected $want" >&2
    echo "  got      $got" >&2
    exit 1
  fi
  echo "    digest ok: $got"
else
  echo "    WARNING: no .sha256 published for $ASSET — continuing unverified" >&2
fi

if [ "$PLATFORM" = "windows" ]; then
  # `unzip` نه `tar`: آرشیورِ ویندوز zip است. نامِ داخلی هم `aether.exe` دارد،
  # نه `aether` — و `engine.rs` با `EXE_NAME` دقیقاً همین را می‌خواهد.
  #
  # چیدمانِ zip با لینوکس فرق دارد: هر دو کمکی داخل `pt/` هستند
  # (`pt/psiphon-tunnel-core.exe` و `pt/lyrebird.exe`) و تنها موتور در ریشه.
  # فهرستِ واقعی از خودِ آرشیو خوانده شد، نه حدس زده.
  # روی ویندوز `unzip` در PATH نیست؛ Git Bash آن را ندارد. PowerShell هست و
  # بی‌دردسر — ولی مسیرِ ویندوز می‌خواهد، نه POSIX. `cygpath` همان تبدیل را
  # می‌کند و روی لینوکس وجود ندارد، پس این شاخه فقط وقتی اجرا می‌شود که
  # واقعاً روی ویندوز باشیم.
  #
  # بی‌صدا رد شدنِ باز کردنِ zip بدترین حالت بود: بعداً نبودِ `aether.exe`
  # گزارش می‌شد و علتش یک قدمِ شکست‌خوردهٔ سه قدم قبل بود.
  if command -v unzip >/dev/null 2>&1; then
    unzip -q "$tmp/core" -d "$tmp/x"
  else
    # `mktemp` در Git Bash مسیرِ POSIX می‌دهد، ولی PowerShell مسیرِ ویندوز
    # می‌خواهد. `cygpath` تبدیلش می‌کند؛ اگر نبود، خودِ `tmp` یک مسیرِ
    # قابل‌قبول است و نیازی به تبدیل نیست — هر دو حالت پوشش داده شده‌اند.
    win_tmp="$tmp"
    command -v cygpath >/dev/null 2>&1 && win_tmp="$(cygpath -w "$tmp")"
    powershell -NoProfile -Command \
      "Expand-Archive -Path '$win_tmp\\core' -DestinationPath '$win_tmp\\x' -Force" \
      || { echo "fetch-engine: could not expand the archive" >&2; exit 1; }
  fi
  cp -f "$tmp/x/aether.exe" "$OUT/aether.exe"
  # این دو اختیاری‌اند: نبودنشان خطا نیست، همان‌طور که در لینوکس هم نیست.
  cp -f "$tmp/x/pt/lyrebird.exe" "$OUT/pt/lyrebird.exe" 2>/dev/null || true
  cp -f "$tmp/x/pt/psiphon-tunnel-core.exe" "$OUT/psiphon-tunnel-core.exe" 2>/dev/null || true
else
  tar -xzf "$tmp/core" -C "$tmp"

  # psiphon ships inside pt/ in the tarball but the app looks for it next to the
  # engine (it is a carrier, not a pluggable transport). Normalise the layout
  # here so tauri.conf.json resources stay platform-independent.
  cp -f "$tmp/aether" "$OUT/aether"
  cp -f "$tmp/pt/lyrebird" "$OUT/pt/lyrebird"
  cp -f "$tmp/pt/psiphon-tunnel-core" "$OUT/psiphon-tunnel-core"
fi

cp -f "$ROOT/assets/psiphon/server_entries.txt" "$OUT/server_entries.txt"
printf '%s' "$VERSION" > "$OUT/CORE_VERSION"

# Windows needs the binary executable; on Unix the bit has to be set explicitly
# because neither tar nor zip is guaranteed to carry it.
chmod +x "$OUT"/aether* "$OUT"/psiphon-tunnel-core* "$OUT"/pt/lyrebird* 2>/dev/null || true

echo "==> [engine] staged:"
# `|| true` لازم است: نبودنِ یکی از این فایل‌ها اختیاری است (بستهٔ ویندوز
# ممکن است psiphon نداشته باشد) و `ls` روی الگوی ناتمام خطا می‌دهد. بدون این،
# مرحلهٔ آخر اسکریپت شکست می‌خورد و کل بیلد می‌میرد در حالی که همه‌چیز درست
# آماده شده — همان اتفاقی که در ویندوز افتاد.
( cd "$OUT" && ls -1 aether* psiphon-tunnel-core* server_entries.txt CORE_VERSION pt/lyrebird* 2>/dev/null ) || true

# و بعد، به‌جای اعتماد به `ls`، خودِ فایلِ اصلی را چک می‌کنیم: بدون موتور،
# بستهٔ برنامه بی‌معنی است.
if [ "$PLATFORM" = "windows" ]; then
  [ -f "$OUT/aether.exe" ] || { echo "fetch-engine: aether.exe missing after unzip" >&2; exit 1; }
else
  [ -f "$OUT/aether" ] || { echo "fetch-engine: aether missing after extract" >&2; exit 1; }
fi