#!/usr/bin/env bash
# بازرسی روی خودِ ZIP، از حالت استخراج‌شده — نه روی پوشهٔ کاری.
# چیزی که تحویل می‌شود همین است، پس همین سنجیده می‌شود.
set -u
V=/tmp/verify
rm -rf "$V" && mkdir -p "$V"
unzip -q /mnt/user-data/outputs/AetherDesktop-1.2.4.zip -d "$V"
cd "$V/Aether_Desktop-main" || exit 1
fail=0
ok()   { echo "  ✓ $1"; }
bad()  { echo "  ✗ $1"; fail=1; }
have() { if grep -qF "$2" "$1" 2>/dev/null; then ok "$3"; else bad "$3"; fi; }

echo "── نسخه"
for f in package.json src-tauri/tauri.conf.json; do
  v=$(python3 -c "import json;print(json.load(open('$f'))['version'])")
  [ "$v" = "1.2.4" ] && ok "$f = $v" || bad "$f = $v"
done
v=$(grep -m1 '^version' src-tauri/Cargo.toml | cut -d'"' -f2); [ "$v" = "1.2.4" ] && ok "Cargo.toml = $v" || bad "Cargo.toml = $v"
v=$(grep -m1 'define MyAppVersion' installer/aether.iss | cut -d'"' -f2); [ "$v" = "1.2.4" ] && ok "installer = $v" || bad "installer = $v"
v=$(grep -oE 'assemblyIdentity version="[0-9.]+"' src-tauri/windows-manifest.xml | head -1 | cut -d'"' -f2); [ "$v" = "1.2.4.0" ] && ok "manifest = $v" || bad "manifest = $v"

echo "── مورد ۱: اعمالِ پچ از دلِ چت"
have src-tauri/src/ai_prompts.rs 'APPLY_FENCE' 'ai_prompts: نامِ اختصاصیِ fence هست'
have src-tauri/src/ai_prompts.rs 'aether-apply' 'ai_prompts: fence همان aether-apply است'
have src-tauri/src/ai_prompts.rs 'fn split_chat_reply' 'ai_prompts: بلوک از متنِ حباب جدا می‌شود'
have src-tauri/src/ai_patch.rs 'fn chat_writable' 'ai_patch: فهرستِ مجازِ باریکِ چت'
have src-tauri/src/ai_patch.rs 'fn apply_chat' 'ai_patch: مسیرِ نوشتنِ چت'
have src-tauri/src/ai_session.rs 'fn vet_changes' 'ai_session: اعتبارسنجی پیش از رسم'
have src-tauri/src/ai_session.rs 'ProposedChange' 'ai_session: ساختارِ تغییرِ پیشنهادی با before/why'
have src-tauri/src/main.rs 'fn ai_apply_changes' 'main.rs: فرمانِ ai_apply_changes'
have src-tauri/src/main.rs 'ai_apply_changes,' 'main.rs: فرمان در invoke_handler ثبت شده'
have src/ai.js 'ai_apply_changes' 'ai.js: پلِ applyChanges'
have src/views/chat.js 'chat__apply' 'chat.js: دکمهٔ اعمال روی کارت'
have src/views/chat.js 'chat__changekey' 'chat.js: خطِ «قدیم → جدید»'
have src/styles/chat.css '.chat__changes' 'chat.css: استایلِ کارتِ پیشنهاد'
# کلیدهای خطرناک نباید در فهرستِ چت باشند.
for k in upstream routeDirect manualPeer backend splitMode lanShare killSwitch accessSecret; do
  if sed -n '/fn chat_writable/,/^}/p' src-tauri/src/ai_patch.rs | grep -q "\"$k\""; then
    bad "کلیدِ خطرناک در فهرستِ چت: $k"
  else
    ok "چت نمی‌تواند $k را بنویسد"
  fi
done

