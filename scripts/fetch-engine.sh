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
mkdir -p "$OUT"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# Upstream publishes a .sha256 next to every asset. Verify: this is the core the
# app ships and runs, so an unverified download is not acceptable.
echo "==> [engine] downloading $ASSET"
curl -fL --retry 3 --retry-delay 5 -o "$tmp/core.tar.gz" "$URL"
curl -fL --retry 3 --retry-delay 5 -o "$tmp/core.sha256" "$URL.sha256" || true

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
