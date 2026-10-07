#!/usr/bin/env bash
# =============================================================================
#  apt-retry.sh — apt with retries and a time limit
# -----------------------------------------------------------------------------
#  `apt-get` has no timeout of its own. One real run sat on `apt-get update`
#  for fifty minutes — not failing, just waiting on a mirror — while the
#  job's own timeout-minutes is 90. A step that hangs is worse than one that
#  fails: it burns the runner and produces nothing to read.
#
#  Three things, and all three were needed:
#
#    * three attempts with a pause, because a flaky mirror is transient and
#      the whole run should not die for it
#    * a per-invocation ceiling, so a hang becomes a failure with a message
#    * progress on every attempt, because silence is what made the fifty
#      minutes hard to read
#
#  Usage: apt-retry.sh update | install <package>...
# =============================================================================
set -euo pipefail

ATTEMPTS=3
TIMEOUT_SECONDS=300

what="${1:-}"
if [ -z "$what" ]; then
  echo "usage: apt-retry.sh update | install <package>..." >&2
  exit 2
fi
shift

# آرایه، نه eval: نامِ بسته‌ها از این می‌آید و eval روی ورودی، درسی است که
# لازم نیست دوباره پیدا شود.
#
# `-E` عمداً نیست: `sudo -E` روی این رانرها می‌گوید «preserving the entire
# environment is not supported» و محیط را دور می‌اندازد. apt به متغیرِ محیطی
# برای این کار نیاز ندارد؛ `DEBIAN_FRONTEND` را با `env` می‌دهیم.
cmd=(sudo env DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Retries=3)

case "$what" in
  update)
    cmd+=(update)
    ;;
  install)
    [ "$#" -gt 0 ] || { echo "apt-retry: install needs packages" >&2; exit 2; }
    cmd+=(install -y --no-install-recommends)
    cmd+=("$@")
    ;;
  *)
    echo "apt-retry: unknown action '$what' (want: update | install)" >&2
    exit 2
    ;;
esac

status=1
for attempt in $(seq 1 "$ATTEMPTS"); do
  echo "==> apt $what, attempt $attempt of $ATTEMPTS"
  # `if ! cmd` یعنی `$?` دیگر کدِ فرمان نیست — کدِ «ناموفق بودنِ if» است، یعنی
  # همیشه صفر. این را یک بار واقعی دیدم: سه تلاش شکست خورد و اسکریپت
  # «last exit 0» چاپ کرد و خودش صفر برگشت. `set -e` را باید درست دور بزنیم.
  if timeout "$TIMEOUT_SECONDS" "${cmd[@]}"; then
    exit 0
  else
    status=$?
  fi
  # timeout ممکن است با سیگنال بمیرد؛ همان کد را برمی‌گردانیم.
  echo "==> apt $what gave up this attempt (exit $status)" >&2
  if [ "$attempt" -lt "$ATTEMPTS" ]; then
    sleep 20
  fi
done

echo "apt $what failed after $ATTEMPTS attempts (last exit $status)" >&2
exit "$status"