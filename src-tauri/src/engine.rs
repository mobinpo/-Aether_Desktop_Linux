//! پورت ۱:۱ از `core/AetherProcess.kt`.
//!
//! تمام رفتارهایی که در ۱.۲.۲ روی اندروید درست شدند عیناً حفظ شده‌اند:
//!   * درنگ کردن (drain) خروجی موتور تا لوله پر نشود
//!   * انتظار قابل‌قطع (interruptible) به‌جای polling
//!   * SIGTERM کوتاه (250ms) و سپس kill قطعی
//!   * منتظر ماندن برای آزادشدن پورت SOCKS5 محلی پیش از اجرای بعدی

use crate::log::DiagnosticsLog;
use crate::profile::{ConnectionProfile, CoreCaps};
use anyhow::{anyhow, Result};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// همان پورتی که موتور در اندروید باز می‌کند. استیج ۱ همیشه همین است.
pub const LOCAL_SOCKS_PORT: u16 = 1819;

/// پورتی که کل خط لولهٔ زنجیره‌ای `Aether → Psiphon` بیرون می‌دهد.
///
/// همان عددِ `TunnelConfig.CHAIN_SOCKS_PORT` اندروید است تا لاگ و مستندات دو
/// سکو یکی بمانند — ولی **معنایش کمی متفاوت است، و این عمدی است**: در اندروید
/// این پورت را `PsiphonSocksFront` می‌گیرد (لایه‌ای که UDP/udpgw/DNS را حمل
/// می‌کند و Psiphon خودش روی ۱۸۲۷ می‌نشیند)، چون tun2socks آن‌جا
/// `UDP ASSOCIATE` می‌خواهد. مسیر دادهٔ ویندوز پروکسی سیستمی WinINET است، یعنی
/// از بنیاد TCP-only، و فقط `CONNECT` لازم دارد؛ پس Psiphon مستقیم همین پورت را
/// می‌گیرد و آن لایهٔ میانی وجود ندارد. مفصل در `psiphon.rs`.
pub const CHAIN_SOCKS_PORT: u16 = 1825;
/// ۱.۲.۲: 10808/10809 با v2rayNG تداخل داشت، پس به 10810/10811 منتقل شد.
pub const SHARE_SOCKS_PORT: u16 = 10810;
pub const SHARE_HTTP_PORT: u16 = 10811;

const GRACEFUL_EXIT_MS: u64 = 250;

/// پورتی که **خروجی** خط لولهٔ فعال است: هرچه مسیر داده و خودآزما باید به آن
/// وصل شوند.
///
/// در نشست عادی همان [LOCAL_SOCKS_PORT] است. در نشست زنجیره‌ای، پس از بالا
/// آمدن استیج ۲، روی [CHAIN_SOCKS_PORT] تنظیم می‌شود.
///
/// ریشه‌ای که این متغیر برایش وجود دارد: پیش از این، `share.rs`، `probe.rs` و
/// `diagnostics.rs` هر سه مستقیم به `LOCAL_SOCKS_PORT` سیم‌کشی شده بودند. در یک
/// نشست زنجیره‌ای این یعنی پل و خودآزما به **استیج ۱** وصل می‌شدند — یعنی تونل
/// بالا می‌آمد، نشانِ IP کشور هاپ اول را نشان می‌داد و کاربر از خروجی اِتِر
/// بیرون می‌رفت، درست همان چیزی که زنجیره برای عوض‌کردنش هست. یک منبع حقیقت،
/// یک بار تنظیم، بدون تغییر امضای هیچ تابعی.
static EXIT_SOCKS_PORT: AtomicU16 = AtomicU16::new(LOCAL_SOCKS_PORT);

/// پورت خروجیِ خط لولهٔ فعال.
pub fn exit_socks_port() -> u16 {
    EXIT_SOCKS_PORT.load(Ordering::Relaxed)
}

/// پورت خروجی را تنظیم می‌کند (استیج ۲ بالا آمد).
pub fn set_exit_socks_port(port: u16) {
    let previous = EXIT_SOCKS_PORT.swap(port, Ordering::Relaxed);
    if previous != port {
        // The latency probe's warm connection belongs to the OLD pipeline. Keeping
        // it would time a dead path and publish its timeout as the user's ping.
        crate::ping::reset();
        DiagnosticsLog::i(
            "engine",
            &format!("Pipeline exit SOCKS5 port: {previous} -> {port}"),
        );
    }
}

