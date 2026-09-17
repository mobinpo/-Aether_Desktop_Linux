# Aether Desktop 1.2.5

## What's new

**Upgrade notice:** 1.2.5 brings the mobile edition's **Tor support** to Windows — four new transport backends, bridges, and a bootstrap percentage in your own language — bundles **Aether Core 2.0.0**, and closes a cleartext lookup found while auditing this upgrade. Saved profiles load unchanged, the default backend is still `Aether`, and the desktop version is `1.2.5`.

### New in this release

**Tor, ported from Aether Mobile 1.3.0.** Four backends in Advanced → *Backend*, each mapping onto a core flag rather than onto logic of our own: **Tor** alone (`--tor-only`, exit is Tor, nothing under it), **Aether → Tor** (`--tor`, Tor is dialled through the tunnel), **Tor → Aether** (`--tor-reverse`, Tor goes first), and **Tor → Psiphon** (Tor first, exit from an ordinary hosting address).

**MASQUE×2, ported from Aether Mobile 1.3.0.** The protocol selector also exposes `MASQUE×2` (`--mim`): two MASQUE hops, with the inner hop reached through the outer hop. It is gated on Core 2.0.0, so older engines never receive the flag. It remains a manual protocol, as in the mobile Smart Auto ladder; Smart Auto does not add a second MASQUE hop to a network strategy that has not selected it.

**Bridges, only where they mean something.** `Auto`, `Always` and `Off` — enabled in **Tor** and **Tor → Psiphon** only. In *Aether → Tor* the network Tor sees is not the operator's, so a bridge changes nothing and the control is greyed out with the reason on screen. *Tor → Aether* carries a limitation that is not ours to fix: Tor must reach the network before the tunnel exists, so its bridge traffic crosses the operator's network.

**The bootstrap percentage, in your language.** Progress is read from the engine log and a stall is told apart from slow progress by a time budget rather than by a guess. The percentage travels as a separate numeric field and the sentence is built by the translation layer, so the Persian interface shows a Persian line.

**A failed Tor attempt names the cause that fits that attempt.** Three situations that used to share one sentence are now told apart: Tor stalling *inside* the tunnel in *Aether → Tor* (where bridges would change nothing), Tor facing the operator's network *with* a pluggable transport, and Tor facing it *without* one — where only plain bridges remain, which a network that filters Tor usually blocks too, so the message points at the *Aether → Tor* mode instead. Each message carries the percentage Tor stopped at, in both languages.

**A stalled Tor attempt gives up on a budget instead of retrying forever.** In the chained mode the attempt had neither a deadline nor an attempt limit, so a connection that was never going to complete kept the app in "Connecting…". Every Tor shape now has a bounded budget and hands over to the next strategy.

**The pluggable transport ships with the app — and a published build cannot go out without it.** `obfs4` needs a lyrebird binary, so the build compiles lyrebird 0.6.1 for Windows and bundles it in `engine/pt/`. A failure there still does not break a pull-request build, but a **published** build is gated: if `engine/pt/lyrebird.exe` is missing or implausibly small, the release stops rather than shipping an empty transport directory.

**Defaults matched to Aether Mobile 1.3.0.** Scan mode starts at **Balanced** and the reconnect limit at **5 attempts**, as on mobile, while the Smart Auto ladder keeps scanning each rung in **Turbo**.

**Settings the engine changes appear immediately.** When a connection attempt rewrites the profile, every profile-dependent panel is rebuilt from the new values — deferred while you are typing in a field.

**A readiness gate sits between connecting and the self-test.** The post-connect self-test waits until the network is usable, so a Tor stage still bootstrapping is never judged on a half-built connection. Tor modes are also kept off the Smart Auto ladder, which is there to find the *fastest* transport and would read a healthy 90-second bootstrap as "slow, drop it".

**Aether Core 1.9.0 → 2.0.0.** `CoreCaps::for_version` compares with `>=`, so Tor capabilities switch on with 2.0.0 and no version is hardcoded; with an older core placed next to the app the Tor backends are greyed out with a reason rather than selectable and silently broken.

**Upgrade note:** saved profiles load untouched, the default backend is unchanged, and the Tor backends are **not** writable from the AI chat — `backend` was already outside the chat allowlist and stayed there.

### Security audit summary

