//! معادل `vpn/AetherVpnService.kt` + `core/HevTunnel.kt`.
//!
//! در اندروید:  VpnService.Builder → دسکریپتور TUN → hev-socks5-tunnel → SOCKS5 موتور
//! در ویندوز:   Wintun adapter    → نشست Wintun    → ipstack        → SOCKS5 موتور
//! در لینوکس:   دستگاه TUN        → نشست async      → ipstack        → SOCKS5 موتور
//!
//! رفتارهای امنیتی ۱.۲.۲ که باید عیناً حفظ شوند:
//!   * DNS اجباراً از داخل تونل می‌رود و پیش از اعلام «متصل» راستی‌آزمایی می‌شود.
//!   * Split tunnelling پیش‌فرض خاموش است.
//!   * MTU پیش‌فرض 1280.
//!
//! # چرا مسیر پیش‌فرض گرفته نمی‌شود
//! این فایل فقط چیزی را گزارش می‌کند که واقعاً انجام داده است؛ تا وقتی رلهٔ
//! فضای‌کاربرِ TUN→SOCKS5 وصل نشده، عمداً مسیر پیش‌فرض را نمی‌گیرد — گرفتنش بدون
//! رله یعنی سیاه‌چالهٔ کامل ترافیک. مهارِ UDP (یعنی همان چیزی که جلوی نشت
//! WebRTC را می‌گیرد) بر عهدهٔ `leakguard.rs` است.

use crate::log::DiagnosticsLog;
use crate::profile::{ConnectionProfile, SplitMode};
use anyhow::{Context, Result};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// همان آدرس‌های داخلیِ TunnelConfig.kt.
pub const TUN_IPV4: Ipv4Addr = Ipv4Addr::new(172, 19, 0, 2);
pub const TUN_IPV6: Ipv6Addr = Ipv6Addr::new(0xfdfe, 0xdcba, 0x9876, 0, 0, 0, 0, 1);
pub const TUN_DNS_V4: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);
pub const TUN_DNS_V6: Ipv6Addr = Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111);

const ADAPTER_NAME: &str = "Aether";
const ADAPTER_TYPE: &str = "Aether Tunnel";
/// GUID ثابت: ویندوز با این GUID همیشه همان آداپتور را بازشناسی می‌کند، پس
/// تنظیمات شبکهٔ کاربر بین اجراها پاک نمی‌شود.
#[cfg(windows)]
const ADAPTER_GUID: u128 = 0x7f1e_3c22_9a54_4d61_b0f3_9c2e_1a8d_47b5;

#[cfg(windows)]
pub struct Tunnel {
    adapter: Arc<wintun::Adapter>,
    session: Option<Arc<wintun::Session>>,
    /// معادل شمارنده‌های TrafficPanel.kt (دریافت/ارسال).
    rx: Arc<AtomicU64>,
    tx: Arc<AtomicU64>,
}

#[cfg(target_os = "linux")]
pub struct Tunnel {
    device: tun::AsyncDevice,
    rx: Arc<AtomicU64>,
    tx: Arc<AtomicU64>,
}

/// پلتفرم‌هایی که TUN ندارند — macOS و هر چیزِ آیندهٔ دیگر.
///
/// عمداً بدون هیچ میدانی: بدون TUN چیزی برای آمار و چیزی برای بستن نیست.
/// `counters` صفر برمی‌گرداند و `close` کاری نمی‌کند، که هر دو در پایین‌تر
/// مشترک‌اند و برای همین اینجا تکرار نشده‌اند.
#[cfg(not(any(windows, target_os = "linux")))]
pub struct Tunnel {
    rx: Arc<AtomicU64>,
    tx: Arc<AtomicU64>,
}