/// بازگشت به موتور تنها — در هر قطع اتصال، شکست و چرخش نردبان صدا زده می‌شود.
///
/// اگر جا بیفتد، یک نشست عادیِ بعدی به پورت استیج ۲ که دیگر وجود ندارد وصل
/// می‌ماند و «متصل ولی هیچ سایتی باز نمی‌شود» برمی‌گردد.
pub fn reset_exit_socks_port() {
    set_exit_socks_port(LOCAL_SOCKS_PORT);
}

/// نسخهٔ هستهٔ همراه برنامه — از فایل CORE_VERSION کنار aether.exe خوانده
/// می‌شود (همان فایلی که پنل About نشان می‌دهد). در صورت هر ابهامی (0،0)
/// برمی‌گردد تا رفتار محافظه‌کارانه باشد.
fn engine_core_version(exe: &Path) -> (u32, u32) {
    let Some(dir) = exe.parent() else { return (0, 0) };
    let Ok(raw) = std::fs::read_to_string(dir.join("CORE_VERSION")) else {
        return (0, 0);
    };
    let mut parts = raw.trim().trim_start_matches('v').split('.');
    let major: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor)
}

/// آیا هستهٔ همراه برنامه سوییچ --log-level (نسخهٔ 1.4.0 به بعد) را می‌فهمد؟
/// در صورت هر ابهامی محافظه‌کارانه false برمی‌گردد تا هسته‌های قدیمی با
/// فلگ ناشناخته از کار نیفتند.
fn engine_supports_log_level(exe: &Path) -> bool {
    engine_core_version(exe) >= (1, 4)
}

/// v10: قابلیت‌های هستهٔ همراه — Zero Trust / routing / --dns فقط از 1.5.0.
/// v11: پروکسی بالادست، تشخیص نام میزبان و بازثبت هویت فقط از 1.7.0.
pub fn engine_caps(exe: &Path) -> CoreCaps {
    let (major, minor) = engine_core_version(exe);
    CoreCaps::for_version(major, minor)
}

pub struct AetherProcess {
    exe: PathBuf,
    working_dir: PathBuf,
    child: Option<Child>,
}

impl AetherProcess {
    pub fn new(install_dir: &Path, working_dir: &Path) -> Self {
        let bundled_dir = install_dir.join("engine");
        let exe = match prepare_runtime_engine(&bundled_dir, working_dir) {
            Ok(exe) => exe,
            Err(e) => {
                DiagnosticsLog::w(
                    "engine",
                    &format!("Could not stage the engine in a writable folder ({e}); running it from the install folder."),
                );
                bundled_dir.join("aether.exe")
            }
        };
        Self {
            exe,
            working_dir: working_dir.to_path_buf(),
            child: None,
        }
    }