| Area | Result |
|---|---|
| **Fixed: cleartext exit-country lookup** | A plain `http://ip-api.com` request — no TLS, outside the tunnel — put a domain name on the wire and left the answer forgeable. It now happens only from inside an established tunnel; with no tunnel nothing is sent |
| Released artefact | A published build with a missing or implausibly small lyrebird binary fails the release instead of shipping an empty `engine/pt/` |
| Advice given on failure | Bridge advice is only given where bridges can act |
| Bounded attempts | No Tor shape retries without a deadline |
| Capability gate | Tor backends are disabled with a stated reason on cores older than 2.0.0 |
| AI boundary | `backend`, and with it every Tor mode, stays outside the chat's writable allowlist |
| Leak protection | Mandatory DNS, IPv6 and WebRTC protection unchanged |

**Overall audit score: 83 / 100.** Ten areas, each weighted, each scored against
evidence that was actually produced rather than assumed. The 17 missing points are
almost entirely one thing: the Windows-only part of the pipeline cannot be exercised
on a Linux host, so it is scored zero instead of being claimed.

| # | Area | Weight | Score | Evidence |
|---|---|---|---|---|
| 1 | Compile and format integrity | 12 | 11 | `cargo check --all-targets` green on `x86_64-pc-windows-gnu` and `i686-pc-windows-gnu`; `cargo fmt --check` clean; a guard proves all 69 `.rs` files parse. 44 clippy findings remain, no errors — 30 are unused API surface, 14 are style |
| 2 | Automated tests | 12 | 8 | 12 test files green (UI, chat, profile sync, glow direction, protocol options); the desktop layout test now measures in a real Chromium at two window sizes in both languages. `cargo test` runs only on Windows, so it is unproven here |
| 3 | Regression guards and negative controls | 12 | 12 | 10 positive guards, 5 negative-control suites; every guard is proven to turn red on a deliberate mutation, and each mutation verifies that it actually applied |
| 4 | CI pipeline | 10 | 9 | 24 preflight steps, extracted from the workflow and executed verbatim, all exit 0. Release publishing is gated by the account spending limit, which no code change can lift |
| 5 | Package integrity | 10 | 10 | `verify-package.sh` passes 15 cases plus structure on the extracted archive; the archive carries no build artefacts, no `node_modules`, no `target/` |
| 6 | Memory safety and error paths | 10 | 8 | 5 `unsafe` blocks, all Win32 FFI or wintun loading; no `panic!`, `todo!` or `unreachable!` anywhere; 39 `unwrap()` remain in live code, which is the honest deduction |
| 7 | Secrets and keys | 8 | 8 | No hardcoded credential in Rust or JavaScript; the only match is a fixture string inside a test; the Gemini key stays DPAPI-sealed and upstream credentials are not persisted |
| 8 | Network and leak hygiene | 10 | 9 | No cleartext `http://` target left in live code; DNS, IPv6 and WebRTC protection mandatory; country lookup answered from a local GeoIP file before any network refinement |
| 9 | Durability of core patches | 8 | 8 | All 11 patched core files are protected across a core upgrade, each with an upstream baseline, and a guard fails the build if a patched file ever falls out of that list |
| 10 | Windows runtime proof | 8 | 0 | MSVC link, `cargo test` on Windows, `tauri build`, installer, signature and smoke test run only on the Windows runner — none of them can be demonstrated from this host, so no credit is taken |

<div dir="rtl" align="right" markdown="1">

## تازه‌های نسخهٔ ۱.۲.۵

### امکانات جدید

