//! پروکسی سیستمی لینوکس — معادل WinINET ویندوز.
//!
//! مسیر دادهٔ فعلی روی لینوکس همین است: پل محلیِ `share.rs` روی
//! 127.0.0.1 بالا می‌آید و این ماژول تنظیمِ پروکسیِ سیستم را به آن می‌دواند تا
//! مرورگرها و برنامه‌های GNOME از تونل رد شوند.
//!
//! مکانیزم، GSettings است (`org.gnome.system.proxy`) چون Wayland اجازهٔ
//! ویرایش رجیستری/کلیدهای سراسری را نمی‌دهد و این تنها راهِ استانداردِ
//! GNOME/KDE برای خواندنِ «system proxy» است.
//!
//! آنچه ذخیره می‌شود فقط «چه بود» است، نه «چه شد» — تا قطعِ اتصال دقیقاً
//! همان حالتِ قبلیِ کاربر را برگرداند، نه اینکه پروکسیِ خودش را خاموش کند.

use crate::log::DiagnosticsLog;
use parking_lot::Mutex;
use std::process::Command;
use std::sync::OnceLock;

const TAG: &str = "sysproxy";
const SCHEMA: &str = "org.gnome.system.proxy";

/// پل محلی روی لوپ‌بک است، پس هیچ‌وقت نباید خودش را در bypass بگذاریم —
/// ولی localhost را به‌عنوان استثنا نگه می‌داریم تا درخواست‌های داخلیِ خودِ
/// برنامه (که مستقیم می‌روند) در حلقه نیفتند.
const BYPASS: &[&str] = &["localhost", "127.0.0.1", "::1"];

/// همان چیزی که قبل از دست‌زدن ذخیره می‌کنیم.
#[derive(Debug, Default, Clone)]
struct SavedProxy {
    mode: String,
    http_host: String,
    http_port: u16,
    https_host: String,
    https_port: u16,
    socks_host: String,
    socks_port: u16,
    ignore: Vec<String>,
}

fn saved() -> &'static Mutex<Option<SavedProxy>> {
    static CELL: OnceLock<Mutex<Option<SavedProxy>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

/// `'none'` → `none`؛ رشته‌های دیگر دست‌نخورده می‌مانند.
fn unquote(raw: &str) -> String {
    raw.trim().trim_matches('\'').to_string()
}

fn quote(value: &str) -> String {
    format!("'{value}'")
}

/// یک کلید GSettings را می‌خواند؛ `None` یعنی کلید یا اسکیما نیست.
///
/// `key` مسیرِ کاملِ کلید است — مثلاً `http.host`. نکتهٔ مهم اینجاست:
/// در `org.gnome.system.proxy` کلیدهای هر پروتکل **زیرشاخهٔ جدا** دارند
/// (`org.gnome.system.proxy.http.host`)، نه کلیدِ نقطه‌دار. نوشتنِ
/// `http.host` روی اسکیمای ریشه با `No such key` رد می‌شود — و همین بود
/// دلیلِ اینکه `enable()` همیشه خطا می‌داد و مرورگرها خودکار از تونل رد
/// نمی‌شدند.
fn get(key: &str) -> Option<String> {
    let out = Command::new("gsettings")
        .args(["get", &schema_for(key), leaf(key)])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| unquote(&String::from_utf8_lossy(&out.stdout)))
}