echo "── مورد ۲: حذف بی‌تأیید نیست"
have src/views/chat.js 'function confirmDialog' 'chat.js: دیالوگِ تأییدِ خودِ برنامه'
# سطلِ روی حباب و حذفِ گروهی هر دو به `ctx.remove` می‌روند و تأیید داخلِ همان یک
# تابع است — پس دو صدا زدنِ confirmDialog (حذف + پاک‌کردنِ گفت‌وگو) درست است و
# سه تا نشانهٔ کدِ تکراری بود.
have src/views/chat.js 'ctx.remove([message.id])' 'chat.js: سطلِ روی حباب به مسیرِ تأییدشده می‌رود'
have src/views/chat.js 'ctx.remove([...selection])' 'chat.js: حذفِ گروهی به همان مسیر می‌رود'
have src/views/chat.js 'confirm: t(' 'chat.js: دکمهٔ تأیید ترجمه‌پذیر است'
# فقط صدا زدنِ واقعی مهم است، نه نامش در توضیحاتِ کد.
if grep -vE '^\s*(//|\*|/\*)' src/views/chat.js | grep -q 'window\.confirm('; then
  bad "window.confirmِ سیستمی باقی مانده"
else
  ok "هیچ window.confirmِ سیستمی صدا زده نمی‌شود"
fi

echo "── مورد ۳: پرسش پیش از پاسخ"
have src-tauri/src/ai_session.rs 'fn append_user_message' 'ai_session: نشاندنِ همگامِ حبابِ کاربر'
have src-tauri/src/ai_session.rs 'fn take_failed_prompt' 'ai_session: حذفِ همگامِ حبابِ قرمز پیش از تلاش مجدد'
# ترتیب در ai_send_chat: append باید پیش از spawn_blocking باشد.
if python3 - <<'PY'
import re, sys
s = open('src-tauri/src/main.rs', encoding='utf-8').read()
body = s[s.index('fn ai_send_chat'):]
body = body[:body.index('\n}\n')]
a, p = body.find('append_user_message'), body.find('spawn_blocking')
sys.exit(0 if (a != -1 and p != -1 and a < p) else 1)
PY
then ok "main.rs: در ai_send_chat حباب پیش از رفتن به شبکه نشانده می‌شود"
else bad "main.rs: ترتیبِ ai_send_chat درست نیست"; fi

echo "── اشکالِ گزارش‌شده: «اعمال شد» ولی تنظیم عوض نمی‌شد"
have src-tauri/src/main.rs 'fn publish_profile' 'main.rs: اعلامِ پروفایلی که Rust نوشته'
have src-tauri/src/main.rs 'aether://profile' 'main.rs: نامِ رخداد'
# هر دو مسیرِ نوشتنِ هوش مصنوعی باید اعلام کنند، وگرنه همان اشکال در مسیرِ دیگر می‌ماند.
for f in ai_apply_changes ai_advise; do
  if python3 - "$f" <<'PY'
import re, sys
s = open('src-tauri/src/main.rs', encoding='utf-8').read()
name = sys.argv[1]
body = s[s.index(f'fn {name}'):]
body = body[:body.index('\n}\n')]
sys.exit(0 if 'publish_profile' in body else 1)
PY
  then ok "$f پروفایل را اعلام می‌کند"; else bad "$f بی‌اعلام می‌نویسد"; fi
done
have src-tauri/src/main.rs 'chat applied: {}' 'main.rs: لاگ کلید=مقدار را می‌نویسد و نه فقط تعداد'
have src/main.js "listen('aether://profile'" 'main.js: به رخداد گوش می‌دهد'
have src/main.js 'export function applyProfileSnapshot' 'main.js: جایگزینیِ کپیِ رابط'
# قلبِ اصلاح: saveProfile باید **اول بخواند**. الگو روی متنِ خودِ تابع سنجیده
# می‌شود تا اگر کسی روزی به کپیِ محلی برگردد، همین‌جا بشکند.
if python3 - <<'PY'
import sys
s = open('src/main.js', encoding='utf-8').read()
body = s[s.index('export async function saveProfile'):]
body = body[:body.index("\n}\n")]
read, write = body.find("invoke('get_profile')"), body.find("invoke('set_profile'")
sys.exit(0 if (read != -1 and write != -1 and read < write) else 1)
PY
then ok "saveProfile اول می‌خواند بعد می‌نویسد"; else bad "saveProfile روی کپیِ محلی می‌نویسد"; fi
[ -f tests/profile-sync.mjs ] && ok "tests/profile-sync.mjs هست" || bad "tests/profile-sync.mjs غایب"
grep -q "profile-sync" package.json && ok "در npm test اجرا می‌شود" || bad "در npm test نیست"