- **تور، پورت‌شده از AetherMobile 1.3.0**: چهار بک‌اند در «پیشرفته ← بک‌اند» — `Tor` تنها (`--tor-only`، خروجی تور، بی هیچ تونلی زیرش)، `Aether → Tor` (`--tor`، تور از داخل تونل شماره‌گیری می‌شود)، `Tor → Aether` (`--tor-reverse`، تور اول است) و `Tor → Psiphon` (تور اول، خروجی از یک آی‌پیِ هاستینگ عادی). هر چهارتا مستقیم روی فلگ‌های خودِ هسته می‌نشینند.
- **MASQUE×2، پورت‌شده از AetherMobile 1.3.0**: گزینهٔ `MASQUE×2` در انتخابگر پروتکل به فلگ `--mim` وصل است و فقط با هستهٔ ۲.۰.۰ ارسال می‌شود؛ در Smart Auto مثل نسخهٔ موبایل به‌طور خودکار وارد نردبان نمی‌شود و انتخاب دستی باقی می‌ماند.
- **پل‌ها فقط جایی که معنا دارند**: `Auto`، `Always` و `Off`، فعال در `Tor` و `Tor → Psiphon`. در `Aether → Tor` شبکه‌ای که تور می‌بیند شبکهٔ اپراتور نیست، پس پل بی‌اثر است و کنترل با ذکرِ دلیل خاکستری می‌شود. در `Tor → Aether` محدودیتی هست که دستِ ما نیست: تور پیش از بالا آمدنِ تونل به شبکه می‌رسد، پس ترافیکِ پل از شبکهٔ اپراتور می‌گذرد.
- **درصدِ راه‌اندازی، به زبانِ خودتان**: پیشرفت از لاگِ موتور خوانده می‌شود و «گیر کردن» با یک بودجهٔ زمانی از «کند بودن» تشخیص داده می‌شود. درصد یک فیلدِ عددیِ جداست و جمله در لایهٔ ترجمه ساخته می‌شود.
- **شکستِ تور علتی را می‌گوید که به همان تلاش می‌خورد**: گیر کردنِ تور *داخلِ* تونل در `Aether → Tor` (جایی که پل بی‌اثر است)، رویارویی **با** ترابرِ افزودنی، و رویارویی **بی** ترابر — که آن‌وقت فقط پلِ ساده می‌ماند و شبکهٔ فیلترکنندهٔ تور معمولاً آن را هم می‌بندد، پس پیام به حالتِ `Aether → Tor` راهنمایی می‌کند. هر سه پیام درصدِ توقف را دارند، در هر دو زبان.
- **تلاشِ گیرکردهٔ تور با بودجه تسلیم می‌شود، نه بی‌پایان**: در حالتِ زنجیره‌ای تلاش نه مهلت داشت و نه سقفِ تعداد، پس برنامه در «در حال اتصال…» می‌ماند. حالا هر شکلِ تور بودجهٔ محدود دارد و به استراتژیِ بعدی واگذار می‌کند.
- **ترابرِ افزودنی همراهِ برنامه می‌آید — و بستهٔ منتشرشده بی آن بیرون نمی‌رود**: lyrebird ۰.۶.۱ برای ویندوز بیلد و در `engine/pt/` بسته می‌شود. شکستِ آن مرحله بیلدِ pull-request را نمی‌شکند، ولی بستهٔ **منتشرشدنی** دروازه دارد: اگر `engine/pt/lyrebird.exe` نباشد یا اندازه‌اش باورنکردنی کوچک باشد، انتشار متوقف می‌شود.
- **پیش‌فرض‌ها همان AetherMobile 1.3.0**: حالتِ اسکن **متعادل** و سقفِ اتصالِ مجدد **۵ تلاش**، و نردبانِ Smart Auto هر پله را در **توربو** اسکن می‌کند.
- **تنظیماتی که موتور عوض می‌کند فوراً روی صفحه می‌آید**: هر پنلِ پروفایل‌محور از مقادیرِ تازه بازساخته می‌شود — و اگر وسطِ تایپ باشید، بازسازی عقب می‌افتد.
- **یک دروازهٔ آمادگی میانِ اتصال و خودآزمون**: خودآزمونِ پس از اتصال تا قابلِ استفاده شدنِ شبکه صبر می‌کند، پس مرحلهٔ توری که در حالِ راه‌اندازی است داوری نمی‌شود. حالت‌های تور هم بیرونِ نردبانِ Smart Auto می‌مانند.
- **هستهٔ ۱.۹.۰ → ۲.۰.۰**: با هستهٔ قدیمی‌تر، بک‌اندهای تور خاکستری می‌شوند و دلیلش گفته می‌شود — نه اینکه انتخاب شوند و بی‌صدا شکست بخورند.

**نکتهٔ ارتقا:** پروفایل‌های ذخیره‌شده دست‌نخورده بارگذاری می‌شوند، بک‌اندِ پیش‌فرض همان `Aether` است، و بک‌اندهای تور از دلِ چتِ دستیار **نوشتنی نیستند**.

### خلاصهٔ ممیزی امنیتی