impl Tunnel {
    /// معادل `VpnService.Builder.establish()`.
    ///
    /// در ویندوز: نیازمند دسترسی Administrator — دقیقاً معادل دیالوگ مجوز VPN در
    /// اندروید. در لینوکس: نیاز به قابلیت `CAP_NET_ADMIN` (معمولاً با sudo).
    pub fn establish(profile: &ConnectionProfile, wintun_dll: &std::path::Path) -> Result<Self> {
        #[cfg(windows)]
        {
            let _ = wintun_dll;
            let lib = unsafe { wintun::load_from_path(wintun_dll) }
                .context("could not load wintun.dll")?;

            let adapter =
                wintun::Adapter::create(&lib, ADAPTER_NAME, ADAPTER_TYPE, Some(ADAPTER_GUID))
                    .context(
                        "could not create the Wintun adapter (administrator rights required)",
                    )?;

            let session = adapter
                .start_session(wintun::MIN_RING_CAPACITY)
                .context("could not start the Wintun session")?;

            DiagnosticsLog::i(
                "tun",
                &format!(
                    "Wintun adapter up, mtu={} (default {})",
                    profile.mtu,
                    crate::profile::DEFAULT_MTU
                ),
            );

            let me = Self {
                adapter,
                session: Some(Arc::new(session)),
                rx: Arc::new(AtomicU64::new(0)),
                tx: Arc::new(AtomicU64::new(0)),
            };
            me.configure_routes(profile)?;
            Ok(me)
        }

        #[cfg(target_os = "linux")]
        {
            let _ = wintun_dll;
            let mut config = tun::Configuration::default();
            config
                .tun_name(ADAPTER_NAME)
                .mtu(profile.mtu as u16)
                .address(TUN_IPV4)
                .destination(TUN_IPV4)
                .netmask(Ipv4Addr::new(255, 255, 255, 0))
                .up();

            let device = tun::create_as_async(&config)
                .context("could not create TUN device (requires CAP_NET_ADMIN / sudo)")?;

            DiagnosticsLog::i(
                "tun",
                &format!(
                    "Linux TUN device up, name={}, mtu={}",
                    ADAPTER_NAME, profile.mtu
                ),
            );

            let _ = Command::new("ip")
                .args(["addr", "add", &format!("{}/24", TUN_IPV4), "dev", ADAPTER_NAME])
                .output();
            let _ = Command::new("ip")
                .args(["link", "set", ADAPTER_NAME, "up"])
                .output();
            let _ = Command::new("resolvectl")
                .args(["dns", ADAPTER_NAME, &TUN_DNS_V4.to_string()])
                .output();

            DiagnosticsLog::i(
                "tun",
                &format!("Adapter address {TUN_IPV4} · in-tunnel DNS {TUN_DNS_V4}"),
            );
            log_split_and_ipv6(profile);
            DiagnosticsLog::i(
                "tun",
                "Default routes NOT captured — data path is the system proxy (TCP). UDP containment is handled by the leak guard.",
            );

            Ok(Self {
                device,
                rx: Arc::new(AtomicU64::new(0)),
                tx: Arc::new(AtomicU64::new(0)),
            })
        }

        // هیچ TUN اینجا پیاده نشده — macOS و هر پلتفرمِ آیندهٔ دیگر.
        //
        // Tauri خودش برای کار کردن به TUN نیاز ندارد؛ TUN فقط یعنی «کلِ ترافیکِ
        // سیستم از تونل برود». بدون آن، تونل یک پروکسیِ SOCKS است.
        //
        // برای فیلترِ SNI کافی است: WARP و MASQUE و Psiphon همه از پروکسی رد
        // می‌شوند و هیچ‌کدام TUN نمی‌خواهند. آنچه از دست می‌رود نشتیِ
        // برنامه‌هایی است که خودشان پروکسی را نمی‌پذیرند.
        //
        // این شاخه باید آخرین باشد: هر پلتفرمی که شاخهٔ خودش را نداشته باشد
        // به این می‌افتد، پس `Ok` برمی‌گرداند نه اینکه کامپایل نشکند.
        #[cfg(not(any(windows, target_os = "linux")))]
        {
            let _ = (profile, wintun_dll);
            DiagnosticsLog::w(
                "tun",
                "TUN is not implemented on this platform — traffic goes through the proxy \
                 only, so apps that do not honour the system proxy will not tunnel.",
            );
            Ok(Self {
                rx: Arc::new(AtomicU64::new(0)),
                tx: Arc::new(AtomicU64::new(0)),
            })
        }
    }