echo "── ردمی‌ها: فقط تازه‌های ۱.۲.۴، بی‌تاریخِ آزمون‌وخطا"
for f in README.md README.fa.md .github/release-notes.md UPGRADE-1.2.4.md; do
  if grep -qiE 'اصلاحیهٔ p[0-9]|correction p[0-9]|چرا بیلد قبلی سبز نشد' "$f"; then
    bad "$f: رد پای اصلاحیه‌های ما"
  else
    ok "$f: پاک"
  fi
done
have README.md 'Chat is its own tab' 'README.md: چتِ مستقل نوشته شده'
have README.md 'The assistant can change settings for you' 'README.md: اعمالِ تنظیمات نوشته شده'
have README.fa.md 'هیچ پیامی بی‌تأیید حذف نمی‌شود' 'README.fa.md: تأییدِ حذف نوشته شده'
have .github/release-notes.md 'تازه‌های نسخهٔ ۱.۲.۴' 'release-notes: بخشِ فارسی هست'
# بخشِ نسخهٔ قبلی نباید باشد. اشاره به «آپ‌استریم کارِ توان‌عبوری ۱.۲.۳ را جذب
# کرد» توضیحِ یک تغییرِ همین نسخه است و تاریخِ نسخهٔ قبلی نیست.
# این سنجه با python نوشته می‌شود و نه grep: locale این محیط POSIX است، پس grep
# بایت‌به‌بایت کار می‌کند و یک کلاسِ ارقام فارسی مثل [۰-۳] روی UTF-8 بی‌معنا
# می‌شود — «۴» را هم می‌گیرد، چون بایتِ اولش مشترک است. یک بار همین سنجه به تیترِ
# درستِ «۱.۲.۴» ایراد گرفت.
if python3 - <<'PY'
import re, sys, unicodedata
def older_heading(path):
    for line in open(path, encoding='utf-8'):
        if not re.match(r'^#{2,3} ', line):
            continue
        # ارقام فارسی/عربی را به لاتین برگردان، بعد شمارهٔ نسخه را بخوان.
        norm = ''.join(str(unicodedata.digit(c)) if c.isdigit() else c for c in line)
        for m in re.finditer(r'(\d+)\.(\d+)\.(\d+)', norm):
            if tuple(map(int, m.groups())) < (1, 2, 4):
                return line.strip(), m.group(0)
    return None
hit = older_heading('.github/release-notes.md')
if hit:
    print(f'    {hit[1]} در تیترِ: {hit[0]}')
    sys.exit(1)
sys.exit(0)
PY
then
  ok "release-notes: هیچ بخشی برای نسخه‌های قبلی ندارد"
else
  bad "release-notes: بخشِ نسخهٔ قبلی دارد"
fi
if grep -qE '^#{2,3} .*(1\.2\.[0-3])' UPGRADE-1.2.4.md; then bad "UPGRADE: بخشِ نسخهٔ قبلی"; else ok "UPGRADE-1.2.4.md: فقط همین نسخه"; fi

echo "── ساختار"
for f in src/i18n.js src/ai.js src/views/chat.js src/styles/chat.css tests/chat-smoke.mjs UPGRADE-1.2.4.md; do
  [ -f "$f" ] && ok "$f هست" || bad "$f غایب"
done
[ -d node_modules ] && bad "node_modules داخل بسته است" || ok "بی‌node_modules"
[ -d src-tauri/target ] && bad "target داخل بسته است" || ok "بی‌target"

echo
echo "── و همان فرمان‌های CI روی خودِ بستهٔ استخراج‌شده"
cp -r /home/claude/w/Aether_Desktop-main/node_modules ./node_modules
if out=$(npm test 2>&1); then ok "npm test ($(echo "$out" | grep -c '^  ok') سنجه)"; else bad "npm test"; echo "$out" | tail -15; fi
if out=$(bash scripts/preflight-local.sh 2>&1); then ok "$(echo "$out" | tail -1)"; else bad "preflight"; echo "$out" | tail -15; fi
rm -rf node_modules

echo
[ $fail -eq 0 ] && echo "VERIFY OK" || echo "VERIFY FAILED"
exit $fail
