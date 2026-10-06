#!/usr/bin/env bash
# =============================================================================
#  fetch-engine.sh — stage the official Aether core into dist-engine/
# -----------------------------------------------------------------------------
#  The core is NOT built here: it is a prebuilt binary the upstream project
#  publishes per platform. Downloading the official artifact is faster and
#  safer than rebuilding it, and rebuilding would drag in the Windows-only
#  build scripts.
#
#  The tarball contains exactly three files:
#      aether                  the engine
#      pt/lyrebird             Tor pluggable transport (obfs4 / meek / webtunnel)
#      pt/psiphon-tunnel-core  the psiphon carrier (stage 2)
#
#  Two files are NOT in it and must come from this repo:
#      server_entries.txt      tracked at assets/psiphon/server_entries.txt
#      CORE_VERSION            derived from the release tag
#
#  Usage:  scripts/fetch-engine.sh [version] [arch]
#          defaults: version 2.3.0, arch from `uname -m`
# =============================================================================
set -euo pipefail

VERSION="${1:-${CORE_VERSION:-2.3.0}}"
ARCH="${2:-$(uname -m)}"

# uname says x86_64 / aarch64 / armv7l; upstream names them the same way except
# that 32-bit arm needs the explicit `armv7`.
case "$ARCH" in
  x86_64|aarch64|arm64) SLUG="$ARCH" ;;
  armv7l|armv7)          SLUG="armv7"   ;;
  i386|i686)             SLUG="x86"     ;;
  *) echo "fetch-engine: unsupported arch: $ARCH" >&2; exit 1 ;;
esac

ASSET="aether-linux-${SLUG}.tar.gz"
URL="https://github.com/CluvexStudio/Aether/releases/download/v${VERSION}/${ASSET}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/dist-engine"

echo "==> [engine] v$VERSION / $SLUG"
# هر دو پوشه ساخته می‌شوند، نه فقط ریشه: `dist-engine/` در `.gitignore` است و
# روی یک چک‌اوت تازه اصلاً وجود ندارد. ساختنِ فقطِ ریشه باعث می‌شد `cp` روی
# `pt/lyrebird` با «No such file or directory» بمیرد — و همین خطا بود که هر سه
# معماری را در CI می‌کشت، بی‌آنکه پیامش کسی ببیند.
mkdir -p "$OUT/pt"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Upstream publishes a .sha256 next to every asset. Verify: this is the core the
# app ships and runs, so an unverified download is not acceptable.
#
# `--retry-all-errors` matters here: on CI the three architecture jobs start
# together and each pulls ~24 MB at the same moment, which is exactly the shape
# that gets a runner throttled with a 403/429. A plain `--retry 3` gives up on
# those because it only retries transient transport errors, not HTTP status
# codes. `-C -` resumes a partial file instead of starting over.
fetch() {
  curl -fL --retry 8 --retry-delay 10 --retry-all-errors --retry-max-time 600 \
       --speed-limit 1024 --speed-time 60 \
       -C - -o "$1" "$2"
}

echo "==> [engine] downloading $ASSET"
if ! fetch "$tmp/core.tar.gz" "$URL"; then
  echo "fetch-engine: could not download $URL" >&2
  exit 1
fi
fetch "$tmp/core.sha256" "$URL.sha256" || true

if [[ -s "$tmp/core.sha256" ]]; then
  # The .sha256 file is "<hash>  <filename>" or a bare "<hash>"; take field one.
  want="$(awk '{print $1; exit}' "$tmp/core.sha256")"
  got="$(sha256sum "$tmp/core.tar.gz" | awk '{print $1}')"
  if [[ "$want" != "$got" ]]; then
    echo "fetch-engine: DIGEST MISMATCH for $ASSET" >&2
    echo "  expected $want" >&2
    echo "  got      $got" >&2
    exit 1
  fi
  echo "    digest ok: $got"
else
  echo "    WARNING: no .sha256 published for $ASSET — continuing unverified" >&2
fi

tar -xzf "$tmp/core.tar.gz" -C "$tmp"

# یک بستهٔ نیم‌کاره از تلاشِ قطع‌شده، `tar` را با خطای مبهم می‌دهد و پیامش
# هیچ اشاره‌ای به دانلود ندارد. اینجا صریح می‌گوییم مشکل از کجا بود.
for want in aether pt/lyrebird pt/psiphon-tunnel-core; do
  if [[ ! -f "$tmp/$want" ]]; then
    echo "fetch-engine: $ASSET is missing '$want' — download incomplete?" >&2
    exit 1
  fi
done

# psiphon ships inside pt/ in the tarball but the app looks for it next to the
# engine (it is a carrier, not a pluggable transport). Normalise the layout here
# so tauri.conf.json resources stay arch-independent.
cp -f "$tmp/aether"                 "$OUT/aether"
cp -f "$tmp/pt/lyrebird"            "$OUT/pt/lyrebird"
cp -f "$tmp/pt/psiphon-tunnel-core" "$OUT/psiphon-tunnel-core"
cp -f "$ROOT/assets/psiphon/server_entries.txt" "$OUT/server_entries.txt"
printf '%s' "$VERSION" > "$OUT/CORE_VERSION"
chmod +x "$OUT/aether" "$OUT/pt/lyrebird" "$OUT/psiphon-tunnel-core"

echo "==> [engine] staged:"
( cd "$OUT" && ls -1 aether psiphon-tunnel-core server_entries.txt CORE_VERSION pt/lyrebird )
