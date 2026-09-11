# Aether Desktop 1.2.4

## What's new

**Upgrade notice:** 1.2.4 brings the mobile edition's **AI assistant** to Windows — including a full chat page whose assistant can *apply* tuning settings for you — rebuilds the entire settings area as the mobile **hub-and-subpage menu**, bundles **Aether Core 1.9.0**, and fixes a setting that never did anything: a pinned **address range** was handed to the engine and read by nobody. Saved profiles load unchanged and the desktop version is `1.2.4`.

### New in this release

**An AI assistant, ported from Aether Mobile 1.2.9.** A ✨ button next to every setting explains what that setting does on this machine, with a *Did not understand? Ask the assistant* footer that carries the question — and the explanation you just read — into the chat. Alongside it: a connection advisor that reads a redacted log excerpt, and a settings advisor whose output passes a hard allowlist before anything is written.

**Chat is its own tab, with everything the mobile edition has.** Four ready-made questions on the empty page, your message on screen the moment you press send, copy on any answer, editing a message you already sent, deleting one or several, *Try again* on a message that never went out, and *Stop* for an answer in flight. A failed request becomes a retryable bubble with a translated sentence and Google's raw text kept as detail, not a line in an error bar. Nothing is ever deleted without a confirmation that names the count.

**The assistant can change settings for you.** Ask for a change and the answer arrives with a proposal card: every entry as `setting: old → new` with the model's own one-line reason, an **Apply** button, and — once applied — a note that tunnel settings reach the engine at start-up, so a reconnect is needed. Two rules make this safe: the proposal is validated against the allowlist *before* it is drawn, so a change the app would refuse never appears as a button; and the chat's allowlist is deliberately **narrower** than the advisor's. The network backend, the upstream proxy, routing rules, split tunnelling, a manual endpoint, LAN sharing, the kill switch and every credential are not writable from a conversation, because those decide which traffic is protected and where it goes. Tuning — protocol, obfuscation strength, MTU, fragmentation, keepalive, DNS, IP version, reconnect behaviour — is.

**The settings area is now the mobile menu.** A hub of grouped rows — icon, title, subtitle, current value, chevron, and a per-row ✨ — opening subpages, with the old flat *Advanced* page gone and the reset row as a separate confirmed action. Hub and subpages are rendered from one section definition, so no control exists twice and storage behaviour cannot drift between the two.

**Aether Core 1.8.0 → 1.9.0.** A real three-way merge — upstream 1.9.0, this repository's patched 1.8.0, and the recorded 1.8.0 baseline — with 20 conflicts resolved and none left; the merged core passes its own suite at 265 tests. Upstream absorbed the 1.2.3 throughput work, so the `masque_h2.rs` patch was **dropped** instead of carried; keeping it would have meant maintaining a fork of code upstream now owns. What upstream did not absorb still ships as a patch and now carries `AETHER-APP-PATCH` markers in the source: CUBIC selection in smoltcp (pinned by the `socket-tcp-cubic` feature), the packet-queue depth cap, and the split `SO_RCVBUF`/`SO_SNDBUF` budgets.

**Fixed: endpoint mode *Manual range* had no effect.** The app sent `AETHER_SCAN_CIDRS`, `AETHER_MASQUE_CIDRS` and `AETHER_WG_CIDRS`, and no core version has ever read them, so the scan swept its own built-in ranges instead. Both scanners now honour a pinned range, invalid entries are dropped rather than silently collapsed to a single address, and built-in seed addresses outside the range are no longer probed.

**Upgrade note:** saved profiles load untouched. The AI layer is inert until you enter a key, and every AI control is additive — no existing setting changed its default, its name, or its meaning.

### How the AI layer is contained

Six boundaries, because this is a censorship-circumvention tool and an AI feature that leaks is worse than no AI feature:

* the Gemini key is sealed with **DPAPI** in `secrets.bin`, separate from `profile.json`, so *Reset all settings* does not take it and a copied file is worthless elsewhere;
* every request is dialled **through the tunnel's own local SOCKS5 proxy** with the host name sent as `ATYP=DOMAIN`, so the exit resolves the API host instead of the operator's resolver, and a gate refuses to send anything while the tunnel is down;
* the log excerpt is **redacted, not just truncated**: secret-shaped strings replaced (the user's own key included), public IPv4 masked to /16 and IPv6 to /32, WARP enrolment ids and bare UUIDs dropped, loopback and private ranges kept, and a hard size cap;
* one **model allowlist** is applied at all four points where a model id can reach a request — fresh list, cached list, default pick, and the id sent to `generateContent`;
* the model **cannot write security settings**: only allowlisted keys are applied, an unknown key is rejected and logged, `accessSecret` and `accessToken` are deliberately absent, and every value is type- and range-checked before the profile's own `normalize`;
* **the chat's allowlist is narrower still**, and a proposal is nothing until you press Apply — the model proposes, you decide, and the app writes through the single gate that also persists and revises the profile.

<div dir="rtl" align="right" markdown="1">

## تازه‌های نسخهٔ ۱.۲.۴

### امکانات جدید

- **دستیار هوش مصنوعی (Gemini)، پورت‌شده از AetherMobile 1.2.9**: ✨ کنار هر تنظیم، و پایکِ «متوجه نشدید؟ از دستیار بپرسید» که پرسش را همراهِ همان توضیحی که خوانده‌اید به گفت‌وگو می‌برد؛ به‌همراه مشاور اتصال روی برشِ پاک‌سازی‌شدهٔ لاگ و مشاور تنظیمات که پیشنهادش پیش از نوشتن از فهرست مجاز می‌گذرد.
- **چت یک تبِ مستقل است، با تمامِ امکاناتِ موبایل**: چهار پرسشِ آماده روی صفحهٔ خالی، دیدنِ پیام همان لحظهٔ زدنِ «بفرست»، کپیِ پاسخ، ویرایشِ پیامِ فرستاده‌شده، حذفِ یکی یا چند پیام، «تلاش مجدد» روی پیامی که نرفته، و «توقف» برای پاسخی که در راه است. درخواستِ شکست‌خورده یک حبابِ قابلِ تلاش مجدد می‌شود: جمله‌اش ترجمه‌شده و متنِ خامِ گوگل به‌عنوان جزئیات می‌ماند.
- **هیچ پیامی بی‌تأیید حذف نمی‌شود**: سطلِ روی هر حباب، حذفِ گروهی و پاک‌کردنِ کلِ گفت‌وگو، هر سه از یک دیالوگ رد می‌شوند که تعداد را در خودِ پرسش می‌گوید.
- **دستیار می‌تواند تنظیمات را برایتان اعمال کند**: پاسخ با کارتِ پیشنهاد می‌آید — هر تغییر به شکلِ `تنظیم: قدیم → جدید` با دلیلِ خودِ مدل — و دکمهٔ «اعمال». پس از اعمال، پنجره‌ای می‌گوید تنظیمات تونل هنگام راه‌اندازی به موتور داده می‌شود، پس باید یک‌بار قطع و وصل کنید.
- **مرزِ چت باریک‌تر از مرزِ مشاور است، عمداً**: بک‌اند شبکه، پروکسی بالادست، قواعد مسیریابی، تونلِ تفکیکی، اندپوینتِ دستی، اشتراک در شبکهٔ محلی، سوییچ قطع و هر اعتبارنامه‌ای از دلِ گفت‌وگو **نوشتنی نیستند**، چون تصمیم می‌گیرند کدام ترافیک محافظت شود و کجا برود. تنظیمِ ترابرد — پروتکل، شدت مبهم‌سازی، MTU، تکه‌تکه‌کردن، keepalive، DNS، نسخهٔ IP و رفتار اتصال مجدد — نوشتنی است.
- **منوی تنظیمات مثل موبایل**: هاب با ردیف‌های گروه‌بندی‌شده (آیکن، عنوان، زیرعنوان، مقدار فعلی، ✨) و زیرصفحه‌ها؛ صفحهٔ مسطح «پیشرفته» حذف شد و بازنشانی یک کنش جدا با تأیید است.
- **هستهٔ Aether Core 1.9.0** با merge سه‌طرفهٔ واقعی روی پچ‌های دسکتاپ، نه کپیِ ساده.
- **کلید API با DPAPI مهر می‌شود** و جدا از پروفایل ذخیره می‌شود؛ ترافیک هوش مصنوعی فقط از داخل تونل می‌رود و نام میزبان را نقطهٔ خروج حل می‌کند.
- **اصلاح: بازهٔ آدرسِ پین‌شده بی‌اثر بود.** هر دو اسکنر حالا آن را می‌خوانند، ورودی نامعتبر رد می‌شود و دانه‌های بیرون از بازه پروب نمی‌شوند.

### مرزهای لایهٔ هوش مصنوعی

- کلید در `secrets.bin` با DPAPI مهر می‌شود و «بازنشانی همهٔ تنظیمات» آن را نمی‌برد.
- هر درخواست از پروکسی SOCKS5 محلیِ خودِ تونل می‌رود و نام میزبان به‌صورت `ATYP=DOMAIN` فرستاده می‌شود؛ تا تونل بالا نباشد، دروازه چیزی نمی‌فرستد.
- برش لاگ پاک‌سازی می‌شود: رشته‌های شبیه راز جایگزین، IPv4 به /16 و IPv6 به /32 ماسک، شناسه‌های نصب حذف، و سقف حجم.
- فهرست مجاز مدل در چهار نقطه اعمال می‌شود، نه فقط روی پاسخ تازه.
- مدل فقط کلیدهای مجاز را می‌نویسد؛ `accessSecret` و `accessToken` در فهرست نیستند و نوع و بازهٔ هر مقدار پیش از `normalize` بررسی می‌شود.
- پیشنهادِ چت پیش از **رسم شدن** اعتبارسنجی می‌شود، پس تغییری که برنامه رد می‌کند هرگز به شکل یک دکمه دیده نمی‌شود؛ و تا «اعمال» را نزنید، هیچ چیزی نوشته نمی‌شود.

</div>