| حوزه | نتیجه |
|---|---|
| **اصلاح‌شده: پرسشِ بی‌رمزِ کشورِ خروج** | درخواستِ `http://ip-api.com` بی TLS و بیرون از تونل، هم نامِ دامنه را روی سیم می‌گذاشت و هم پاسخ را دستکاری‌پذیر می‌کرد. حالا فقط از داخلِ تونلِ برقرار می‌رود |
| بستهٔ منتشرشده | lyrebirdِ غایب یا باورنکردنی کوچک، انتشار را رد می‌کند |
| توصیه در شکست | پیشنهادِ پل فقط جایی داده می‌شود که پل می‌تواند کاری کند |
| محدودیتِ تلاش | هیچ شکلی از تور بی مهلت تلاش نمی‌کند |
| دروازهٔ قابلیت | روی هستهٔ قدیمی‌تر از ۲.۰.۰، بک‌اندهای تور با ذکرِ دلیل غیرفعال‌اند |
| مرزِ هوش مصنوعی | `backend` و همهٔ حالت‌های تور بیرونِ فهرستِ نوشتنیِ چت می‌مانند |
| محافظتِ نشت | محافظتِ اجباریِ DNS، IPv6 و WebRTC بی تغییر |

**نمرهٔ کلیِ ممیزی: ۸۳ از ۱۰۰.** ده حوزه، هر یک با وزنِ خودش، و هر نمره روی
شاهدی که واقعاً تولید شد نشسته است، نه روی فرض. ۱۷ نمرهٔ کم‌آمده تقریباً همه از
یک چیز است: بخشِ ویندوزیِ خط روی میزبانِ لینوکس اجرا نمی‌شود، پس صفر گرفته
است تا ادعا نشود.

| # | حوزه | وزن | نمره | شاهد |
|---|---|---|---|---|
| ۱ | صحتِ کامپایل و آرایش | ۱۲ | ۱۱ | `cargo check --all-targets` روی `x86_64-pc-windows-gnu` و `i686-pc-windows-gnu` سبز؛ `cargo fmt --check` تمیز؛ گاردی ثابت می‌کند هر ۶۹ فایل `.rs` پارس می‌شود. ۴۴ یافتهٔ clippy مانده، بی هیچ خطا — ۳۰ موردش سطحِ APIِ استفاده‌نشده و ۱۴ موردش سبکی |
| ۲ | آزمون‌های خودکار | ۱۲ | ۸ | ۱۲ فایلِ تست سبز (رابط، چت، همگامیِ پروفایل، جهتِ درخشش، گزینه‌های پروتکل)؛ آزمونِ چیدمانِ دسکتاپ حالا در یک کرومیومِ واقعی، در دو اندازهٔ پنجره و هر دو زبان اندازه می‌گیرد. `cargo test` تنها روی ویندوز اجرا می‌شود، پس اینجا اثبات‌نشده است |
| ۳ | گاردهای پس‌رفت و کنترل‌های منفی | ۱۲ | ۱۲ | ۱۰ گاردِ مثبت و ۵ مجموعهٔ کنترلِ منفی؛ برای هر گارد ثابت شده که با یک جهشِ عمدی قرمز می‌شود، و هر جهش خودش تأیید می‌کند که واقعاً اعمال شد |
| ۴ | خطِ CI | ۱۰ | ۹ | ۲۴ گامِ preflight، عیناً از ورک‌فلو بیرون کشیده و اجرا شد و همه با کدِ ۰ تمام شدند. انتشار به سقفِ خرجِ حساب بند است، که هیچ تغییرِ کدی آن را باز نمی‌کند |
| ۵ | یکپارچگیِ بسته | ۱۰ | ۱۰ | `verify-package.sh` روی بستهٔ استخراج‌شده ۱۵ مورد به‌علاوهٔ ساختار را می‌گذراند؛ بسته هیچ آرتیفکتِ بیلد، `node_modules` و `target/` ندارد |
| ۶ | ایمنیِ حافظه و مسیرهای خطا | ۱۰ | ۸ | ۵ بلوکِ `unsafe`، همه FFIِ Win32 یا بارگذاریِ wintun؛ هیچ `panic!`، `todo!` و `unreachable!` در کد نیست؛ ۳۹ `unwrap()` در کدِ زنده مانده، و همین کسرِ صادقانه است |
| ۷ | رمزها و کلیدها | ۸ | ۸ | هیچ اعتبارنامهٔ سخت‌کدی در Rust و جاوااسکریپت نیست؛ تنها تطبیق، یک رشتهٔ نمونه داخلِ یک تست است؛ کلیدِ Gemini با DPAPI مهر می‌ماند و اعتبارنامه‌های آپ‌استریم ذخیره نمی‌شوند |
| ۸ | بهداشتِ شبکه و نشت | ۱۰ | ۹ | هیچ هدفِ `http://` بی‌رمزی در کدِ زنده نمانده؛ محافظتِ DNS و IPv6 و WebRTC اجباری است؛ پرسشِ کشور پیش از هر پالایشِ شبکه‌ای از فایلِ GeoIPِ محلی پاسخ می‌گیرد |
| ۹ | دوامِ پچ‌های هسته | ۸ | ۸ | هر ۱۱ فایلِ پچ‌خوردهٔ هسته در ارتقای هسته محافظت می‌شوند، هر یک با مبنای آپ‌استریم، و گاردی بیلد را می‌شکند اگر فایلی از آن فهرست بیفتد |
| ۱۰ | اثباتِ اجرا روی ویندوز | ۸ | ۰ | لینکِ MSVC، `cargo test` روی ویندوز، `tauri build`، اینستالر، امضا و اسموک‌تست تنها روی رانرِ ویندوز اجرا می‌شوند — هیچ‌یک از این میزبان نشان‌دادنی نیست، پس نمره‌ای برداشته نشده |

