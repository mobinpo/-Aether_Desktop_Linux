## Aether Desktop 1.1.0

<div dir="rtl">

### 🇮🇷 تازه‌های نسخهٔ ۱.۱.۰ — هم‌ترازی با هستهٔ ۱.۵.۰

این نسخه هستهٔ برنامه را به **Aether Core 1.5.0** می‌رساند. بررسی انتشار بالادست نشان داد
۱.۵.۰ **فقط یک بروزرسانی هسته نیست**: سه قابلیت کاربرمحور تازه دارد که به رابط کاربری نیاز
داشتند و هر سه در این بیلد پیاده شده‌اند.

**🆕 Zero Trust — واپ سازمانی (WARP for organizations)**
- اتصال به‌عنوان یک دستگاه مدیریت‌شدهٔ سازمان Cloudflare Zero Trust، به‌جای کاربر ناشناس WARP
- سه روش ورود، هر سه در تنظیمات پیشرفته: **کد ایمیلی** (کد یک‌بارمصرف به صندوق ایمیل)،
  **توکن سرویس** (شناسه + راز، برای ماشین‌های بدون تعامل) و **توکن دسترسی** (JWT آماده)
- یک هویت تیمی بین پروتکل‌ها مشترک است؛ جابه‌جایی بین MASQUE و WireGuard ورود دوباره نمی‌خواهد
- گزینهٔ **پروکسی Gateway** برای عبور HTTP/HTTPS از دروازهٔ سازمان — پیش‌فرض **خاموش**،
  چون یک هاپ اضافه می‌کند و مرور شما را لاگ می‌کند (همان تصمیم محافظه‌کارانهٔ خودِ هسته)

**🆕 قوانین مسیریابی (به سبک قواعد Xray)**
- **مقصدهای مسدود**: این اتصال‌ها کاملاً رد می‌شوند
- **مقصدهای مستقیم**: از تونل عبور نمی‌کنند و از اینترنت واقعی شما می‌روند — دقیقاً چیزی که
  برای اپ‌های بانکی، سرویس‌های شبکهٔ محلی و سایت‌های داخلی لازم است
- قواعد روی دامنه، آی‌پی، CIDR و پورت کار می‌کنند؛ هر خط یک قاعده

**🆕 انتخاب DNS داخل تونل**
- تعیین حل‌کننده‌های نامی که داخل تونل استفاده می‌شوند؛ خالی = پیش‌فرض موتور

**بهبودهای امنیتی و پایداری که از هستهٔ ۱.۵.۰ به ارث می‌رسد**
- رفع نشتی مهم: رلهٔ UDP در SOCKS5 دیگر دیتاگرام را از هر مبدأیی نمی‌پذیرد و به همتای
  بازکنندهٔ اتصال قفل می‌شود
- کتابخانهٔ quiche به ۰.۲۹.۳ رسید (سه رفع امنیتی بالادست در صف رخداد مسیر، حساب‌داری QPACK و
  سقف priority-update)
- نسخهٔ h2 دقیق پین شد تا بیلد انتشار پیاده‌سازی HTTP/2 آزمایش‌نشده برندارد
- رفع افتادن بی‌صدای WARP-in-WARP (گول) بعد از یک‌دو ساعت، و رفع نشت netstack در اتصال مجدد
- ترتیب اسکن نقاط اتصال طبق مستندات Cloudflare اصلاح شد (پورت ۲۴۰۸ و رنج ۱۶۲.۱۵۹.۱۹۷.۰/۲۴ اول)

**سخت‌سازی امنیتی خودِ نسخهٔ ویندوز (در همین بیلد)**
- اسرار Zero Trust (راز توکن سرویس و JWT) **هرگز روی دیسک نوشته نمی‌شوند** — نه در
  `profile.json` و نه هیچ جای دیگر؛ فقط برای طول عمر فرآیند در حافظه می‌مانند
- این مقادیر «فقط-نوشتنی» هستند: بک‌اند هیچ‌وقت آن‌ها را به رابط کاربری برنمی‌گرداند و
  رابط کاربری بلافاصله پس از ذخیره فیلد را از DOM پاک می‌کند
