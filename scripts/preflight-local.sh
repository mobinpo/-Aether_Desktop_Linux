#!/usr/bin/env bash
# گام‌های Preflight که بی‌ویندوز هم قابل اجرا هستند، عیناً از .github/workflows.
# قبل از هر بسته‌بندی اجرا می‌شود: یک ZIP که CI ردش می‌کند، تحویل نیست.
set -u
cd "$(dirname "$0")/.." || exit 1
fail=0

echo "── ۱) فایل‌های لازم"
missing=0
for f in $(sed -n '/required=(/,/)/p' .github/workflows/build.yml | grep -oE '"[^"]+"' | tr -d '"'); do
  [ -e "$f" ] || { echo "  ✗ غایب: $f"; missing=1; }
done
[ $missing -eq 0 ] && echo "  ✓ همه هست" || fail=1

echo "── ۲) JSON‌ها معتبرند"
for f in package.json src-tauri/tauri.conf.json src-tauri/capabilities/default.json; do
  if python3 -c "import json,sys; json.load(open('$f'))" 2>/dev/null; then
    echo "  ✓ $f"
  else
    echo "  ✗ $f"; fail=1
  fi
done

echo "── ۳) نسخه در همهٔ جاها یکی است"
v_pkg=$(python3 -c "import json;print(json.load(open('package.json'))['version'])")
v_conf=$(python3 -c "import json;print(json.load(open('src-tauri/tauri.conf.json'))['version'])")
v_cargo=$(grep -m1 '^version' src-tauri/Cargo.toml | cut -d'"' -f2)
v_iss=$(grep -m1 'define MyAppVersion' installer/aether.iss | cut -d'"' -f2)
# اولین version در XML، اعلانِ خودِ XML است (1.0) — نسخهٔ برنامه در assemblyIdentity است.
v_man=$(grep -oE 'assemblyIdentity version="[0-9.]+"' src-tauri/windows-manifest.xml | head -1 | cut -d'"' -f2)
echo "  package.json=$v_pkg tauri.conf=$v_conf Cargo=$v_cargo installer=$v_iss manifest=$v_man"
if [ "$v_pkg" = "$v_conf" ] && [ "$v_pkg" = "$v_cargo" ] && [ "$v_pkg" = "$v_iss" ] && [ "$v_man" = "$v_pkg.0" ]; then
  echo "  ✓ همه $v_pkg"
else
  echo "  ✗ ناهمخوانی نسخه"; fail=1
fi

echo "── ۴) حالت پراکسی نباید مسیر کد داشته باشد"
if grep -rniE 'proxy[_-]?mode' src src-tauri/src installer 2>/dev/null | grep -vE 'profile\.rs|\.md$'; then
  echo "  ✗ پیدا شد"; fail=1
else
  echo "  ✓ هیچ مسیری نیست"
fi

echo "── ۵) بیلد وب"
if npx vite build >/tmp/vite.log 2>&1; then
  echo "  ✓ $(grep -c . /tmp/vite.log) خط، بدون خطا"
else
  echo "  ✗ شکست:"; tail -5 /tmp/vite.log; fail=1
fi

# دقیقاً همان فرمانی که CI صدا می‌زند — نه یک بازنویسی مشابه. اگر این دو از هم
# فاصله بگیرند، preflight سبز می‌شود و بیلد می‌شکند؛ یک بار همین اتفاق افتاد.
echo "── ۶) تست‌های JS (npm test — همان فرمان CI)"
if out=$(npm test 2>&1); then
  echo "  ✓ npm test"
else
  echo "  ✗ npm test"; echo "$out" | tail -20; fail=1
fi

echo "── ۷) تست چیدمان در مرورگر واقعی (npm run test:layout)"
if out=$(npm run test:layout 2>&1); then
  if echo "$out" | grep -q '^SKIP'; then
    echo "  ~ رد شد: $(echo "$out" | grep '^SKIP' | head -1)"
  else
    echo "  ✓ $(echo "$out" | grep -E 'ALL OK|همه' | tail -1)"
  fi
else
  echo "  ✗ test:layout"; echo "$out" | tail -20; fail=1
fi

echo
[ $fail -eq 0 ] && echo "PREFLIGHT OK" || echo "PREFLIGHT FAILED"
exit $fail
