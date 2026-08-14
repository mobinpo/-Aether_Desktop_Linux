# Aether Desktop 1.2.1

## What's new

### Short comparison with 1.2.0

**Upgrade notice:** Upgrade to 1.2.1 for the bundled Aether Core 1.6.0 and its validated, self-recovering tunnel path. The 1.2.0 leak protections remain mandatory.

**Added:** vendored Aether Core 1.6.0, end-to-end data-plane validation, automatic MASQUE/WireGuard recovery, last-good-gateway reuse, HTTP/2 ClientHello fragmentation, and aligned sync/rollback/build metadata.

**Preserved:** mandatory WebRTC and IPv6 fail-closed protection, browser/network kill-switch, three-target desktop watchdog, exact proxy/PAC restoration, bounded shutdown, and bilingual security controls from 1.2.0.

### Detailed changes

**Core 1.6.0:** The supplied engine source is bundled under `native/aether`; CI, rollback, portable packaging, and runtime version reporting now share the 1.6.0 baseline. The engine validates real tunnel data, reconnects automatically, retries the last good gateway, and supports MASQUE HTTP/2 ClientHello fragmentation.

**Mandatory leak protection:** WebRTC protection cannot be disabled from the UI. Browser policy and elevated firewall enforcement block direct STUN/TURN UDP before the connection is considered safe.

**IPv6 protection:** Global IPv6 is protected by the tunnel when a valid route exists and blocked fail-closed otherwise.

**Kill-switch:** Browser fallback traffic is blocked while the protected session is unavailable. Disconnect removes only Aether rules and restores the exact pre-session proxy and PAC values.

**Connection watchdog:** Three independent SOCKS5 targets are probed every 30 seconds in the background. Restart occurs only after three consecutive failed rounds.

**Startup and shutdown:** The shell renders before IPC, initial state calls run in parallel, engine logging uses `info`, and native cleanup is ordered and bounded.

### Security audit summary

| Area | Result |
|---|---|
| Secrets and keys | No hardcoded credentials; sensitive access values are not persisted |
| TLS and certificates | Platform validation plus SPKI pin verification |
| DNS, IPv6 and WebRTC | Protected path verified; direct UDP and unsafe IPv6 fallback blocked |
| Local storage and logs | IPs masked; secrets excluded; identity-file encryption remains a hardening item |
| Permissions and build | Mandatory UAC; CI checks source, tests, manifest, installer, and cleanup |

Full report: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

<div dir="rtl">

## تازه‌های نسخهٔ ۱.۲.۱

### مقایسهٔ خلاصه با نسخهٔ ۱.۲.۰

**یادآوری ارتقا:** برای هستهٔ همراه Aether Core 1.6.0 و مسیر تونل اعتبارسنجی‌شده و خودترمیم به نسخهٔ ۱.۲.۱ بروزرسانی کنید. محافظت‌های اجباری نسخهٔ ۱.۲.۰ حفظ شده‌اند.

**افزوده شد:** هستهٔ vendorشدهٔ ۱.۶.۰، بررسی واقعی data-plane، بازیابی خودکار MASQUE/WireGuard، استفاده از آخرین gateway سالم، fragmentation روی HTTP/2 و هم‌ترازی کامل sync، rollback و build.

**حفظ شد:** محافظت اجباری WebRTC و IPv6، کیل‌سوییچ، واچداگ سه‌هدفهٔ دسکتاپ، بازگردانی دقیق proxy/PAC، خروج محدود به زمان و کنترل‌های امنیتی دوزبانهٔ نسخهٔ ۱.۲.۰.

### جزئیات تغییرها

**هستهٔ ۱.۶.۰:** سورس ارسالی موتور در `native/aether` قرار گرفت و CI، rollback، بسته‌بندی پرتابل و نمایش نسخهٔ runtime همگی از baseline یکسان ۱.۶.۰ استفاده می‌کنند. موتور عبور واقعی داده را تأیید می‌کند، خودکار reconnect می‌شود، آخرین gateway سالم را دوباره امتحان می‌کند و fragmentation پیام ClientHello روی HTTP/2 را دارد.

**محافظت اجباری در برابر نشت:** محافظت WebRTC از رابط کاربری خاموش‌شدنی نیست. سیاست مرورگر و فایروال با دسترسی مدیر، UDP مستقیم STUN/TURN را پیش از امن اعلام‌شدن اتصال مسدود می‌کنند.

**محافظت IPv6:** IPv6 عمومی در صورت وجود مسیر معتبر از تونل حفاظت‌شده عبور می‌کند و در غیر این صورت fail-closed مسدود می‌شود.

