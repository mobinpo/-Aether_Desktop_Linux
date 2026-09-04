# Aether Desktop 1.2.3

## What's new

**Upgrade notice:** 1.2.3 adds the **Aether → Psiphon** chained transport backend — the capability of Aether Mobile 1.2.8, brought to Windows with the same method — bundles **Aether Core 1.8.0**, and gives the exit-country picker a flag on every row. Saved profiles load unchanged and the desktop version stays `1.2.3`.

### New in this release

**Added:** the **Aether → Psiphon** transport backend (Advanced → *Backend*), which keeps Aether's obfuscated transport as the first hop and takes its exit from an ordinary hosting IP; an **Exit country** picker covering all 56 Psiphon egress regions, each row carrying its country flag, with complete flag artwork across the region list; the filtering-server watchdog and exit-region steering from the mobile edition; the Psiphon stage (`psiphon-tunnel-core`) built from a pinned upstream tag in CI and shipped inside both installers; the complete Aether Core 1.8.0 source and build baseline; and a **desktop throughput profile** — per-flow receive windows sized to a desktop bandwidth-delay product, CUBIC congestion control on every tunnelled flow, batched WireGuard encapsulation, data-plane socket buffers applied on Windows, an RTT budget for endpoint selection, and a new `[uplink]` telemetry line.

### The chained backend

```text
stage 1   Aether engine  → SOCKS5 127.0.0.1:1819     (no data path yet)
stage 2   Psiphon        → SOCKS5 127.0.0.1:1825     dials out through 1819
then      bridge + system proxy → 127.0.0.1:1825     exit = Psiphon
```

Aether's exits are Cloudflare WARP anycast addresses, and a large set of destinations serves a different view of the internet from them. The chain takes the **exit** from an ordinary hosting address while keeping Aether's obfuscated transport on the **first hop**, which is the hop that has to survive the local network. Single-hop Psiphon is deliberately absent: the chain exists precisely because the first hop is the one that needs Aether.

Brought over from mobile: the Psiphon config keys including `UpstreamProxyUrl`, so every connection Psiphon makes leaves through stage 1; two-pass establishment (your country first, then no region filter with a fresh datastore), because `EgressRegion` is a hard filter; following the port Psiphon actually bound (`ListeningSocksProxyPort`); the stage-1 gate that proves the engine is a working SOCKS5 proxy before stage 2 starts; a 150-second verification window sized for two hops warming up; and the filtering-server watchdog with the same thresholds, blacklist, region steering and rotation grace window.

### The desktop throughput profile

The engine sizes the data plane for a PC on a fat line: the per-flow smoltcp **receive window** is sized separately from the send buffer and set to a desktop bandwidth-delay product, because a flow can never download faster than window / RTT; every tunnelled flow runs **CUBIC**, pinned by the `socket-tcp-cubic` feature; outbound packets are **encapsulated in bursts** under a single boringtun session acquisition; `SO_RCVBUF` and `SO_SNDBUF` are **applied on Windows** to every data-plane datagram socket as independent budgets; the netstack backlog is ordered **per flow** with its own per-pass budget for control messages; the device transmit ring holds a burst across a retry; endpoint selection carries an **RTT budget** with a floor, so a rescan can only ever return an endpoint at least as fast as the cached one it replaced; and the diagnostics log gains an `[uplink <peer>]` line every 15 seconds.

<div dir="rtl" align="right" markdown="1">

## تازه‌های نسخهٔ ۱.۲.۳

### امکانات جدید

- **بک‌اند ترابرد «اِتِر → سایفون»** در «پیشرفته ← بک‌اند»: هاپ اول روی ترابرد مبهم‌سازی‌شدهٔ اِتِر می‌ماند و خروجی یک IP هاستینگ عادی می‌شود، برای سایت‌هایی که رنج‌های WARP را نمی‌پذیرند.
- **انتخابگر کشور خروج** با پوشش هر ۵۶ منطقهٔ خروج سایفون و پرچم کنار هر ردیف.
- **هستهٔ Aether Core 1.8.0** با سورس کامل و بیلدِ قابل بازتولید داخل هر دو نصاب.
- **حامل HTTP/3 (QUIC) به‌عنوان مسیر پیش‌فرض MASQUE**، همراه با حاملِ HTTP/2 (TCP) برای شبکه‌هایی که UDP را می‌بندند.
- **اثرانگشت شبکه با آزمون واقعی UDP** پیش از اتصال: نردبان Smart Auto حامل مناسب را بر اساس نتیجهٔ همین آزمون انتخاب می‌کند.
- **پروفایل توان‌عبوری دسکتاپ**: پنجره‌های جریان به اندازهٔ حاصل‌ضرب پهنای‌باند در تأخیر دسکتاپ، کنترل ازدحام CUBIC روی هر جریان تونل‌شده، بسته‌بندی دسته‌ای بسته‌ها روی هر دو حامل، و بافرهای سوکت مسیر داده روی ویندوز.
- **سطر تلمتری `[h2]`** در تشخیص‌ها: پنجره‌ها، نرخ دانلود، و تعداد بستهٔ درون هر فریم، هر ۱۵ ثانیه.
- **بودجهٔ RTT با کف تضمینی برای انتخاب اندپوینت**، تا اسکن مجدد فقط اندپوینتی را برگرداند که دست‌کم به‌اندازهٔ اندپوینت فعلی سریع است.
- **واچ‌داگ سرور فیلترشده، هدایت منطقهٔ خروج و چرخش سرور** برای استیج سایفون.
- **پیش‌فرض‌های تازهٔ پنل پیشرفته**: زبان English، بک‌اند Aether، کشور خروج Automatic، پروتکل Smart، حالت اسکن Turbo، نسخهٔ IP یعنی IPv4، نویز Off و اندپوینت Automatic.
- **اشتراک تونل روی شبکهٔ محلی**، گارد نشتی WebRTC/UDP، کلید قطع و حفاظت نشتی IPv6.

### خط لولهٔ حالت ترکیبی

```text
stage 1    موتور اِتِر      → SOCKS5 127.0.0.1:1819
stage 2    سایفون          → SOCKS5 127.0.0.1:1825
سپس        پل + پروکسی سیستمی → 127.0.0.1:1825      خروجی = سایفون
```

</div>