    /// Launches the engine for one rung of the ladder.
    ///
    /// `rung_budget_ms` is how long the caller ([`crate::state`]) will wait for
    /// the SOCKS5 port before it tears this attempt down.
    ///
    /// ## 1.2.3-p3: the budget has to be told to the engine, not kept secret
    ///
    /// It was not passed before, and the two sides disagreed badly. The app gave
    /// the first rung 35 s (`FIRST_PASS_MAX_MS`); the engine's own turbo scan
    /// budget is 45 s and it only STARTS after loading the identity, an optional
    /// ECH lookup and a cached-gateway verification. So on any network that
    /// needed a real scan, the first rung was mathematically guaranteed to be
    /// killed - with the scan roughly three quarters finished and its results
    /// thrown away - and the second rung started the same doomed scan from
    /// scratch. The field log shows precisely that: two attempts, 97 seconds,
    /// `scan deadline reached with no gateway`, nothing learned.
    ///
    /// Now the engine gets the deadline it is actually being held to and shortens
    /// its scan to fit, so a rung either finishes its scan or reports honestly
    /// that it could not - and the ladder advances immediately instead of after
    /// a stopwatch runs out.
    pub fn start(
        &mut self,
        profile: &ConnectionProfile,
        rung_budget_ms: Option<u64>,
    ) -> Result<()> {
        if !self.exe.exists() {
            return Err(anyhow!("Engine binary missing: {}", self.exe.display()));
        }

        // v10: فلگ‌های هستهٔ 1.5.0 (Zero Trust / routing / dns) فقط وقتی
        // فرستاده می‌شوند که هستهٔ همراه واقعاً آن‌ها را بفهمد.
        let caps = engine_caps(&self.exe);
        let mut args = profile.to_args_with_caps(caps);
        // لاگر جدید هستهٔ 1.4.0 متغیر RUST_LOG را نادیده می‌گیرد و فقط از
        // سوییچ رسمی خودش دستور می‌گیرد (لاگ v12 این را ثابت کرد: هیچ
        // خط debug چاپ نشد). سطح trace تنها راه دیدن عملیاتی است که
        // بلافاصله بعد از sysprofile با code 5 می‌میرد.
        // v16 (سرعت/پینگ): سطح trace فقط برای شکار باگ code 5 لازم بود و در
        // مسیر داده سربار جدی دارد؛ حالا که ریشه رفع شد به info (پیش‌فرضی که
        // v9 با آن سریع بود) برمی‌گردیم. همچنین sysprofile خودکار هستهٔ
        // 1.4.0 روی این سیستم پروفایل Medium با بافرهای کوچک
        // (netstack 256KB/64KB) انتخاب می‌کند که نسبت به هستهٔ 1.3.0 سرعت را
        // پایین می‌آورد؛ با --perf high بافرهای بزرگ و همروندی کامل اسکن
        // برمی‌گردد. (هر دو فلاگ فقط برای هستهٔ 1.4 به بالا فرستاده می‌شود.)
        if engine_supports_log_level(&self.exe) {
            args.insert(0, "--log-level".into());
            args.insert(1, "info".into());
            args.insert(2, "--perf".into());
            args.insert(3, "high".into());
        }
        // ریشهٔ قطعی خطای Access is denied (code 5): تست‌های تشخیصی روی
        // سیستم کاربر ثابت کرد هستهٔ 1.4.0 با CWD=ریشهٔ پوشهٔ داده می‌میرد
        // (T1/T2) ولی با CWD=پوشهٔ خود موتور کامل وصل می‌شود (T3).
        // پس موتور را در پوشهٔ خودش اجرا می‌کنیم؛ فایل‌های هویت/کانفیگ هم
        // در prepare_runtime_engine به همین پوشه منتقل می‌شوند.
        let run_dir = self
            .exe
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.working_dir.clone());
        let mut cmd = Command::new(&self.exe);
        cmd.args(&args)
            .current_dir(&run_dir)
            // v11: متغیرهای هستهٔ 1.7.0 هم مثل فلگ‌ها گِیت شده‌اند.
            .envs(profile.to_env_with_caps(caps))
            .envs(scan_budget_env(rung_budget_ms))
            .env("HOME", &run_dir)
            .env("TMPDIR", &run_dir)
            // هستهٔ 1.4.0 بلافاصله بعد از مرحلهٔ جدید sysprofile با
            // Error: Io(code 5, Access is denied) خارج می‌شود. این دو متغیر
            // backtrace کامل از خود هسته در صورت خطا در کنسول لاگ برنامه ثبت شود؛
            // سطح عادی info از تولید هزاران خط TLS در مسیر موفق جلوگیری می‌کند.
            // (برای نسخه‌های قدیمی‌تر هسته بی‌ضررند و نادیده گرفته می‌شوند.)
            .env("RUST_BACKTRACE", "full")
            .env("RUST_LOG", "info")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // معادل ویندوزیِ اینکه در اندروید فرآیند headless است: پنجرهٔ کنسول
        // نباید جلوی کاربر بالا بیاید.
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd.spawn()?;