**کیل‌سوییچ:** ترافیک بازگشتی مرورگرها هنگام نبود مسیر امن مسدود است. قطع اتصال فقط قواعد Aether را پاک می‌کند و مقادیر دقیق پروکسی و PAC قبل از اتصال را برمی‌گرداند.

**واچداگ اتصال:** سه مقصد مستقل SOCKS5 هر ۳۰ ثانیه در پس‌زمینه بررسی می‌شوند و راه‌اندازی مجدد فقط پس از سه دور شکست متوالی انجام می‌شود.

**شروع و خروج:** پوسته قبل از IPC نمایش داده می‌شود، درخواست‌های اولیه موازی‌اند، لاگ موتور روی `info` است و پاک‌سازی native مرتب و محدود به زمان انجام می‌شود.

### خلاصهٔ ممیزی امنیتی

| بخش | نتیجه |
|---|---|
| کلیدها و اسرار | اعتبارنامهٔ هاردکدشده وجود ندارد؛ مقادیر حساس ذخیره نمی‌شوند |
| TLS و گواهی‌ها | اعتبارسنجی سیستم‌عامل به‌همراه پین SPKI |
| DNS، IPv6 و WebRTC | مسیر حفاظت‌شده بررسی می‌شود؛ UDP مستقیم و IPv6 ناامن مسدود است |
| ذخیره‌سازی و لاگ | آی‌پی‌ها ماسک و اسرار حذف می‌شوند؛ رمزگذاری فایل هویت هنوز مورد سخت‌سازی است |
| مجوزها و بیلد | UAC اجباری است؛ CI سورس، تست، مانیفست، نصاب و پاک‌سازی را بررسی می‌کند |

گزارش کامل: [SECURITY-AUDIT.md](SECURITY-AUDIT.md).

</div>

---

<details>
<summary>Previous release: 1.1.0</summary>

The previous release introduced the first complete desktop UI, protocol selection, live connection diagnostics, network sharing, bilingual support, and the initial security controls. Its full historical notes remain in the repository history.

</details>


## Important reminder

To get the best result on Android or Windows:

- Wait 1 to 3 minutes on each protocol. Connection time depends on the operator and region.
- Test different protocols and settings because DPI behavior varies by SIM, region, city, and network.
- On mobile data, toggle Airplane mode several times to obtain a different IP range, then retry.
- On Wi-Fi, turn the modem off for 1 to 2 minutes to obtain a different IP range, then retry.
- If it still does not connect, this VPN may not be compatible with that network.
- Different results across users are expected because operator DPI policies differ.

<div dir="rtl">

## یادآوری مهم

یه یادآوری مهم که حتماً بخونیدش 👇
برای اینکه اپ (چه نسخه اندروید چه ویندوز) براتون وصل شه، این چند تا نکته رو رعایت کنید تا بهترین نتیجه رو بگیرید:
⏳ رو هر پروتکل ۱ تا ۳ دقیقه صبر کنید تا وصل شه. بسته به اپراتور و منطقه‌تون این زمان فرق داره، عجله نکنید.
🔄 پروتکل‌ها و تنظیمات مختلف رو تست کنید. چرا؟ چون DPI هر سیم‌کارت با سیم‌کارت دیگه، هر منطقه با منطقه دیگه و هر شهر با شهر دیگه فرق داره.
📱 اگه با موبایل وصل نشدید: چند بار گوشی رو ببرید رو حالت هواپیما و برگردونید تا رنج آی‌پی‌تون عوض شه، بعد دوباره پروتکل‌های مختلف رو تست کنید. خلاصه باید قلق DPI اپراتور و منطقه خودتون دستتون بیاد 😉
📶 اگه با وای‌فای هستید: مودم رو ۱ تا ۲ دقیقه خاموش کنید تا رنج آی‌پی عوض شه، بعد دوباره با پروتکل‌ها و تنظیمات مختلف امتحان کنید.
❌ اگه بازم وصل نشد، یعنی این وی‌پی‌ان با نت شما جواب نمی‌ده و باید برید سراغ وی‌پی‌انی که با نت شما سازگاره.
⚠️ و نکته آخر: بعضی از کاربرا میگن این اپ مشکل داره و واسشون کار نمیکنه. اگه مشکل از خود اپ بود، نباید برای هیچ‌کس کار می‌کرد! برای خیلی‌ها داره کار می‌کنه و هر کسی تجربه متفاوتی داره. پس اگه برای شما وصل نمی‌شه، مشکل از Aether نیست؛ مشکل از DPI ایه که رو اپراتور شماست و جلوی کار کردن اپ رو می‌گیره.
</div>