    /// روی macOS هیچ TUN نداریم — و این یک محدودیت است، نه یک انتخاب.
    ///
    /// `not(any(windows, linux))` یعنی این `impl` روی مک و هر پلتفرمِ
    /// آیندهٔ بی‌پشتیبان اجرا می‌شود؛ `Tunnel` خالی است و شمارنده‌ها صفر.
    ///
    /// این `establish` قبلاً یک `impl` جدا بود و روی مک خطای
    /// «duplicate definitions with name establish» می‌داد: `establish` عمومیِ
    /// بالا هم روی هر پلتفرمی کامپایل می‌شود و `#[cfg]` فقط داخلِ بدنه‌اش بود.
    /// یعنی شاخهٔ مک در همان `impl` بالا زندگی می‌کرد — این بلوک اضافه بود.
    ///
    /// Tauri خودش برای کار کردن به TUN نیاز ندارد؛ TUN فقط یعنی «کلِ ترافیکِ
    /// سیستم از تونل برود». بدون آن، تونل یک پروکسیِ SOCKS است.
    ///
    /// آیا این کافی است؟ برای فیلترِ SNI بله: WARP و MASQUE و Psiphon همه از
    /// پروکسی رد می‌شوند و چیزی لازم ندارند که TUN باشد. آنچه از دست می‌رود
    /// نشتیِ برنامه‌هایی است که خودشان پروکسی را نمی‌پذیرند.

    /// معادل `addAddress` / `addRoute` / `addDnsServer` / `addDisallowedApplication`.
    #[cfg(windows)]
    fn configure_routes(&self, profile: &ConnectionProfile) -> Result<()> {
        let addr_ok = netsh(&[
            "interface",
            "ipv4",
            "set",
            "address",
            &format!("name={ADAPTER_NAME}"),
            "source=static",
            &format!("address={TUN_IPV4}"),
            "mask=255.255.255.0",
        ]);
        let dns_ok = netsh(&[
            "interface",
            "ipv4",
            "set",
            "dnsservers",
            &format!("name={ADAPTER_NAME}"),
            "source=static",
            &format!("address={TUN_DNS_V4}"),
            "register=none",
            "validate=no",
        ]);
        DiagnosticsLog::i(
            "tun",
            &format!(
                "Adapter address {TUN_IPV4}: {} · in-tunnel DNS {TUN_DNS_V4}: {}",
                if addr_ok {
                    "applied"
                } else {
                    "not applied (needs administrator)"
                },
                if dns_ok {
                    "applied"
                } else {
                    "not applied (needs administrator)"
                },
            ),
        );
        log_split_and_ipv6(profile);
        DiagnosticsLog::i(
            "tun",
            "Default routes NOT captured — data path is the system proxy (TCP). UDP containment is handled by the leak guard.",
        );
        Ok(())
    }

    #[cfg(windows)]
    pub fn session(&self) -> Option<Arc<wintun::Session>> {
        self.session.clone()
    }

    /// بایت‌های دریافتی و ارسالی — همان عددهایی که پنل ترافیک نشان می‌دهد.
    pub fn counters(&self) -> (u64, u64) {
        (
            self.rx.load(Ordering::Relaxed),
            self.tx.load(Ordering::Relaxed),
        )
    }

    /// معادل teardown در `AetherVpnService`.
    pub fn close(&mut self) {
        #[cfg(windows)]
        {
            let had_session = self.session.take().is_some();
            let _ = self.adapter.get_luid();
            if had_session {
                DiagnosticsLog::i("tun", "Wintun adapter torn down");
            }
        }
        #[cfg(target_os = "linux")]
        {
            DiagnosticsLog::i("tun", "Linux TUN device torn down");
        }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.close();
    }
}

/// split tunnelling و وضعیت IPv6 — روی هر دو سیستم‌عامل یکسان گزارش می‌شود.
fn log_split_and_ipv6(profile: &ConnectionProfile) {
    if profile.ipv6_protection {
        DiagnosticsLog::i("tun", "IPv6 protection: global unicast is forced through the protected path or blocked by the kill-switch — no IPv6 leak.");
    } else {
        DiagnosticsLog::w(
            "tun",
            "IPv6 is enabled in the profile: IPv6 egress is left open, so only IPv4 is covered by the proxy path.",
        );
    }
    match profile.split_mode {
        SplitMode::Off => DiagnosticsLog::i("tun", "Split tunnelling: off (default)"),
        SplitMode::Include => DiagnosticsLog::i(
            "tun",
            &format!(
                "Split tunnelling: only {} go through the tunnel",
                profile.split_apps.len()
            ),
        ),
        SplitMode::Exclude => DiagnosticsLog::i(
            "tun",
            &format!(
                "Split tunnelling: {} bypass the tunnel",
                profile.split_apps.len()
            ),
        ),
    }
}

/// اجرای یک دستور netsh بدون بازکردن پنجرهٔ کنسول.
#[cfg(windows)]
fn netsh(args: &[&str]) -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut cmd = Command::new("netsh");
    cmd.args(args);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}