        // موتور بدون کنسول اجرا می‌شود (CREATE_NO_WINDOW). تست cmd کاربر ثابت
        // کرد هستهٔ 1.4.0 در کنسول واقعی کامل وصل می‌شود و سؤال تعاملی
        // quick-reconnect ([Y/n]) می‌پرسد؛ زیر برنامه بدون stdin معتبر، دسترسی
        // کنسولی هسته با Access is denied (code 5) شکست می‌خورد.
        // اینجا یک stdin معتبر می‌دهیم و جواب پیش‌فرض «بله» را می‌نویسیم؛
        // با بسته‌شدن pipe، خواندن‌های بعدی EOF تمیز می‌گیرند نه خطا.
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(b"y\n");
        }

        DiagnosticsLog::i("engine", &format!("Spawned aether.exe {}", redact_args(&args).join(" ")));

        // درنگ کردن stdout و stderr — دقیقاً مثل ترد «aether-log» در اندروید.
        if let Some(out) = child.stdout.take() { spawn_drain(out); }
        if let Some(err) = child.stderr.take() { spawn_drain(err); }

        self.child = Some(child);
        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
        match self.child.as_mut() {
            Some(c) => matches!(c.try_wait(), Ok(None)),
            None => false,
        }
    }

    /// معادل `awaitExit`: مسدود می‌ماند تا خروج موتور یا اتمام مهلت.
    /// برخلاف نسخهٔ قبلی اندروید، هیچ polling دومّینی‌ای در کار نیست.
    pub fn await_exit(&mut self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.child.as_mut().map(|c| c.try_wait()) {
                None | Some(Ok(Some(_))) => return true,
                Some(Ok(None)) => std::thread::sleep(Duration::from_millis(20)),
                Some(Err(_)) => return false,
            }
        }
        false
    }

    /// معادل `stop()`: خاتمهٔ مؤدبانه، سپس kill قطعی پس از 250ms.
    /// برگشت از این تابع یعنی فرآیند واقعاً reap شده است.
    pub fn stop(&mut self) {
        let Some(mut child) = self.child.take() else { return };
        let _ = child.kill(); // در ویندوز TerminateProcess فوری است
        let deadline = Instant::now() + Duration::from_millis(GRACEFUL_EXIT_MS);
        while Instant::now() < deadline {
            if let Ok(Some(_)) = child.try_wait() { break; }
            std::thread::sleep(Duration::from_millis(10));
        }
        let _ = child.wait();
        DiagnosticsLog::w("engine", "Engine stopped and reaped.");
    }
}

impl Drop for AetherProcess {
    fn drop(&mut self) { self.stop(); }
}

/// ریشهٔ باگ «روی هیچ پروتکلی کانکت نمی‌شود» (هستهٔ 1.4.x):
/// هستهٔ جدید بلافاصله بعد از شروع، وضعیت خودش (کش quick-reconnect و…) را
/// کنار فایل اجرایی‌اش می‌نویسد. وقتی موتور از `C:\Program Files\…\engine`
/// اجرا شود، آن پوشه برای فرآیندِ بدون Administrator فقط‌خواندنی است و موتور
/// در همان میلی‌ثانیهٔ اول با `Io(Os { code: 5 … Access is denied })` می‌میرد
/// — دقیقاً امضای لاگ کاربر (همهٔ پروتکل‌ها، خروج فوری، قبل از بازشدن SOCKS5).
///
/// رفع ریشه‌ای: موتور در اولین اجرا به پوشهٔ دادهٔ کاربر (قابل‌نوشتن) کپی و
/// همیشه از همان‌جا اجرا می‌شود تا هر نوشتنِ «کنار exe» مجاز باشد. این کار
/// نسخه‌های آیندهٔ هسته را هم در برابر همین کلاس خطا بیمه می‌کند.
/// اگر کپی به هر دلیلی شکست بخورد، رفتار قدیمی (اجرای مستقیم از پوشهٔ نصب)
/// حفظ می‌شود تا هیچ‌وقت وضع بدتر از قبل نشود.
fn prepare_runtime_engine(bundled_dir: &Path, working_dir: &Path) -> Result<PathBuf> {
    let runtime_dir = working_dir.join("engine");
    if runtime_dir == *bundled_dir {
        let exe = runtime_dir.join("aether.exe");
        return if exe.exists() {
            Ok(exe)
        } else {
            Err(anyhow!("Engine binary missing: {}", exe.display()))
        };
    }
    std::fs::create_dir_all(&runtime_dir)?;
    for entry in std::fs::read_dir(bundled_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let src = entry.path();
        let dst = runtime_dir.join(entry.file_name());
        if !runtime_copy_is_fresh(&src, &dst) {
            std::fs::copy(&src, &dst)?;
        }
    }
    // هویت ثبت‌شدهٔ قبلی کاربر (اگر در ریشهٔ پوشهٔ داده باشد) یک‌بار به
    // پوشهٔ اجرای موتور منتقل می‌شود تا دوباره ثبت‌نام لازم نشود.
    // (فایل‌های دیگر مثل masque/lastconn عمداً کپی نمی‌شوند؛ هستهٔ
    // جدید خودش نسخهٔ سالم می‌سازد.)
    for name in ["aether.toml", "aether-secondary.toml"] {
        let src = working_dir.join(name);
        let dst = runtime_dir.join(name);
        if src.is_file() && !dst.exists() {
            let _ = std::fs::copy(&src, &dst);
        }
    }
    let exe = runtime_dir.join("aether.exe");
    if !exe.exists() {
        return Err(anyhow!(
            "Engine binary missing: {}",
            bundled_dir.join("aether.exe").display()
        ));
    }
    Ok(exe)
}