- مقدار فلگ‌های محرمانه در لاگ ماندگار **ماسک** می‌شود (همان قاعدهٔ ماسک آی‌پی خروجی)
- «بازنشانی به تنظیمات پیش‌فرض» اکنون اسرارِ در-حافظه را هم واقعاً پاک می‌کند
- **گارد نسخهٔ هسته**: فلگ‌های ۱.۵.۰ فقط به هسته‌ای فرستاده می‌شوند که آن‌ها را می‌فهمد؛
  اگر نسخهٔ قدیمی‌تری پین شده باشد، این بخش‌ها در رابط کاربری غیرفعال و علتش توضیح داده
  می‌شود. قاعدهٔ همیشگی مخزن پابرجاست: ارتقای خودکار هسته هرگز یک انتشار را نمی‌شکند

**رابط کاربری**
- بخش‌های تازه کاملاً دوزبانه‌اند (English + فارسی) و راست‌به‌چپ را کامل رعایت می‌کنند

### کدام فایل را دانلود کنم؟

| فایل | توضیح |
|---|---|
| `Aether-Setup-1.1.0-x64.exe` | ویندوز ۶۴بیتی — نصب معمول (توصیه‌شده) |
| `Aether-Setup-1.1.0-x86.exe` | ویندوز ۳۲بیتی |
| `Aether-Portable-1.1.0-x64.zip` | بدون نصب، ۶۴بیتی |
| `Aether-Portable-1.1.0-x86.zip` | بدون نصب، ۳۲بیتی |
| `SHA256SUMS.txt` | برای راستی‌آزمایی سلامت فایل‌ها |

**پیش‌نیاز:** ویندوز ۱۰ نسخهٔ ۱۸۰۹ یا بالاتر. برای برقراری تونل، دسترسی مدیر (Administrator) لازم است.

</div>

---

### 🇬🇧 What's new in 1.1.0 — parity with engine core 1.5.0

This release moves the bundled engine to **Aether Core 1.5.0**. Reviewing the upstream release
showed 1.5.0 is **not only an engine bump**: it ships three user-facing features that needed UI,
and all three are implemented in this build.

**🆕 Zero Trust (WARP for organizations)**
- Connect as a managed device of a Cloudflare Zero Trust organization instead of an anonymous
  consumer WARP device. Works on both MASQUE and WireGuard.
- Three sign-in methods, all in Advanced settings: **email code** (one-time code to your mailbox),
  **service token** (ID + secret, for headless machines) and **access token** (a JWT you already have)
