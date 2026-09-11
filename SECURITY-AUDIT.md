# Aether Desktop 1.2.4 Security Audit

**Scope:** Windows desktop shell, Rust control plane, bundled tunnel engine (Aether Core 1.9.0) and its app patches, the chained Psiphon stage, the local proxy bridge, firewall policy, the new AI layer (key storage, network path, redaction, model-output boundary), local storage, logs, build pipeline, and bundled runtime. **Date:** 2026-09-10 (1.2.4).

**Method and its limits — read this before quoting the score.** This is a source-level review of this repository plus the tests that were actually executed here: the merged core's own suite (265 tests, 0 failed), the frontend jsdom suites, and `cargo check` on the core. It is **not** a penetration test, there was no run on real Windows hardware in this pass, no fuzzing, and no third-party review. Anything below that describes runtime behaviour is derived from code and from those tests, not from a live measurement.

## Executive result

**Score: 90/100** (1.2.2: 88). No hardcoded credentials. The connection path keeps TLS/SPKI pin verification, mandatory WebRTC protection, fail-closed IPv6 handling and the kill-switch. The two points gained over 1.2.2 are the DPAPI-sealed secret store and the validated user input on the engine's scan path; the deductions are unchanged Windows design constraints plus release-engineering duties.

| Area | Result | Evidence / remaining risk |
|---|---|---|
| **Secrets and keys** | **Pass** | No API keys, passwords, private keys or tokens are hardcoded. The Gemini key is sealed with DPAPI (`CryptProtectData`, `CRYPTPROTECT_UI_FORBIDDEN`) in `secrets.bin`, written atomically, and kept out of `profile.json`. Zero Trust secrets are write-only and excluded from serialization. |
| **AI network path** | **Pass** | Requests are dialled through the app's own SOCKS5 proxy with `ATYP=DOMAIN`, so the API host is resolved at the exit, not on the operator's resolver. A gate blocks every AI call while the tunnel is down. No new HTTP dependency; TLS is SChannel via `native-tls`. |
| **AI data minimisation** | **Pass with residual, accepted** | Log excerpts are redacted before leaving: secret-shaped strings replaced (including the user's own key), public IPv4 → /16 and IPv6 → /32, WARP `device=` ids and bare UUIDs dropped, loopback/private kept, hard size cap. **Residual:** the feature still sends a redacted excerpt to a third-party API on explicit user action. That is inherent to the feature and is disclosed in the README; it is inert without a key. |
| **AI output as configuration** | **Pass** | Only `WRITABLE` keys are applied; unknown keys are rejected **and logged**; `accessSecret`/`accessToken` are absent from the list by design; values are type- and range-checked before the profile's own `normalize`. The allowlist is derived from `ConnectionProfile`, so it cannot silently drift from the real field names. |
| **Model selection** | **Pass** | One allowlist enforced at four points (fresh list, cached list, default pick, outgoing id), so a stale cache cannot reintroduce a filtered model. |
| **Cryptography and protocols** | **Pass with pin rotation duty** | Core 1.9.0 validates the data plane before exposing the proxy and recovers MASQUE/WireGuard tunnels. TLS validates the platform chain and SPKI pins; an unpinned key is rejected. Pin rotation must ship before certificate/key rollover. |
| **DNS, IPv4, IPv6 and WebRTC leaks** | **Pass** | DNS/HTTP verification runs through SOCKS5. WebRTC direct UDP is blocked by browser policy and firewall rules. Global IPv6 is protected or blocked fail-closed. |
| **Traffic bypass** | **Pass for the supported desktop path** | System HTTP/HTTPS and SOCKS-aware applications use the local bridge; the kill-switch covers installed browsers and IPv6 fallback. Arbitrary third-party UDP applications are not transformed into TCP and stay outside the proxy model. |
| **User input reaching the engine** | **Pass (new in 1.2.4)** | A pinned address range is validated before it reaches the scanner: family and prefix length are checked, an out-of-range prefix such as `10.0.0.0/64` is rejected instead of collapsing to a single host, a bare address becomes a one-host range, and an all-invalid list falls back to built-in behaviour rather than producing an empty scan. |
| **Local storage** | **Partial** | Profile configuration and rotating diagnostics logs are plaintext by design. The AI key is now DPAPI-sealed. WARP identity files still need DPAPI/ACL hardening. |
| **OS privileges and UAC** | **Pass** | `requireAdministrator` is embedded and verified; the installer is admin-only; the manifest version tracks the release (`1.2.4.0`). |
| **Logging and errors** | **Pass** | Persistent logs mask public IPs and record no secrets; the upstream proxy value and Zero Trust secrets are masked; engine runtime logging stays at `info`. AI prompts and answers are not written to the rotating log. |
| **Dependencies and supply chain** | **Pass with release controls** | No new runtime dependency was added for the AI layer. Cargo dependencies are versioned and the lockfile moved with the core upgrade (`smoltcp 0.14.0`). Production releases should still add reproducible lockfile verification and Authenticode signatures. |
| **Engine upgrade integrity** | **Pass (new in 1.2.4)** | Core 1.9.0 was merged three-way against a recorded pristine baseline rather than copied over the app patches; 20 conflicts resolved, none left; surviving patches carry `AETHER-APP-PATCH` markers and are registered in `sync-core.sh`, so the next upgrade rebases them instead of dropping them. Verified by `cargo check` and 265 passing core tests. |

## Findings and controls

**A01–A05 (1.2.0–1.2.1), fixed and still enforced.** WebRTC direct UDP exposure, IPv6 fallback exposure, proxy restoration after disconnect, periodic upstream stalls, and UI blank/freeze. No regression was introduced by the settings-menu rebuild: the shell still paints before IPC and network work stays off the UI thread.

**A06: upstream proxy credentials, controlled (1.2.2).** `--upstream` may embed `user:pass`, so it is masked in the rotating log next to the Zero Trust secrets and validated on both sides of the IPC boundary.

**A07: identity refused by Cloudflare, handled (1.2.2).** A saved WARP identity the account API no longer accepts is replaced; offline or rate-limited answers never discard an identity.

**A08: host-name routing, scoped (1.2.2).** Domain rules match TLS SNI or HTTP `Host` only when domain rules exist and the proxy was handed a bare address; the connection still goes to the address the client asked for, bounded by `AETHER_ROUTE_SNIFF_MS`.

**A09: chained backend, contained (1.2.3).** Stage 2 listens on loopback only, `UpstreamProxyUrl` forces every Psiphon connection — server-list fetches included — through stage 1, and the exit-region value is validated in Rust before it reaches the config.

**A10: AI key at rest, fixed (1.2.4).** The key is not stored beside the profile in cleartext. DPAPI ties the ciphertext to the Windows login session, so `secrets.bin` copied to another account or machine does not open. A failed decrypt is treated as "no key" and the UI shows the same path a fresh install sees, which avoids an error state that would tempt a user to paste the key somewhere else.

**A11: prompt injection into configuration, contained (1.2.4).** The threat is not hypothetical: the settings advisor asks a third-party model for a JSON patch, and the app it patches controls the machine's network path. Defence is structural rather than textual — an allowlist that omits every credential field, rejection with a log entry for anything unknown, and type/range validation before `normalize`. A prompt is a request, not a guarantee, and is therefore not counted as a control.

**A12: log exfiltration through the advisor, contained (1.2.4).** A connection log written for a monitored user is exactly the wrong thing to forward verbatim. The excerpt is filtered (not merely truncated) and capped so a verbose session cannot quietly ship a megabyte of history off the device.

**A13: a setting that silently did nothing, fixed (1.2.4).** Endpoint mode *Manual range* sent three environment variables no core version read, so a user who pinned a range to avoid a blocked or monitored prefix kept scanning the built-in ranges — a trust failure, not a leak, and the more dangerous kind because the UI claimed otherwise. Both scanners now honour the range, and built-in seeds outside it are no longer probed, since seeds are probed first and would otherwise have landed the tunnel on an address the user deliberately excluded.

## Deductions (why 90 and not 100)

* **−4 — the system proxy is not a universal VPN route.** HTTP/HTTPS and SOCKS-aware applications are covered; an arbitrary UDP application is not. This is a deliberate Windows design decision, documented in the README, not an oversight.
* **−3 — plaintext at rest for non-secret state.** `profile.json`, the rotating log and the WARP identity files are unencrypted. DPAPI/ACL hardening of the identity files is the next step; the AI key already has it.
* **−2 — release artifacts are not Authenticode-signed** unless a signing certificate is configured in CI.
* **−1 — SPKI pin rotation is a manual duty** that must precede any certificate or key rollover.

## Release gate

Do not publish unless all are green: `cargo fmt --check`, x64 and x86 `cargo test`, the merged core's own suite, the frontend jsdom suites, the frontend build, the Tauri release build, embedded manifest verification, installer silent install/uninstall, kill-switch cleanup, and a manual WebRTC/DNS/IPv6 leak test on real hardware.

<div dir="rtl" align="right" markdown="1">

# ممیزی امنیتی نسخهٔ ۱.۲.۴

**دامنه:** پوستهٔ ویندوز، کنترل‌پلین Rust، هستهٔ تونل ۱.۹.۰ و پچ‌هایش، استیج
زنجیره‌ای سایفون، پل پروکسی، فایروال، **لایهٔ هوش مصنوعی**، ذخیره‌سازی محلی،
لاگ‌ها و زنجیرهٔ بیلد. **امتیاز: ۹۰ از ۱۰۰** (۱.۲.۲: ۸۸).

**روش و حدِ آن:** این یک بازبینی در سطح سورس است به‌همراه آزمون‌هایی که واقعاً
اجرا شدند (۲۶۵ آزمون هستهٔ merge‌شده، آزمون‌های jsdom رابط، و `cargo check`).
آزمون نفوذ نیست، روی سخت‌افزار واقعی ویندوز اجرا نشده، فازینگ نشده و بازبینی
شخص ثالث ندارد.

| بخش | نتیجه |
|---|---|
| **کلیدها و اسرار** | چیزی هاردکد نشده؛ کلید هوش مصنوعی با DPAPI در `secrets.bin` مهر می‌شود و جدا از پروفایل است؛ اسرار Zero Trust ذخیره نمی‌شوند. |
| **مسیر شبکهٔ هوش مصنوعی** | فقط از پروکسی SOCKS5 خودِ تونل، با `ATYP=DOMAIN` تا نام میزبان را نقطهٔ خروج حل کند؛ با تونلِ خاموش، دروازه هر تماس را رد می‌کند. |
| **کمینه‌سازی دادهٔ ارسالی** | برش لاگ پاک‌سازی و سقف‌دار می‌شود: رشته‌های شبیه راز جایگزین، IPv4 به /16 و IPv6 به /32، شناسه‌های نصب حذف، لوپ‌بک دست‌نخورده. **باقیمانده و پذیرفته‌شده:** ارسال برشِ پاک‌سازی‌شده به API شخص ثالث، با کنش صریح کاربر. |
| **خروجی مدل به‌عنوان پیکربندی** | فقط کلیدهای `WRITABLE`؛ کلید ناشناخته رد و لاگ می‌شود؛ `accessSecret`/`accessToken` در فهرست نیستند؛ نوع و بازه پیش از `normalize` بررسی می‌شود. |
| **رمزنگاری و پروتکل** | اعتبارسنجی زنجیرهٔ TLS و پین SPKI؛ چرخش پین وظیفهٔ دستی است. |
| **نشت DNS، IPv6 و WebRTC** | مسیر حفاظت‌شده تأیید می‌شود؛ UDP مستقیم و IPv6 ناامن fail-closed هستند. |
| **عبور خارج از تونل** | HTTP/HTTPS و برنامه‌های SOCKS-aware حفاظت می‌شوند؛ UDP دلخواه بیرون از مدل پروکسی است. |
| **ورودی کاربر به موتور** | بازهٔ پین‌شده پیش از رسیدن به اسکنر اعتبارسنجی می‌شود؛ `10.0.0.0/64` رد می‌شود نه اینکه بی‌صدا به یک میزبان تبدیل شود. |
| **ذخیره‌سازی محلی** | پروفایل و لاگ متن ساده‌اند؛ کلید هوش مصنوعی مهرشده است؛ سخت‌سازی فایل هویت WARP باقی است. |
| **مجوز و سیستم‌عامل** | `requireAdministrator` اجباری و verify می‌شود؛ نسخهٔ مانیفست `1.2.4.0`. |
| **لاگ** | آی‌پی ماسک، اسرار حذف، سطح موتور `info`؛ پرسش و پاسخ هوش مصنوعی در لاگ نوشته نمی‌شود. |
| **یکپارچگی ارتقای هسته** | merge سه‌طرفه با baseline بکر، ۲۰ تعارض حل و صفر باقی‌مانده، پچ‌های بازمانده با نشانهٔ `AETHER-APP-PATCH` ثبت‌شده در `sync-core.sh`. |

**کسری‌ها:** ۴− پروکسی سیستمی مسیر جهانی VPN نیست (UDP برنامه‌های ثالث)؛ ۳−
پروفایل، لاگ و فایل هویت متن ساده‌اند؛ ۲− امضای Authenticode در انتشار؛ ۱−
چرخش دستی پین SPKI.

</div>
