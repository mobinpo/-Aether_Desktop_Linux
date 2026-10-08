//! پروکسی سیستمی macOS — معادل WinINET ویندوز و GSettings لینوکس.
//!
//! مسیر دادهٔ فعلی روی مک همین است: پل محلیِ `share.rs` روی 127.0.0.1 بالا
//! می‌آید و این ماژول تنظیمِ پروکسیِ سیستم را به آن می‌دواند تا مرورگرها از
//! تونل رد شوند.
//!
//! مکانیزم، `networksetup` است — تنها راهِ پشتیبانی‌شده و بدون نیاز به دسترسیِ
//! ریشه. سرویس‌های فعال از `scutil --nwi` خوانده می‌شوند، چون نامِ سرویس با
//! UUID تغییر می‌کند و «Wi‑Fi» فرض کردن روی مکی که با کابل وصل است، پروکسی را
//! روی دستگاهِ اشتباه می‌گذارد.
//!
//! # چرا `-webproxy` و نه `-socksfirewallproxy`
//!
//! هر دو پرچم وجود دارند، ولی `-socksfirewallproxy` پروکسیِ SOCKS را روی همهٔ
//! سرویس‌ها می‌گذارد و در نبودِ TUN — که روی مک نداریم — تنها راهِ عبورِ UDP
//! می‌شود، و در عین حال مرورگرها را از مسیرِ درست خارج می‌کند. `-webproxy` همان
//! چیزی است که ویندوز و لینوکس اینجا می‌کنند، پس رفتار در سه سکو یکی می‌ماند.
//!
//! آنچه ذخیره می‌شود فقط «چه بود» است، نه «چه شد» — تا قطعِ اتصال دقیقاً همان
//! حالتِ قبلیِ کاربر را برگرداند، نه اینکه پروکسیِ خودش را خاموش کند.

use crate::log::DiagnosticsLog;
use std::process::Command;
use std::sync::Mutex;

const TAG: &str = "sysproxy";

/// حالتِ قبلی، تا `disable` بتواند دقیقاً همان را برگرداند.
struct Saved {
    services: Vec<String>,
    host: String,
    port: u16,
    was_on: bool,
}

static SAVED: Mutex<Option<Saved>> = Mutex::new(None);

/// سرویس‌های شبکهٔ فعال — «Wi‑Fi» یا «Ethernet» یا هر دو.
///
/// `scutil --nwi` سرویس‌هایی را می‌دهد که واقعاً آدرس دارند، نه سرویس‌هایی که
/// فقط تعریف شده‌اند. روی لپ‌تاپی که با کابل وصل است، این تفاوت یعنی پروکسی
/// روی هر دو گذاشته می‌شود، نه فقط روی Wi‑Fi.
fn active_services() -> Vec<String> {
    let Ok(out) = Command::new("scutil").args(["--nwi"]).output() else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut services: Vec<String> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Network interfaces: ") {
            for iface in rest.split_whitespace() {
                // نامِ رابط به نامِ سرویسِ قابل‌قبولِ networksetup نگاشت می‌شود.
                //
                // `split_whitespace()` یک `&str` می‌دهد. با `match *iface` الگوی
                // `"en0"` یک `str` می‌شد و بازوهای دیگر `&str` — کامپایلر
                // ناسازگاری نوع می‌داد (E0308 و E0277 برای `str` ناشناخته).
                services.push(match iface {
                    "en0" => "Wi-Fi".to_string(),
                    "en1" => "Ethernet".to_string(),
                    other => (*other).to_string(),
                });
            }
        }
    }
    services.sort();
    services.dedup();
    services
}

/// `networksetup -getwebproxy <service>` — آیا روشن است؟
fn webproxy_on(service: &str) -> bool {
    Command::new("networksetup")
        .args(["-getwebproxy", service])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some_and(|o| String::from_utf8_lossy(&o.stdout).contains("Enabled: Yes"))
}

fn set(service: &str, host: &str, port: u16) -> bool {
    let port = port.to_string();
    // امن و هم برای HTTP و هم برای HTTPS: بدون دومی، مرورگرها فقط صفحه‌های
    // `http://` را از تونل می‌برند و `https://` مستقیم می‌رود.
    [
        vec!["-setwebproxy", service, host, port.as_str()],
        vec!["-setsecurewebproxy", service, host, port.as_str()],
    ]
    .iter()
    .all(|args| {
        Command::new("networksetup")
            .args(args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

fn clear(service: &str) -> bool {
    ["-setwebproxystate", "-setsecurewebproxystate"]
        .iter()
        .all(|flag| {
            Command::new("networksetup")
                .args([flag, service, "off"])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
}

/// پروکسیِ سیستم را روی پلِ محلی می‌گذارد. `false` یعنی شکست — و `state.rs` آن
/// را یک اتصالِ ناموفق می‌شمارد، نه یک هشدار.
pub fn enable(http_port: u16, socks_port: u16) -> bool {
    // مثل ویندوز و لینوکس، فقط پورتِ HTTP استفاده می‌شود.
    let _ = socks_port;
    let services = active_services();
    if services.is_empty() {
        DiagnosticsLog::w(
            TAG,
            "No active network service found — the system proxy stays off, so the \
             tunnel will only be reachable through the app's own port.",
        );
        return false;
    }

    // اینکه پروکسی از قبل روشن بوده یا نه، **پیش** از عوض کردنش پرسیده می‌شود.
    let was_on = services.iter().any(|s| webproxy_on(s));

    let mut any = false;
    for service in &services {
        if set(service, "127.0.0.1", http_port) {
            any = true;
        } else {
            DiagnosticsLog::w(
                TAG,
                &format!("Could not set the proxy on '{service}' — networksetup refused."),
            );
        }
    }
    if !any {
        return false;
    }

    if let Ok(mut slot) = SAVED.lock() {
        *slot = Some(Saved {
            services,
            host: "127.0.0.1".to_string(),
            port: http_port,
            was_on,
        });
    }
    DiagnosticsLog::i(TAG, "System proxy set to the local bridge.");
    true
}

/// بازگرداندنِ دقیقِ حالتِ قبلیِ کاربر.
///
/// اگر پروکسی از قبل روشن بوده، خاموشش نمی‌کنیم — فقط همان را برمی‌گردانیم،
/// چون خاموش کردنِ تنظیمِ خودِ او بدترین حالت است.
pub fn disable() -> bool {
    let Ok(mut slot) = SAVED.lock() else {
        return false;
    };
    let Some(saved) = slot.take() else {
        return false;
    };
    let mut ok = true;
    for service in &saved.services {
        ok &= if saved.was_on {
            set(service, &saved.host, saved.port)
        } else {
            clear(service)
        };
    }
    ok
}

/// اگر برنامه کرش کرده و پروکسی روشن مانده، برگرداندنش.
///
/// همان کاری که نسخهٔ لینوکس می‌کند، با همان دلیل: یک پروکسیِ روشن به
/// 127.0.0.1 که چیزی پشتش نیست، اینترنتِ کاربر را می‌برد.
pub fn recover_stale() -> bool {
    let Ok(mut slot) = SAVED.lock() else {
        return false;
    };
    let Some(saved) = slot.take() else {
        return false;
    };
    let mut ok = true;
    for service in &saved.services {
        if webproxy_on(service) {
            DiagnosticsLog::i(TAG, "Clearing a proxy left behind by a previous run.");
            ok &= clear(service);
        }
    }
    ok
}