- One team identity is shared across protocols, so switching transport does not force a second sign-in
- **Gateway proxy** toggle routes HTTP/HTTPS through the organization's Gateway. Off by default:
  it adds a hop inside the tunnel and logs your browsing (matching the engine's own decision).

**🆕 Routing rules (in the style of Xray's routing)**
- **Blocked destinations**: the connection is refused outright
- **Direct destinations**: sent out of your real interface instead of the tunnel — what you want for
  banking apps, LAN services and domestic sites that reject foreign addresses
- Rules match on domain, IP, CIDR and port; one rule per line

**🆕 In-tunnel DNS selection**
- Choose the resolvers used inside the tunnel; empty means engine defaults

**Security and reliability inherited from core 1.5.0**
- Important leak fix: the SOCKS5 UDP relay no longer accepts datagrams from any source and is
  pinned to the peer that opened the control connection
- Vendored quiche updated to 0.29.3 (bounded path-event queue, QPACK field overhead accounting,
  enforced maximum priority-update size)
- h2 pinned to an exact version so a release build cannot pick up an untested HTTP/2 implementation
- Fixed WARP-in-WARP (gool) dropping silently without reconnecting, plus netstack leaks on reconnect
- Endpoint scanning now follows Cloudflare's documented port and range order

**Windows-edition hardening added in this build**
- Zero Trust secrets (service-token secret and JWT) are **never written to disk** — not in
  `profile.json`, nowhere; they live in memory for the process lifetime only
- Those values are write-only: the backend never returns them to the UI, and the UI clears the
  field from the DOM immediately after saving
- Sensitive flag values are **masked** in the persistent log (same rule as the masked exit IP)
- "Reset to defaults" now genuinely clears the in-memory secrets too
- **Core version gate**: 1.5.0 flags are only passed to an engine that understands them. If an
  older core is pinned, those sections are disabled in the UI with an explanation. The standing
  repository rule holds: an automatic core upgrade can never break a release.

**Interface**
- The new sections are fully bilingual (English + فارسی) with complete right-to-left support

### Which file do I download?

| File | Description |
|---|---|
| `Aether-Setup-1.1.0-x64.exe` | Windows 64-bit — normal install (recommended) |
| `Aether-Setup-1.1.0-x86.exe` | Windows 32-bit |
| `Aether-Portable-1.1.0-x64.zip` | No install, 64-bit |
| `Aether-Portable-1.1.0-x86.zip` | No install, 32-bit |
| `SHA256SUMS.txt` | For verifying file integrity |

**Requirements:** Windows 10 build 1809 or newer. Establishing the tunnel requires Administrator rights.

---

<div dir="rtl">

### 🛡️ ممیزی امنیتی نسخهٔ ۱.۱.۰ — امتیاز ۹۳ از ۱۰۰ (v10)

ممیزی کامل هشت‌محوری روی نسخهٔ موبایل/مشترک و لایهٔ ویندوز (اسرار هاردکد، رمزنگاری و پروتکل،
نشت داده، ذخیره‌سازی محلی، مجوزها و مانیفست، لاگ، مرز اعتماد رابط کاربری، و زنجیرهٔ تأمین):

- ✅ هیچ کلید API، توکن یا رمز هاردکدشده‌ای در سورس نیست؛ هویت WARP در زمان اجرا ساخته می‌شود.
- ✅ اعتبارسنجی TLS با پین‌کردن SPKI روی MASQUE (هر دو مسیر HTTP/2 و HTTP/3) — MitM روی کانال
  کنترل عملاً ممکن نیست.
- 🆕 **رفع شد در ۱.۵.۰:** رلهٔ UDP در SOCKS5 دیگر از هر مبدأیی دیتاگرام نمی‌پذیرد. این جدی‌ترین
  یافتهٔ ممیزی قبلی در سطح هسته بود و بالادست آن را بست.
- 🆕 **اسرار Zero Trust روی دیسک نوشته نمی‌شوند.** راز توکن سرویس و JWT فقط در حافظه‌اند،
  «فقط-نوشتنی»اند و در لاگ ماسک می‌شوند. قابلیت جدید هیچ سطح حملهٔ ماندگاری اضافه نکرد.
- 🆕 **گارد نسخهٔ هسته** مانع فرستادن فلگ ناشناخته به هستهٔ قدیمی‌تر می‌شود — جلوگیری از یک کلاس
  کامل خطای «موتور در میلی‌ثانیهٔ اول می‌میرد».
- 🆕 **پیش‌فرض محافظه‌کارانهٔ Gateway:** خاموش است. روشن‌بودنش مرور کاربر را برای سازمان لاگ
  می‌کند، پس تصمیم آگاهانه به کاربر واگذار شده و در رابط کاربری صریحاً هشدار داده می‌شود.
- ✅ تونل واقعی سطح سیستم (Wintun): DNS از داخل تونل عبور می‌کند و IPv6 طبق انتخاب کاربر مدیریت می‌شود.
- ✅ پل اشتراک LAN فقط اتصال‌های loopback / شبکهٔ خصوصی / link-local را می‌پذیرد (فیلتر مبدأ).
- ✅ مانیفست اندروید حداقلی: `allowBackup=false`، بدون `debuggable`، سرویس VPN از بیرون در دسترس نیست.
- ✅ ترافیک cleartext در اندروید کاملاً مسدود است (`network_security_config`).
- ✅ آی‌پی خروجی در لاگ ماندگار ماسک می‌شود (`1.2.3.xxx`)؛ نمایش کامل فقط در رابط کاربری. چرخش ۵۱۲KiB پابرجاست.
- ✅ جستار موقعیت جغرافیایی اول از مسیر TLS روی ۴۴۳ می‌رود و HTTP ساده فقط گزینهٔ پشتیبان است.
- ✅ زنجیرهٔ تأمین: `quiche 0.29.3` و `h2` با نسخهٔ دقیق پین شده‌اند؛ انتشار با CLI رسمی گیت‌هاب
  انجام می‌شود و به اکشن شخص ثالث وابسته نیست.
- ⚠️ متوسط: فایل‌های هویت WARP همچنان به‌صورت متن ساده در پوشهٔ کاری‌اند. قفل ACL آزمایشی در v9
  حذف شد چون دسترسی خود موتور را هم می‌بست؛ **رمزگذاری DPAPI در نقشهٔ راه است** و همچنان
  تنها یافتهٔ متوسط باقی‌مانده است.
- ⚠️ کم: SNI برای نقطهٔ MASQUE به‌صورت cleartext می‌رود (سرور مقصد ECH را نمی‌پذیرد — محدودیت
  سمت سرور، نه سمت ما).
- ⚠️ کم: پروکسی سیستمی ویندوز در `HKCU` نوشته می‌شود (بدون نیاز به Administrator)؛ یک بدافزار
  در همان نشست کاربر می‌تواند آن را بازنویسی کند. این محدودیت ذاتی طراحی پروکسی ویندوز است و
  در حالت Wintun موضوعیت ندارد.

**تغییر امتیاز نسبت به ۱.۰.۰:** ۹۰ → ۹۳. دلیل: بسته‌شدن نشتی رلهٔ UDP در هسته، پین‌شدن
وابستگی‌های حساس، و اضافه‌شدن قابلیت‌های تازه **بدون** ایجاد ذخیره‌سازی محرمانهٔ ماندگار.

</div>

### 🛡️ Security audit — score 93/100 (v10)

Full eight-area audit of the shared/mobile core and the Windows layer (hardcoded secrets,
cryptography & protocols, data-leak risks, local storage, permissions & manifest, logging,
UI trust boundary, supply chain):

- ✅ No hardcoded API keys, tokens or passwords anywhere; WARP identities are generated at runtime.
- ✅ TLS validated with SPKI certificate pinning on MASQUE (both HTTP/2 and HTTP/3); MitM on the
  control channel is not feasible.
- 🆕 **Fixed in 1.5.0:** the SOCKS5 UDP relay no longer accepts datagrams from arbitrary sources.
  This was the most serious core-level finding of the previous audit and upstream closed it.
- 🆕 **Zero Trust secrets are never persisted.** The service-token secret and the JWT live in
  memory only, are write-only towards the UI, and are masked in logs. The new feature added no
  persistent attack surface.
- 🆕 **Core version gate** prevents passing unknown flags to an older engine, eliminating a whole
  class of "engine dies in the first millisecond" failures.
- 🆕 **Conservative Gateway default:** off. Enabling it logs the user's browsing for the
  organization, so the decision stays explicit and is spelled out in the UI.
- ✅ Real system-level tunnel (Wintun): DNS resolves inside the tunnel, IPv6 handled per the
  user's stack selection.
- ✅ The LAN share bridge only accepts loopback / private / link-local peers (source filter).
- ✅ Minimal Android manifest: `allowBackup=false`, non-debuggable, VpnService not exported.
- ✅ All cleartext HTTP is blocked on Android by the network security config.
- ✅ The persistent log stores the exit IP masked (`1.2.3.xxx`); the full IP is shown only in the
  UI. Automatic 512 KiB rotation unchanged.
- ✅ Geolocation lookups try TLS on 443 first; plain HTTP is only a fallback.
- ✅ Supply chain: `quiche 0.29.3` and `h2` pinned to exact versions; releases are published with
  the official GitHub CLI, with no third-party action in the trust path.
- ⚠️ Medium: WARP identity files are still plaintext in the working directory. The experimental
  ACL lock was removed in v9 because it also blocked the engine's own access; **DPAPI encryption
  remains on the roadmap** and this stays the only open medium finding.
- ⚠️ Low: the SNI for the MASQUE endpoint is sent in cleartext (the endpoint does not accept ECH,
  a server-side limitation).
- ⚠️ Low: the Windows system proxy is written under `HKCU` (no Administrator needed), so malware
  running as the same user could overwrite it. This is inherent to the Windows proxy design and
  does not apply in Wintun mode.

**Score change vs 1.0.0:** 90 → 93, driven by the upstream UDP-relay fix, pinned sensitive
dependencies, and shipping the new features **without** introducing persistent secret storage.