/// فقط وقتی دوباره کپی می‌کنیم که نسخهٔ نصب‌شده عوض شده باشد (آپدیت برنامه).
fn runtime_copy_is_fresh(src: &Path, dst: &Path) -> bool {
    let (Ok(a), Ok(b)) = (std::fs::metadata(src), std::fs::metadata(dst)) else {
        return false;
    };
    if a.len() != b.len() {
        return false;
    }
    matches!((a.modified(), b.modified()), (Ok(s), Ok(d)) if d >= s)
}

/// v10 سخت‌سازی امنیتی: مقدار فلگ‌های محرمانه در لاگ ماندگار پوشانده می‌شود.
/// خود فلگ دیده می‌شود (برای عیب‌یابی) ولی راز/توکن/ایمیل هرگز در
/// فایل لاگ چرخان ثبت نمی‌شود (همان قاعدهٔ ماسک IP در diagnostics.rs).
fn redact_args(args: &[String]) -> Vec<String> {
    // v11: مقدار --upstream می‌تواند user:pass پروکسی کاربر را داشته باشد.
    const SENSITIVE: [&str; 4] =
        ["--access-secret", "--access-token", "--access-email", "--upstream"];
    let mut out = Vec::with_capacity(args.len());
    let mut mask_next = false;
    for a in args {
        if mask_next {
            out.push("••••••".to_string());
            mask_next = false;
            continue;
        }
        if SENSITIVE.contains(&a.as_str()) {
            mask_next = true;
        }
        out.push(a.clone());
    }
    out
}

fn spawn_drain<R: std::io::Read + Send + 'static>(reader: R) {
    std::thread::Builder::new()
        .name("aether-log".into())
        .spawn(move || {
            for line in BufReader::new(reader).lines().map_while(Result::ok) {
                // مثل اندروید: خروجی موتور فقط به لاگ خصوصیِ برنامه می‌رود،
                // نه به جایی که بقیهٔ سیستم بخواند (معادل ممنوعیت Logcat).
                DiagnosticsLog::d("engine", &line);
            }
            DiagnosticsLog::w("engine", "Engine output stream closed.");
        })
        .ok();
}

/// Everything the engine has to do before its endpoint scan can even begin:
/// load or provision the identity, optionally look up an ECHConfigList, verify
/// the cached gateway, and afterwards build the tunnel, validate the data plane
/// and open SOCKS5. Measured off the field log, generously rounded up.
const ENGINE_SETUP_RESERVE_MS: u64 = 14_000;

/// Never hand the engine a scan window so short that it cannot even try the
/// documented gateway seeds.
const MIN_SCAN_BUDGET_MS: u64 = 8_000;

/// Translates this rung's wall-clock budget into the scan budget the engine
/// understands. See [`AetherProcess::start`] for why this exists.
fn scan_budget_env(rung_budget_ms: Option<u64>) -> Vec<(String, String)> {
    let Some(total) = rung_budget_ms else {
        return Vec::new();
    };
    let budget = total
        .saturating_sub(ENGINE_SETUP_RESERVE_MS)
        .max(MIN_SCAN_BUDGET_MS);
    DiagnosticsLog::i(
        "engine",
        &format!(
            "Rung budget {}s → engine endpoint-scan budget {}s (the rest is identity, gateway check and tunnel setup).",
            total / 1000,
            budget / 1000
        ),
    );
    vec![("AETHER_SCAN_BUDGET_MS".to_string(), budget.to_string())]
}

/// معادل `PortProbe.kt`: پیش از اجرای موتور جدید، منتظر آزادشدن پورت می‌مانیم.
/// این همان ریشهٔ باگ «تعویض پروتکل طول می‌کشد» در ۱.۲.۱ بود.
pub fn wait_for_port_release(port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_err() { return true; }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}
