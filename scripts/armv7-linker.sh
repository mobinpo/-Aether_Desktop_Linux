#!/usr/bin/env bash
# =============================================================================
#  armv7-linker.sh — armv7 cross-linker that knows where the sysroot is
# -----------------------------------------------------------------------------
#  Cargo takes the linker as a *program*, not as a program plus arguments — there
#  is no `CARGO_TARGET_<T>_LINKER_ARGS`. So the sysroot prefix has to come from a
#  wrapper.
#
#  Why the prefix is needed at all: libc.so, libc_nonshared.a and
#  ld-linux-armhf.so.3 are recorded with absolute paths
#  (`/lib/arm-linux-gnueabihf/…`). The linker resolves those against the real
#  root, where the host's amd64 files live, and reports
#
#      cannot find /lib/arm-linux-gnueabihf/libc.so.6
#
#  `-L` cannot help — it adds search directories but does not rewrite an absolute
#  path. `--sysroot` prepends a prefix to it, which is exactly what is needed.
#  Measured on an armv7 + webkit2gtk build: `-L` alone fails on those three
#  files, and this wrapper produces `ELF 32-bit LSB pie executable, ARM, EABI5`.
#
#  AETHER_SYSROOT must point at the directory holding `usr/` and `lib/`.
# =============================================================================
set -euo pipefail

SYSROOT="${AETHER_SYSROOT:?AETHER_SYSROOT must point at the sysroot}"

exec arm-linux-gnueabihf-gcc --sysroot="$SYSROOT" "$@"