</div>

<div dir="rtl">

## درود دوستان عزیز 🌹

در نسخهٔ 1.2.5 ویندوز، هستهٔ اِتِر به نسخهٔ 2.0.0 به‌روزرسانی شد. قابلیت اتصال Tor (تور) توسط سازندهٔ هستهٔ اِتِر، CluvexStudio، به خودِ هسته اضافه شده است و من تنها حالت‌های ترکیبی مختلف اتصال Tor با سایر کانکشن‌ها را اضافه کرده‌ام.

> **نکته دربارهٔ حالت Tor:** این حالت امنیت بسیار بالایی دارد، اما سرعت آن پایین است. بنابراین برای دوستانی مناسب است که امنیت برایشان مهم‌تر از سرعت است.

### نتیجهٔ تست‌های من

حالت Tor را روی اینترنت آسیاتک و همراه اول تست کردم؛ روی آسیاتک به‌خوبی متصل می‌شد، اما روی همراه اول اتصال به‌سختی برقرار می‌شد و نوسان زیادی داشت. روی آسیاتک نتیجه به این شکل بود:

| حالت اتصال | نتیجه |
| :--- | :--- |
| Tor (تنها) | متصل شد |
| اِتِر + تور | متصل شد |
| تور + سایفون | متصل شد |
| تور + اِتِر | متصل نشد |

*همه‌ی این اتصال‌ها با تنظیمات پیش‌فرض انجام شد.*

توجه داشته باشید که نتیجه ممکن است برای هر کاربر متفاوت باشد، چون وضعیت DPI از شهری به شهر دیگر، از منطقه‌ای به منطقه‌ی دیگر و حتی از سیم‌کارتی به سیم‌کارت دیگر فرق می‌کند. پس اگر برای شما متصل نشد، حتماً با تنظیمات، حالت‌ها و پروتکل‌های مختلف تست کنید.

### نکتهٔ پایانی — زمان اتصال

اگر برنامه را برای بار اول نصب کرده‌اید یا به نسخهٔ جدید به‌روزرسانی کرده‌اید، اولین اتصال در هر کانکشنی ممکن است تا ۲ دقیقه زمان ببرد؛ پس کمی صبور باشید. در دفعات بعدی معمولاً کمتر از ۱ دقیقه طول می‌کشد. به‌طور کلی، بسته به DPI شبکهٔ شما، این زمان می‌تواند بین ۱ تا ۳ دقیقه متغیر باشد.

* **حالت Tor و حالت‌های ترکیبی Tor:** در نسخهٔ ویندوز، زمان برقراری اتصال بسته به شرایط DPI اپراتور شما ممکن است **۲ الی ۳ دقیقه یا حتی بیشتر** طول بکشد. همچنین در برخی مواقع ممکن است پس از ۲ تا ۳ دقیقه اتصال برقرار نشود که در این صورت باید اتصال را قطع کرده و مجدداً تلاش کنید.

---

## گزارش رفع مشکلات و بهبودها