fn set(key: &str, value: &str) -> bool {
    Command::new("gsettings")
        .args(["set", &schema_for(key), leaf(key), value])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// `http.host` → `org.gnome.system.proxy.http`؛ `leaf` → `host`.
fn schema_for(key: &str) -> String {
    match key.split_once('.') {
        Some((proto, _)) => format!("{SCHEMA}.{proto}"),
        None => SCHEMA.to_string(),
    }
}

fn leaf(key: &str) -> &str {
    key.split_once('.').map_or(key, |(_, rest)| rest)
}

fn port(raw: &str) -> u16 {
    raw.trim().parse().unwrap_or(0)
}

/// فهرست رشته‌ای GSettings است: `['a', 'b']`.
fn quote_list(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|s| format!("'{s}'"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// آیا اصلاً دسکتاپِ GNOME در حال اجراست؟ بدون آن `gsettings` بی‌معناست و
/// مسیر دادهٔ درست، TUN است نه پروکسیِ سیستمی.
fn gsettings_available() -> bool {
    static CELL: OnceLock<bool> = OnceLock::new();
    *CELL.get_or_init(|| get("mode").is_some())
}

fn snapshot() -> SavedProxy {
    SavedProxy {
        mode: get("mode").unwrap_or_else(|| "none".into()),
        http_host: get("http.host").unwrap_or_default(),
        http_port: port(&get("http.port").unwrap_or_default()),
        https_host: get("https.host").unwrap_or_default(),
        https_port: port(&get("https.port").unwrap_or_default()),
        socks_host: get("socks.host").unwrap_or_default(),
        socks_port: port(&get("socks.port").unwrap_or_default()),
        ignore: get("ignore-hosts")
            .map(|raw| {
                raw.trim()
                    .trim_matches(['[', ']'])
                    .split(',')
                    .map(|s| unquote(s))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn host_port(host: &str, port: u16) -> String {
    format!("{host}:{port}")
}

/// فعال‌سازی پروکسی سیستمی روی پل محلی. `true` یعنی ثبت شد.
pub fn enable(http_port: u16, socks_port: u16) -> bool {
    if !gsettings_available() {
        DiagnosticsLog::w(
            TAG,
            "No GNOME session (gsettings unavailable) — system proxy is not set. \
             Traffic needs the TUN path or a browser pointed at 127.0.0.1 manually.",
        );
        return false;
    }

    {
        let mut slot = saved().lock();
        if slot.is_none() {
            *slot = Some(snapshot());
        }
    }

    let ignore = BYPASS.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    let ok = set("http.host", &quote("127.0.0.1"))
        && set("http.port", &http_port.to_string())
        && set("https.host", &quote("127.0.0.1"))
        && set("https.port", &http_port.to_string())
        && set("socks.host", &quote("127.0.0.1"))
        && set("socks.port", &socks_port.to_string())
        && set("ignore-hosts", &quote_list(&ignore))
        && set("mode", "'manual'");

    if !ok {
        // Never leave a half-written configuration behind.
        let _ = disable();
        return false;
    }

    DiagnosticsLog::i(
        TAG,
        &format!(
            "System proxy enabled -> http={} socks={} (ignore: {BYPASS:?})",
            host_port("127.0.0.1", http_port),
            host_port("127.0.0.1", socks_port),
        ),
    );
    true
}

/// فقط نشستِ قطع‌شدهٔ Aether را جمع می‌کند. اجرای عادیِ برنامه نباید
/// پروکسیِ نامرتبطِ کاربر را دست بزند.
pub fn recover_stale() -> bool {
    if get("mode").as_deref() != Some("manual") {
        return true;
    }
    let points_at_us = get("http.host").as_deref() == Some("127.0.0.1")
        || get("socks.host").as_deref() == Some("127.0.0.1");
    if points_at_us {
        disable()
    } else {
        true
    }
}

/// غیرفعال‌سازی — در قطع اتصال، خطا و خروج برنامه صدا زنه می‌شود.
///
/// اگر پشتیبانِ همین نشست موجود باشد دقیقاً همان را برمی‌گرداند؛ وگرنه فقط
/// چیزی را که *قطعاً* مال خودمان است پاک می‌کند.
pub fn disable() -> bool {
    let previous = saved().lock().take();

    let ok = match previous {
        Some(p) => {
            set("mode", &quote(&p.mode))
                && set("http.host", &quote(&p.http_host))
                && set("http.port", &p.http_port.to_string())
                && set("https.host", &quote(&p.https_host))
                && set("https.port", &p.https_port.to_string())
                && set("socks.host", &quote(&p.socks_host))
                && set("socks.port", &p.socks_port.to_string())
                && set("ignore-hosts", &quote_list(&p.ignore))
        }
        // Crash recovery: only touch a proxy unmistakably pointing at our bridge.
        None => {
            let port = crate::engine::SHARE_HTTP_PORT.to_string();
            let points_at_us =
                get("http.host").as_deref() == Some("127.0.0.1") && get("http.port").as_deref() == Some(port.as_str());
            if points_at_us {
                set("mode", "'none'")
            } else {
                true
            }
        }
    };

    DiagnosticsLog::i(
        TAG,
        if ok {
            "System proxy settings restored to their pre-Aether state."
        } else {
            "Could not fully restore the system proxy settings."
        },
    );
    ok
}