تعدادی از دوستان از طریق ایمیل و پیام‌ها، مشکلاتی را در رابطه با نسخه دسکتاپ گزارش کرده بودند. از اینکه با دقت این موارد را با ما در میان گذاشتید و به بهبود برنامه کمک می‌کنید، بسیار متشکریم 🙏  
تمامی موارد گزارش‌شده بررسی و برطرف شدند:

1. **ذخیره‌سازی ابعاد پنجره برنامه:**
   * مشکل عدم ذخیره سایز پنجره در مانیتورهایی با رزولوشن‌های مختلف حل شد. اکنون برنامه آخرین اندازه و ابعادی را که تنظیم کرده‌اید به‌خاطر می‌سپارد و در اجراهای بعدی با همان سایز دلخواه باز می‌شود.

2. **بهینه‌سازی مصرف رم (RAM)، پردازنده (CPU) و صدای فن:**
   * مشکل مصرف بیش‌ازحد منابع سیستم که باعث داغ شدن پردازنده و کارکرد مداوم فن سیستم می‌شد کاملاً برطرف شده است. مصرف منابع برنامه اکنون به حداقل رسیده و کاملاً روان و بهینه اجرا می‌شود.

3. **سرعت اتصال و بهبود عملکرد پروتکل‌ها:**
   * تأخیر طولانی‌مدت در اتصال پروتکل‌های نسخه دسکتاپ برطرف شد و زمان برقراری اتصال به‌طور چشمگیری کاهش پیدا کرده است.

4. **اتصال مستقیم و سریع به WireGuard:**
   * اختلال و کندی پروتکل WireGuard در دسکتاپ اصلاح شد. همچنین عملکرد حالت هوشمند (Smart) بهینه‌سازی شده و اکنون مانند نسخه اندروید، در سریع‌ترین زمان ممکن به بهترین پروتکل (از جمله WireGuard) متصل می‌شود.

5. **رفع مشکل عدم اتصال برنامه در تمامی پروتکل‌ها و کانکشن‌ها:**
   * **علت مشکل:** اگر نسخه ویندوز اِتِر روی هیچ کانکشن و پروتکلی متصل نمی‌شود و خطا می‌دهد، علت اصلی آن خاموش بودن **«دیوار آتش ویندوز» (Windows Firewall)** است. برنامه برای محافظت از امنیت شما و جلوگیری از فاش شدن آی‌پی واقعی، در صورت خاموش بودن فایروال عمداً اتصال را برقرار نمی‌کند 🙂
   
   * **راه‌حل سریع (۲ دقیقه):**
     1. کلیدهای **Win + R** را روی کیبورد هم‌زمان بزنید، عبارت `firewall.cpl` را بنویسید و **Enter** بزنید.
     2. از منوی سمت چپ، روی گزینه **Turn Windows Defender Firewall on or off** کلیک کنید.
     3. در هر دو بخش (**Private** و **Public**)، گزینه **Turn on** را انتخاب کرده و دکمه **OK** را بزنید.
     4. اگر آنتی‌ویروس جداگانه (مثل Kaspersky، ESET، Comodo یا ۳۶۰) دارید، فایروال اختصاصی آن را موقتاً خاموش کنید تا با ویندوز تداخل نداشته باشد.
     5. مرورگرهای باز (کروم، اج و...) را کاملاً ببندید، برنامه **Aether** را اجرا کرده و وصل شوید. تمام! ✅

   * **اگر باز هم وصل نشد (راه‌حل کمکی):**
     * در منوی استارت عبارت `cmd` را جستجو کنید، روی **Command Prompt** راست‌کلیک کرده و **Run as administrator** را بزنید. دستور زیر را وارد کرده و Enter بزنید:
     ```text
     netsh advfirewall set allprofiles state on
     ```
     * پس از اجرای دستور، یک بار سیستم را ری‌استارت کرده و دوباره متصل شوید.

   📌 **نکته:** برنامه‌های بهینه‌سازی و افزایش سرعت ویندوز اغلب فایروال را خاموش می‌کنند؛ توصیه می‌شود از آن‌ها استفاده نکنید.  
   🔍 **تست اتصال:** بعد از اتصال، در برنامه به سربرگ **Diagnostics** (عیب‌یابی) بروید و روی **Run test** کلیک کنید؛ سبز بودن تمام گزینه‌ها و نمایش عبارت **PASS** یعنی همه‌چیز مرتب و ایمن است 💚

</div>
