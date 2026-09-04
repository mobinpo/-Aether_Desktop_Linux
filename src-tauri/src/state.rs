//! پورت از `core/AetherController.kt` + `vpn/AetherVpnService.kt` + `model/ConnectionState.kt`.
//!
//! ریشهٔ باگ «هیچ پروتکلی کانکت نمی‌شود» در نسخهٔ قبلی دسکتاپ:
//! `connect()` بلافاصله بعد از اجرای موتور، `Tunnel::establish` را صدا می‌زد؛
//! ساخت آداپتور Wintun بدون دسترسی Administrator شکست می‌خورد، خطا از
//! `connect()` بیرون می‌رفت و ماشین حالت برای همیشه روی StartingEngine گیر
//! می‌کرد — در حالی که موتور واقعاً وصل می‌شد (لاگ کاربر: «socks5 server
//! listening on 127.0.0.1:1819» بدون هیچ «I/state: Connecting» بعد از آن).
//!
//! حالا دقیقاً ترتیب اندروید (`connectAttempt`) اجرا می‌شود:
//!   ۱. StartingEngine → آزادشدن پورت → اجرای موتور → Connecting
//!   ۲. انتظار برای بازشدن پورت SOCKS5 (ground truth — همان PortProbe)
//!   ۳. فقط بعد از آن، مسیر داده برپا می‌شود (معادل VpnService.establish):
//!      پل HTTP/SOCKS محلی + پروکسی سیستمی ویندوز؛ Wintun هم اگر ممکن بود
//!      (شکست Wintun دیگر کل اتصال را نمی‌کُشد — فقط یک هشدار لاگ می‌شود).
//!   ۴. Verifying: خودآزمای ۴ مرحله‌ای (Diagnostics.kt) در ترد پس‌زمینه
//!   ۵. فقط بعد از قبولی همهٔ بررسی‌ها، Connected اعلام می‌شود
//!   ۶. شکست هر پله ← پلهٔ بعدی نردبان (معادل runLadder)، نه گیرکردن ابدی.

use crate::diagnostics;
use crate::engine::{self, AetherProcess};
use crate::leakguard::{self, LeakGuard};
use crate::log::DiagnosticsLog;
use crate::ping;
use crate::probe;
use crate::profile::{ConnectionProfile, Protocol};
use crate::psiphon::PsiphonTransport;
use crate::psiphon_health;
use crate::share::ShareBridge;
use crate::smart_auto::{self, Candidate};
use crate::store::ProfileStore;
use crate::sysproxy;
use crate::tun::Tunnel;
use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

const TAG: &str = "state";

/// همان مقادیر اندروید: MAX_RETRIES=3، BACKOFF = 2s/5s/10s.
const DEFAULT_MAX_RETRIES: u32 = 3;
const BACKOFF_MS: [u64; 3] = [2_000, 5_000, 10_000];
/// پنجرهٔ گریس خودآزما — همان `OUTBOUND_GRACE_MS` (شروع سرد warp-in-warp).
const OUTBOUND_GRACE_MS: u64 = 90_000;
/// معادل `PORT_RELEASE_WAIT_MS` اندروید.
const PORT_RELEASE_WAIT_MS: u64 = 3_000;
const WATCHDOG_INTERVAL_SECS: u64 = 30;
/// How often the live latency badge is refreshed.
///
/// It used to be 15s because each measurement dialled a brand new connection
/// through both hops. A measurement is now one keep-alive round trip on a warm
/// session (see [`crate::ping`]), so it costs a single packet each way and the
/// badge can afford to feel live.
const LATENCY_INTERVAL_SECS: u64 = 10;
/// بودجهٔ کل استیج ۲ (دروازهٔ استیج ۱ + دو پاسِ برقراری Psiphon).
///
/// سخاوتمند است چون پاس دوم عمداً از صفر شروع می‌کند: datastore پاک می‌شود و
/// فیلتر کشور برداشته می‌شود. مهلتِ پلهٔ نردبان در این فاز کنار گذاشته می‌شود،
/// وگرنه یک نشست زنجیره‌ای که فقط کُند است پیش از آنکه شانسی داشته باشد رد
/// می‌شود — همان اشتباهی که مستند موبایل «بدترین نتیجهٔ ممکن» می‌خواندش.
const CHAIN_BUDGET_MS: u64 = 430_000;
const WATCHDOG_FAILURE_THRESHOLD: u8 = 3;

/// معادل دقیق `ConnectionState.kt` — همان هشت حالت، همان ترتیب.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionState {
    Disconnected,
    StartingEngine,
    Connecting,
    Verifying,
    Connected,
    Reconnecting,
    Disconnecting,
    Failed,
}

impl ConnectionState {
    pub fn is_busy(self) -> bool {
        matches!(
            self,
            Self::StartingEngine | Self::Connecting | Self::Verifying | Self::Reconnecting | Self::Disconnecting
        )
    }
    pub fn is_active(self) -> bool {
        self.is_busy() || self == Self::Connected
    }
}

/// معادل `IpInfo` در UI اندروید — خوراک نشان «IP + پرچم».
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpEndpoint {
    pub ip: String,
    pub country_code: Option<String>,
    /// true = IP خروجی سرور (از دل تونل)، false = IP واقعی کاربر.
    pub via_tunnel: bool,
}

/// حالت مشترک جست‌وجوی IP — معادل `ipInfo`/`ipLoading` در MainActivity.
struct IpSlot {
    info: Option<IpEndpoint>,
    loading: bool,
    /// شمارندهٔ نسل — نتیجهٔ جست‌وجوهای قدیمی دور ریخته می‌شود.
    session: u64,
}

/// معادل مجموع StateFlow‌هایی که HomeScreen.kt جمع می‌کرد.
/// `PartialEq` is load-bearing, not decoration: `main.rs` only pushes a snapshot
/// to the UI when it differs from the last one it sent. An idle app used to
/// re-serialise and repaint the entire home screen five times a second forever.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub state: ConnectionState,
    pub detail: String,
    pub error: Option<String>,
    pub endpoint: Option<String>,
    pub protocol: Option<String>,
    pub latency_ms: Option<u64>,
    pub uptime_secs: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub share_socks: Option<String>,
    pub share_http: Option<String>,
    pub ip_info: Option<IpEndpoint>,
    pub ip_loading: bool,
    /// v1.2.0 — نتیجهٔ آخرین سنجش نشتی WebRTC. `None` = هنوز سنجیده نشده.
    pub webrtc_leak: Option<bool>,
    /// v1.2.0 — گارد نشتی همین حالا فعال است؟
    pub leak_guard: bool,
}

/// What the user asked for with the last tap on the big button.
///
/// # Why the tap no longer does the work itself
///
/// `toggle_connection` used to run the whole of `connect()` / `disconnect()`
/// while holding the controller mutex. Both are full of slow Windows calls —
/// a network fingerprint (up to 2.4s), waiting for the SOCKS port to be released
/// (up to 3s), `netsh` firewall rules, registry proxy writes, process teardown.
/// The 200ms snapshot tick wants the same mutex, so for several seconds after a
/// tap nothing repainted: the button did not change, the spinner did not start,
/// and the app read as frozen exactly when the user was watching hardest.
///
/// Now a tap records an intent, flips the visible state, and returns instantly.
/// The tick thread performs the work on the next beat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Intent {
    Connect,
    Disconnect,
}

/// Which stage the off-lock preparation thread is preparing for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Prep {
    /// A fresh session: fingerprint the network, then build the whole ladder.
    Plan,
    /// The next rung of an existing ladder: no fingerprinting needed.
    Candidate,
}

/// Result of the off-lock preparation thread.
struct PrepOutcome {
    /// What the pre-connect probes learned. Only meaningful for [`Prep::Plan`].
    fingerprint: smart_auto::NetFingerprint,
}

pub struct AetherController {
    data_dir: PathBuf,
    store: ProfileStore,
    profile: ConnectionProfile,
    state: ConnectionState,
    detail: String,
    error: Option<String>,
    endpoint: Option<String>,
    effective_protocol: Option<Protocol>,
    latency_ms: Option<u64>,
    connected_at: Option<Instant>,
    engine: AetherProcess,
    /// استیج ۲. `Arc` چون ترد راه‌اندازی و ترد چرخش هم به آن نیاز دارند و
    /// `Drop` باید فقط با آزادشدن آخرین ارجاع فرآیند را بکشد.
    psiphon: Arc<PsiphonTransport>,
    /// نتیجهٔ راه‌اندازی زنجیره در ترد پس‌زمینه — حلقهٔ tick مسدود نمی‌شود.
    chain_slot: Option<Arc<Mutex<Option<Result<u16, String>>>>>,
    tunnel: Option<Tunnel>,
    share: ShareBridge,
    sysproxy_on: bool,
    /// v1.2.0 — گارد نشتی WebRTC/UDP این نشست (Drop خودش آزادش می‌کند).
    guard: Option<LeakGuard>,
    /// v1.2.0 — آخرین نتیجهٔ سنجش نشتی، برای نشانِ صفحهٔ اصلی.
    webrtc_leak: Option<bool>,
    /// نردبان تلاش‌ها — معادل `runLadder` در AetherVpnService.kt.
    plan: Vec<Candidate>,
    plan_index: usize,
    /// تلاش‌های اتصال مجدد پشت‌سرهم — معادل `reconnectAttempts`.
    attempts: u32,
    deadline: Option<Instant>,
    reconnect_at: Option<Instant>,
    /// نتیجهٔ خودآزمای در حال اجرا (ترد پس‌زمینه — UI فریز نمی‌شود).
    verify_slot: Option<Arc<Mutex<Option<diagnostics::SelfTestOutcome>>>>,
    ip_slot: Arc<Mutex<IpSlot>>,
    /// پینگ زنده: نتیجهٔ آخرین اندازه‌گیری دوره‌ای در ترد پس‌زمینه.
    latency_slot: Arc<Mutex<Option<u64>>>,
    /// زمان اندازه‌گیری بعدی پینگ.
    latency_probe_at: Option<Instant>,
    /// نتیجهٔ آخرین پروب واچداگ، خارج از حلقهٔ اصلی محاسبه می‌شود.
    watchdog_slot: Arc<Mutex<Option<bool>>>,
    watchdog_probe_at: Option<Instant>,
    watchdog_failures: u8,
    /// Firewall/registry work is deferred out of the IPC command path.
    security_refresh_pending: bool,
    /// The last tap, waiting for the next tick. See [`Intent`].
    pending_intent: Option<Intent>,
    /// Slow pre-launch work running off the controller lock. See [`Prep`].
    prep_slot: Option<Arc<Mutex<Option<PrepOutcome>>>>,
    prep_kind: Prep,
    /// Stage 2's listener once the chain is actually carrying traffic.
    ///
    /// This is what makes the PROTOCOL tile honest: it says `Aether → Psiphon`
    /// only once the Psiphon hop really is the exit, not merely because the
    /// chained backend is selected in Advanced.
    chain_exit_port: Option<u16>,
}

impl AetherController {
    pub fn new(data_dir: &Path) -> Self {
        let store = ProfileStore::new(data_dir);
        let mut profile = store.load();
        profile.normalize();
        let install_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| data_dir.to_path_buf());

        let ip_slot = Arc::new(Mutex::new(IpSlot { info: None, loading: false, session: 0 }));

        let me = Self {
            data_dir: data_dir.to_path_buf(),
            store,
            profile,
            state: ConnectionState::Disconnected,
            detail: String::new(),
            error: None,
            endpoint: None,
            effective_protocol: None,
            latency_ms: None,
            connected_at: None,
            engine: AetherProcess::new(&install_dir, data_dir),
            psiphon: Arc::new(PsiphonTransport::new(&install_dir, data_dir)),
            chain_slot: None,
            tunnel: None,
            share: ShareBridge::new(),
            sysproxy_on: false,
            guard: None,
            webrtc_leak: None,
            plan: Vec::new(),
            plan_index: 0,
            attempts: 0,
            deadline: None,
            reconnect_at: None,
            verify_slot: None,
            ip_slot,
            latency_slot: Arc::new(Mutex::new(None)),
            latency_probe_at: None,
            watchdog_slot: Arc::new(Mutex::new(None)),
            watchdog_probe_at: None,
            watchdog_failures: 0,
            security_refresh_pending: false,
            pending_intent: None,
            prep_slot: None,
            prep_kind: Prep::Plan,
            chain_exit_port: None,
        };

        // Tell the health scorer which destinations are OURS before anything can
        // dial them, so a refused self-probe can never be read as evidence that
        // the exit filters (mobile parity: registerSelfProbes).
        for (host, port) in ping::probe_targets() {
            psiphon_health::register_self_probe(host, port);
        }
        for (host, port) in probe::watchdog_targets() {
            psiphon_health::register_self_probe(host, port);
        }

        // Stale state from a previous crash is cleared on a worker thread.
        //
        // Both calls shell out: `recover_stale` writes the WinINET registry keys
        // and broadcasts a settings change, `purge_stale` runs several
        // `netsh advfirewall` deletions. Together they cost the better part of a
        // second, and they used to run inside the Tauri `setup` hook — which is
        // to say, before the window was allowed to appear. Nothing about them
        // needs to finish before the UI is on screen; they only have to happen
        // before a connection is brought up, and a connection needs a tap.
        std::thread::Builder::new()
            .name("aether-recover".into())
            .spawn(|| {
                sysproxy::recover_stale();
                leakguard::purge_stale();
            })
            .ok();
        // Do not install network-blocking rules during ordinary app startup.
        // The old behavior blocked Windows before a tunnel/bridge existed,
        // which is why reopening the app could kill internet access. The
        // guard is installed only after the SOCKS bridge is ready.
        // معادل LaunchedEffect فاز idle در MainActivity: نمایش IP واقعی کاربر از لحظهٔ اجرا.
        spawn_ip_lookup(me.ip_slot.clone(), false);
        me
    }

    pub fn profile(&self) -> ConnectionProfile {
        self.profile.clone()
    }

    pub fn set_profile(&mut self, profile: ConnectionProfile) -> Result<()> {
        let mut profile = profile;
        profile.normalize();
        // v10: فیلدهای محرمانه «write-only» هستند: get_profile هرگز آن‌ها را
        // برنمی‌گرداند، پس UI معمولاً رشتهٔ خالی می‌فرستد. خالی = «دست نزن»
        // تا رازِ در-حافظهٔ این نشست با هر تغییر تنظیم دیگر پاک نشود.
        if profile.access_secret.is_empty() {
            profile.access_secret = self.profile.access_secret.clone();
        }
        if profile.access_token.is_empty() {
            profile.access_token = self.profile.access_token.clone();
        }
        self.apply_profile(profile)
    }

    /// v10: «بازنشانی به تنظیمات پیش‌فرض» باید اسرارِ در-حافظه را هم واقعاً
    /// پاک کند. `set_profile` رشتهٔ خالی را «دست نزن» تفسیر می‌کند (چون UI
    /// اسرار را پس نمی‌گیرد)، پس Reset مسیر جداگانهٔ خودش را دارد؛ وگرنه
    /// توکن سازمانی پس از Reset بی‌صدا در حافظه زنده می‌ماند.
    pub fn reset_profile(&mut self) -> Result<ConnectionProfile> {
        let fresh = ConnectionProfile::default();
        self.apply_profile(fresh.clone())?;
        DiagnosticsLog::i(TAG, "Profile reset to factory defaults (in-memory Zero Trust secrets cleared).");
        Ok(fresh)
    }

    /// مسیر مشترک ذخیره‌سازی — هرچه از set_profile/reset_profile بیاید.
    fn apply_profile(&mut self, profile: ConnectionProfile) -> Result<()> {
        self.store.save(&profile)?;
        let lan_toggled = profile.lan_share != self.profile.lan_share;
        let guard_toggled = profile.leak_guard != self.profile.leak_guard;
        let kill_toggled = profile.kill_switch != self.profile.kill_switch;
        let ipv6_toggled = profile.ipv6_protection != self.profile.ipv6_protection;
        self.profile = profile;
        // v1.2.0: خاموش/روشن‌کردن گارد نشتی وسط یک اتصالِ فعال باید فوراً
        // اثر کند — نه در اتصال بعدی. کاربری که سوییچ را می‌زند انتظار دارد
        // همان لحظه محافظت شود (یا آزاد شود).
        let safety_changed = guard_toggled || kill_toggled || ipv6_toggled;
        if safety_changed && self.state.is_active() {
            // Do not run reg.exe/netsh.exe while the UI IPC command is waiting.
            // The 200ms controller tick applies it outside the settings click,
            // preventing the white titlebar/freeze seen on safety toggles.
            self.security_refresh_pending = true;
            self.webrtc_leak = None;
        }
        // Root fix for "Share over LAN shows no IP:port": flipping the switch
        // while a connection is active must rebind the bridge immediately
        // (mobile restarts its ShareBridge the same way), so the UI gets the
        // fresh endpoints in the very next snapshot instead of never.
        if lan_toggled && self.state.is_active() {
            if let Err(e) = self.share.start(
                engine::SHARE_SOCKS_PORT,
                engine::SHARE_HTTP_PORT,
                self.profile.lan_share,
            ) {
                DiagnosticsLog::e(TAG, &format!("Bridge restart after LAN toggle failed: {e}"));
            }
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Snapshot {
        let (tun_rx, tun_tx) = self.tunnel.as_ref().map(Tunnel::counters).unwrap_or((0, 0));
        let (br_rx, br_tx) = self.share.traffic();
        let (ip_info, ip_loading) = {
            let g = self.ip_slot.lock();
            (g.info.clone(), g.loading)
        };
        Snapshot {
            state: self.state,
            detail: self.detail.clone(),
            error: self.error.clone(),
            endpoint: self.endpoint.clone(),
            protocol: self.display_protocol(),
            latency_ms: self.latency_ms,
            uptime_secs: self.connected_at.map(|t| t.elapsed().as_secs()).unwrap_or(0),
            rx_bytes: tun_rx + br_rx,
            tx_bytes: tun_tx + br_tx,
            share_socks: self.share.socks_endpoint(),
            share_http: self.share.http_endpoint(),
            ip_info,
            ip_loading,
            webrtc_leak: self.webrtc_leak,
            leak_guard: leakguard::status().engaged,
        }
    }

    /// The label the PROTOCOL tile shows.
    ///
    /// A chained session that has really handed the exit to Psiphon reports the
    /// whole pipeline (`Aether → Psiphon`, byte-for-byte the mobile string).
    /// Anything else — plain Aether, or a chained session whose stage 2 is not
    /// carrying yet — keeps reporting the concrete protocol exactly as before.
    fn display_protocol(&self) -> Option<String> {
        if self.chain_exit_port.is_some() {
            if let Some(label) = self.profile.backend.protocol_label() {
                return Some(label.to_string());
            }
        }
        self.effective_protocol.map(|p| format!("{p:?}").to_uppercase())
    }

    /// معادل `onToggleConnection` — خطای اتصال دیگر به بیرون پرتاب نمی‌شود؛
    /// همیشه به حالت Failed ترجمه می‌شود تا UI هرگز در StartingEngine گیر نکند.
    ///
    /// Records the intent and repaints; the work happens on the next tick. See
    /// [`Intent`] for why this must not block.
    pub fn request_toggle(&mut self) {
        if self.state.is_active() {
            self.pending_intent = Some(Intent::Disconnect);
            self.set_state(ConnectionState::Disconnecting, "Disconnecting…");
        } else {
            self.error = None;
            self.pending_intent = Some(Intent::Connect);
            self.set_state(ConnectionState::StartingEngine, "Starting engine…");
        }
    }

    /// معادل `connect()` سرویس اندروید — فقط برنامه‌ریزی و اجرای پلهٔ اول؛
    /// بقیهٔ مراحل در tick() دنبال می‌شوند.
    fn connect(&mut self) -> Result<()> {
        self.error = None;
        self.attempts = 0;
        self.chain_slot = None;
        self.chain_exit_port = None;
        // هر نشست از موتور شروع می‌شود. اگر این جا بیفتد، یک نشست عادی به پورت
        // استیج ۲ که دیگر وجود ندارد وصل می‌ماند: «متصل ولی هیچ سایتی باز
        // نمی‌شود».
        engine::reset_exit_socks_port();
        // معادل DiagnosticsLog.clear + resetChecks در شروع اتصال اندروید.
        diagnostics::reset_checks();
        // The warm latency session belongs to the pipeline that is going away.
        ping::reset();
        // The visible state is already StartingEngine — request_toggle set it the
        // moment the user tapped, so the button never waits on this method.
        DiagnosticsLog::i(
            TAG,
            &format!(
                "Connect requested — protocol={:?} scan={:?} ip={:?}",
                self.profile.protocol, self.profile.scan_mode, self.profile.ip_version
            ),
        );
        // SmartAuto.kt parity: fingerprint the network before planning. On a
        // filtered network the ladder leads with the hardened anti-DPI
        // candidate, so the plain first pass can no longer waste 35-75s
        // (slow connects) or win with a tunnel that cannot carry real
        // browser traffic afterwards.
        if self.profile.is_chained() && !self.psiphon.is_available() {
            return Err(anyhow::anyhow!(
                "The Psiphon stage is missing from this installation ({}). Reinstall Aether, or set the transport back to Aether.",
                self.psiphon.missing_parts()
            ));
        }
        DiagnosticsLog::i(TAG, &format!("Pipeline: {}", self.profile.backend.pipeline_label()));
        // Fingerprinting and waiting for the port to be released are the two slow
        // steps, and neither may run on the controller lock. They go to a worker
        // thread and the ladder is built when the tick sees the result.
        self.begin_prep(Prep::Plan);
        Ok(())
    }

    /// Runs the slow pre-launch steps off the controller lock.
    ///
    /// Both used to sit directly in the connect path: `network_looks_filtered`
    /// dials two IP literals with a 1.2s timeout each, and `wait_for_port_release`
    /// polls for up to 3s. Nearly six seconds of a held mutex, on every rung of
    /// the ladder — that is the stall the user felt when tapping Connect.
    fn begin_prep(&mut self, kind: Prep) {
        let slot: Arc<Mutex<Option<PrepOutcome>>> = Arc::new(Mutex::new(None));
        self.prep_slot = Some(slot.clone());
        self.prep_kind = kind;
        let fingerprint = kind == Prep::Plan;
        std::thread::Builder::new()
            .name("aether-prep".into())
            .spawn(move || {
                // معادل PortProbe.awaitClosed — ریشهٔ باگ «تعویض پروتکل گیر می‌کند».
                if !engine::wait_for_port_release(
                    engine::LOCAL_SOCKS_PORT,
                    Duration::from_millis(PORT_RELEASE_WAIT_MS),
                ) {
                    DiagnosticsLog::w(
                        TAG,
                        &format!(
                            "Local port {} is still busy after {}s — starting anyway.",
                            engine::LOCAL_SOCKS_PORT,
                            PORT_RELEASE_WAIT_MS / 1000
                        ),
                    );
                }
                // SmartAuto.kt parity: fingerprint the network before planning.
                // 1.2.3-p2 adds the UDP leg. Without it the planner could not
                // tell HTTP/3 from HTTP/2 and defaulted to the slow carrier.
                // 1.2.3-p3: `udp_ok` is a statement about the MASQUE CARRIER, so it
                // is measured against the carrier - a real QUIC round trip to a
                // Cloudflare edge on UDP:443 - not against a DNS query on UDP:53.
                // The old probe answered a different question and got it right for
                // the wrong network: see `probe::quic_carrier_ok`.
                let fp = if fingerprint {
                    smart_auto::NetFingerprint {
                        filtered: probe::network_looks_filtered(),
                        udp_ok: probe::quic_carrier_ok(),
                    }
                } else {
                    smart_auto::NetFingerprint::default()
                };
                *slot.lock() = Some(PrepOutcome { fingerprint: fp });
            })
            .ok();
    }

    /// Builds the ladder once the fingerprint is in, then launches its first rung.
    fn launch_plan(&mut self, fingerprint: smart_auto::NetFingerprint) -> Result<()> {
        // استیج ۱ نردبانِ Smart Auto و سخت‌سازی ضد‌DPI را دست‌نخورده نگه می‌دارد،
        // پس یک نشست زنجیره‌ای همان اثر‌انگشت‌زنی و همان تلاش‌های مجدد نشست عادی
        // را می‌گیرد — ولی بدون مسیر داده و بدون پل.
        let planning_profile = if self.profile.is_chained() {
            self.profile.chained_stage()
        } else {
            self.profile.clone()
        };
        self.plan = smart_auto::build_plan(&planning_profile, fingerprint);
        self.plan_index = 0;
        self.launch_candidate()
    }

    /// اجرای یک پله از نردبان — معادل یک دور `runLadder`.
    ///
    /// Assumes [`begin_prep`] has already waited for the local port, so all this
    /// does is spawn the engine: fast enough to stay on the lock.
    fn launch_candidate(&mut self) -> Result<()> {
        let cand = self.plan[self.plan_index].clone();
        DiagnosticsLog::i(
            TAG,
            &format!("Attempt {}/{} → {}", self.plan_index + 1, self.plan.len(), cand.label),
        );

        self.effective_protocol = Some(cand.profile.protocol);
        // The engine is told how long this rung is allowed to take, so its own
        // endpoint scan is sized to fit inside that window instead of being
        // killed 78% of the way through it. See `engine::AetherProcess::start`.
        self.engine.start(&cand.profile, Some(cand.timeout_ms))?;
        self.deadline = Some(Instant::now() + Duration::from_millis(cand.timeout_ms));
        self.set_state(ConnectionState::Connecting, "Connecting…");
        DiagnosticsLog::i(
            TAG,
            &format!(
                "Waiting for SOCKS5 on 127.0.0.1:{}… (timeout={}s)",
                engine::LOCAL_SOCKS_PORT,
                cand.timeout_ms / 1000
            ),
        );
        Ok(())
    }

    /// معادل بخش establish در connectAttempt — فقط بعد از بازشدن پورت SOCKS5.
    fn bring_up_data_path(&mut self) {
        let profile = self
            .plan
            .get(self.plan_index)
            .map(|c| c.profile.clone())
            .unwrap_or_else(|| self.profile.clone());

        // ۰) گارد نشتی — *قبل* از هر چیز دیگری. ترتیب امنیتی است، نه سلیقه‌ای:
        // تا وقتی مسیر UDP مستقیم باز است نباید مرورگر را به تونل وصل کنیم،
        // وگرنه بین «پروکسی روشن شد» و «گارد نصب شد» یک پنجرهٔ نشتی می‌ماند.
        if self.profile.leak_guard || self.profile.kill_switch || self.profile.ipv6_protection {
            if let Some(mut old_guard) = self.guard.take() {
                old_guard.disarm_without_cleanup();
            }
            self.guard = Some(LeakGuard::engage(&profile));
        } else {
            DiagnosticsLog::w(
                TAG,
                "Leak guard is disabled in the profile — WebRTC may expose your real IP over direct UDP.",
            );
        }

        // ۱) پل محلی HTTP/SOCKS — معادل hev-socks5-tunnel/ShareBridge (مسیر دادهٔ واقعی).
        if let Err(e) = self.share.start(engine::SHARE_SOCKS_PORT, engine::SHARE_HTTP_PORT, profile.lan_share) {
            DiagnosticsLog::e(TAG, &format!("Bridge failed to start: {e}"));
        }

        // ۲) پروکسی سیستمی ویندوز — معادل کارکرد VpnService (کل سیستم از تونل می‌رود).
        self.sysproxy_on = sysproxy::enable(engine::SHARE_HTTP_PORT, engine::SHARE_SOCKS_PORT);

        // ۳) Wintun — اختیاری. شکست آن دیگر اتصال را نمی‌کُشد (رفع ریشه‌ای گیر StartingEngine).
        if self.tunnel.is_none() {
            let wintun = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("engine").join("wintun.dll")))
                .unwrap_or_default();
            match Tunnel::establish(&profile, &wintun) {
                Ok(t) => self.tunnel = Some(t),
                Err(e) => DiagnosticsLog::w(
                    "tun",
                    &format!("Wintun unavailable ({e}); continuing with the system-proxy data path."),
                ),
            }
        }
    }

    /// استیج ۲ را در ترد پس‌زمینه بالا می‌آورد.
    ///
    /// ترتیب کل نکتهٔ ماجراست و عیناً همان `connectExternal` اندروید است:
    ///
    /// ```text
    ///   stage 1  موتور اِتِر → SOCKS5 127.0.0.1:1819   (هنوز هیچ مسیر داده‌ای!)
    ///   stage 2  Psiphon    → SOCKS5 127.0.0.1:1825   از راه 1819 dial می‌کند
    ///   سپس     پل + پروکسی سیستمی → 1825            خروجی = Psiphon
    /// ```
    ///
    /// استیج ۱ **نباید** مسیر داده بسازد: استیج ۲ باید از لوپ‌بک به موتور برسد
    /// در حالی که خود موتور هنوز از شبکهٔ واقعی به اینترنت می‌رسد. اگر پروکسی
    /// سیستمی همین‌جا روشن شود، Psiphon از داخل تونلی بیرون می‌رود که خودش
    /// دارد می‌سازد و همه‌چیز داخل خودش قفل می‌شود.
    ///
    /// مسدودکننده است (تا سه دقیقه در هر پاس)، پس روی ترد خودش می‌رود و
    /// حلقهٔ ۲۰۰ms هرگز فریز نمی‌شود.
    fn begin_chain(&mut self) {
        let slot: Arc<Mutex<Option<Result<u16, String>>>> = Arc::new(Mutex::new(None));
        self.chain_slot = Some(slot.clone());
        // مهلتِ پلهٔ نردبان کنار گذاشته می‌شود: بودجهٔ استیج ۲ مال خودش است.
        self.deadline = Some(Instant::now() + Duration::from_millis(CHAIN_BUDGET_MS));
        let psiphon = self.psiphon.clone();
        let region = self.profile.exit_region.clone();
        let upstream = ConnectionProfile::chain_upstream_url();
        self.set_state(ConnectionState::Connecting, "Starting the Psiphon stage…");
        DiagnosticsLog::i(
            TAG,
            &format!(
                "Chained mode: stage 1 = Aether engine on 127.0.0.1:{}, stage 2 = Psiphon on 127.0.0.1:{}",
                engine::LOCAL_SOCKS_PORT,
                engine::CHAIN_SOCKS_PORT
            ),
        );
        std::thread::Builder::new()
            .name("aether-chain".into())
            .spawn(move || {
                // دروازهٔ استیج ۱ پیش از هر چیز: بدون یک پروکسی SOCKS5 کارکنده،
                // Psiphon سه دقیقه در تاریکی تلاش می‌کند و شکست در جای اشتباه
                // ظاهر می‌شود.
                if !diagnostics::run_proxy_stage(engine::LOCAL_SOCKS_PORT) {
                    *slot.lock() = Some(Err(
                        "Stage 1 (the Aether engine) is not a working SOCKS5 proxy yet".to_string(),
                    ));
                    return;
                }
                let outcome = psiphon
                    .start(&region, &upstream)
                    .map_err(|e| e.to_string());
                *slot.lock() = Some(outcome);
            })
            .ok();
    }

    /// نتیجهٔ استیج ۲ را برمی‌دارد و مسیر داده را به **خروجی زنجیره** می‌چسباند.
    fn poll_chain(&mut self) {
        let outcome = self.chain_slot.as_ref().and_then(|s| s.lock().take());
        let Some(outcome) = outcome else {
            if !self.engine.is_alive() {
                self.chain_slot = None;
                self.advance_or_fail("Stage 1 (the engine) exited while the Psiphon stage was starting");
            } else if self.past_deadline() {
                self.chain_slot = None;
                self.advance_or_fail("The Psiphon stage did not come up within its budget");
            }
            return;
        };
        self.chain_slot = None;
        match outcome {
            Ok(port) => {
                // از این لحظه پل، خودآزما و نشانِ IP همه به استیج ۲ نگاه
                // می‌کنند. این تک‌خط است که خروجی را از اِتِر به Psiphon
                // منتقل می‌کند.
                engine::set_exit_socks_port(port);
                // From here the PROTOCOL tile may honestly say `Aether → Psiphon`.
                self.chain_exit_port = Some(port);
                DiagnosticsLog::i(
                    TAG,
                    &format!("Psiphon stage is up on 127.0.0.1:{port} — bringing up the data path."),
                );
                self.bring_up_data_path();
                self.begin_verification();
            }
            Err(why) => {
                engine::reset_exit_socks_port();
                self.psiphon.stop();
                // یک خطای پیکربندی/آرگومان در استیج ۲ روی **هر** پلهٔ نردبان
                // یکسان می‌افتد: در لاگ میدانی همین چهار پله را با یک پیام
                // سوزاند و کاربر فقط «کانکت نشد» دید. پس همان‌جا و با همان
                // پیام دقیق شکست می‌خوریم، نه با یک نردبانِ محکوم‌به‌شکست.
                if crate::psiphon::is_config_fault(&why) {
                    self.fail(&format!("The Psiphon stage failed: {why}"));
                    return;
                }
                self.advance_or_fail(&format!("The Psiphon stage failed: {why}"));
            }
        }
    }

    /// خودآزمای ۴ مرحله‌ای در ترد پس‌زمینه — حلقهٔ tick هرگز مسدود نمی‌شود.
    fn begin_verification(&mut self) {
        let slot: Arc<Mutex<Option<diagnostics::SelfTestOutcome>>> = Arc::new(Mutex::new(None));
        self.verify_slot = Some(slot.clone());
        let remaining = self
            .deadline
            .map(|d| d.saturating_duration_since(Instant::now()).as_millis() as u64)
            .unwrap_or(OUTBOUND_GRACE_MS);
        // نشست زنجیره‌ای گرم‌شدنِ هر دو هاپ را می‌پردازد، پس پنجرهٔ بلندتر
        // `EXTERNAL_GRACE_MS` را می‌گیرد — همان تفکیک اندروید.
        let ceiling = if self.profile.is_chained() {
            diagnostics::EXTERNAL_GRACE_MS
        } else {
            OUTBOUND_GRACE_MS
        };
        let grace = remaining.clamp(20_000, ceiling);
        std::thread::Builder::new()
            .name("aether-selftest".into())
            .spawn(move || {
                let outcome = diagnostics::self_test(grace);
                *slot.lock() = Some(outcome);
            })
            .ok();
        self.set_state(ConnectionState::Verifying, "Verifying…");
    }

    /// The visible state is already Disconnecting (see [`request_toggle`]); this
    /// runs the slow teardown on the tick thread, off the UI's IPC path.
    fn disconnect(&mut self) {
        self.cleanup_native(false);
        // v16: تیک‌های سبز Diagnostics باید بلافاصله بعد از دیسکانکت
        // ریست شوند تا برای اتصال بعدی آماده باشند (معادل resetChecks اندروید).
        diagnostics::reset_checks();
        self.latency_probe_at = None;
        self.watchdog_probe_at = None;
        self.watchdog_failures = 0;
        *self.watchdog_slot.lock() = None;
        *self.latency_slot.lock() = None;
        self.connected_at = None;
        self.endpoint = None;
        self.latency_ms = None;
        self.effective_protocol = None;
        self.deadline = None;
        self.reconnect_at = None;
        self.verify_slot = None;
        self.chain_slot = None;
        self.prep_slot = None;
        self.plan.clear();
        self.plan_index = 0;
        self.set_state(ConnectionState::Disconnected, "");
    }

    /// ترتیب ۱.۲.۲: اول پروکسی سیستمی (تا مرورگر به پل مُرده نچسبد)، بعد
    /// اشتراک، بعد تونل، بعد موتور — بدون فریز.
    fn cleanup_native(&mut self, preserve_kill_switch: bool) {
        if self.sysproxy_on {
            sysproxy::disable();
            self.sysproxy_on = false;
        }
        // گارد بعد از پروکسی آزاد می‌شود: تا آخرین لحظه‌ای که مرورگر ممکن است
        // به پل وصل باشد، مسیر UDP هم بسته می‌ماند.
        if preserve_kill_switch {
            if let Some(g) = self.guard.as_mut() {
                g.release_for_reconnect();
            }
        } else if let Some(mut g) = self.guard.take() {
            g.release();
        }
        self.webrtc_leak = None;
        self.share.stop();
        // استیج ۲ بعد از پل و پیش از موتور می‌رود: همان ترتیب معکوسِ بالا آمدن.
        // اگر پیش از پل برود، پل برای چند صد میلی‌ثانیه به یک پروکسی مرده وصل
        // می‌ماند و مرورگر خطای واقعی می‌بیند.
        self.psiphon.stop();
        // خروجی به موتور برمی‌گردد، وگرنه پلهٔ بعدی نردبان (یا نشست بعدی) به
        // پورت استیج ۲ که دیگر وجود ندارد وصل می‌ماند.
        engine::reset_exit_socks_port();
        self.chain_exit_port = None;
        self.chain_slot = None;
        if let Some(mut t) = self.tunnel.take() {
            t.close();
        }
        self.engine.stop();
    }

    /// شکست یک پله → پلهٔ بعدی نردبان؛ تمام‌شدن نردبان → Failed با پیام روشن.
    fn advance_or_fail(&mut self, why: &str) {
        DiagnosticsLog::w(TAG, &format!("{why} — tearing down this attempt."));
        // فقط موتور/مسیر داده را جمع می‌کنیم، وضعیت UI همچنان busy می‌ماند.
        self.cleanup_native(true);
        self.verify_slot = None;
        diagnostics::reset_checks();
        self.plan_index += 1;
        if self.plan_index < self.plan.len() {
            // The next rung also has to wait for the local port, so it goes back
            // through the off-lock prep instead of blocking the tick for 3s.
            self.set_state(ConnectionState::StartingEngine, "Starting engine…");
            self.begin_prep(Prep::Candidate);
        } else if self.profile.protocol == Protocol::Smart {
            self.fail("Smart Auto tried every strategy and none passed the self-test on this network.");
        } else {
            self.fail(
                "This protocol could not establish a working tunnel on this network, even with anti-DPI hardening. Try Smart Auto or another protocol.",
            );
        }
    }

    fn apply_pending_security_refresh(&mut self) {
        if !self.security_refresh_pending { return; }
        self.security_refresh_pending = false;
        if self.profile.leak_guard || self.profile.kill_switch || self.profile.ipv6_protection {
            if let Some(mut old_guard) = self.guard.take() { old_guard.disarm_without_cleanup(); }
            self.guard = Some(LeakGuard::engage(&self.profile));
        } else if let Some(mut guard) = self.guard.take() {
            guard.release();
        }
    }

    /// هر ۲۰۰ms از main.rs صدا زده می‌شود — معادل حلقهٔ نظارت اندروید.
    pub fn tick(&mut self) {
        // The last tap first. Both branches are slow, and both are why this runs
        // here instead of inside the IPC command. See [`Intent`].
        if let Some(intent) = self.pending_intent.take() {
            match intent {
                Intent::Connect => {
                    if let Err(e) = self.connect() {
                        let msg = e.to_string();
                        self.fail(&msg);
                    }
                }
                Intent::Disconnect => self.disconnect(),
            }
            return;
        }

        self.apply_pending_security_refresh();

        match self.state {
            // Waiting for the off-lock prep thread (port release + fingerprint).
            ConnectionState::StartingEngine => {
                let outcome = self.prep_slot.as_ref().and_then(|s| s.lock().take());
                if let Some(outcome) = outcome {
                    self.prep_slot = None;
                    let result = match self.prep_kind {
                        Prep::Plan => self.launch_plan(outcome.fingerprint),
                        Prep::Candidate => self.launch_candidate(),
                    };
                    if let Err(e) = result {
                        let msg = e.to_string();
                        self.fail(&msg);
                    }
                }
            }
            ConnectionState::Connecting => {
                if !self.engine.is_alive() {
                    self.advance_or_fail("Engine exited before it opened the SOCKS5 port");
                    return;
                }
                // یک زنجیرهٔ در حال بالا آمدن، فاز خودش را دارد: استیج ۱ آماده
                // است و استیج ۲ در ترد پس‌زمینه برقرار می‌شود.
                if self.chain_slot.is_some() {
                    self.poll_chain();
                    return;
                }
                if probe::socks_ready(engine::LOCAL_SOCKS_PORT) {
                    if self.profile.is_chained() {
                        self.begin_chain();
                    } else {
                        DiagnosticsLog::i(TAG, "SOCKS5 port is up — bringing up the data path.");
                        self.bring_up_data_path();
                        self.begin_verification();
                    }
                } else if self.past_deadline() {
                    self.advance_or_fail("Engine still scanning — the SOCKS5 port never opened in time");
                }
            }
            ConnectionState::Verifying => {
                let outcome = self.verify_slot.as_ref().and_then(|s| s.lock().take());
                if let Some(out) = outcome {
                    self.verify_slot = None;
                    if out.ok {
                        if let Some(exit) = &out.exit {
                            self.endpoint = Some(match &exit.country_code {
                                Some(cc) => format!("{} · {cc}", exit.ip),
                                None => exit.ip.clone(),
                            });
                            // IP خروجی از خودآزما مستقیماً به نشان IP می‌رود —
                            // معادل offerTunnelIpInfo در Diagnostics.kt.
                            let mut g = self.ip_slot.lock();
                            g.session += 1;
                            g.info = Some(IpEndpoint {
                                ip: exit.ip.clone(),
                                country_code: exit.country_code.clone(),
                                via_tunnel: true,
                            });
                            g.loading = false;
                        }
                        // v1.2.0: نتیجهٔ سنجش نشتی مستقیم به نشانِ صفحهٔ اصلی می‌رود.
                        self.webrtc_leak = out.leak.as_ref().map(|l| l.leaking);
                        if self.webrtc_leak == Some(true) {
                            DiagnosticsLog::w(
                                TAG,
                                "Tunnel is up but WebRTC still reached a STUN server directly. Restart the browser so the WebRTC policy applies, or run Aether as administrator for the firewall layer.",
                            );
                        }
                        // `out.latency_ms` is how long the self-test's own HTTP
                        // fetch took, on a brand new dial, in the busiest second
                        // of the session. Through a chained pipeline that reads
                        // as 1600-3000 ms on a path that is actually fine, and it
                        // then sat frozen on screen — the reported bug. The badge
                        // now waits for the first real warm round trip instead of
                        // opening with a number nobody can reproduce.
                        if let Some(setup) = out.latency_ms {
                            DiagnosticsLog::i(
                                TAG,
                                &format!(
                                    "Self-test egress fetch took {setup} ms (dial + TLS + HTTP during \
                                     connect). Not shown as latency; the badge uses a warm round trip."
                                ),
                            );
                        }
                        self.latency_ms = None;
                        // A fresh pipeline needs a fresh probe session, and the
                        // first measurement should land immediately, not in 10s.
                        ping::reset();
                        self.latency_probe_at = None;
                        *self.latency_slot.lock() = None;
                        self.watchdog_probe_at = Some(Instant::now() + Duration::from_secs(WATCHDOG_INTERVAL_SECS));
                        self.watchdog_failures = 0;
                        self.connected_at = Some(Instant::now());
                        self.attempts = 0;
                        self.set_state(ConnectionState::Connected, "");
                        DiagnosticsLog::i(TAG, "All checks passed — tunnel is ready.");
                        if out.exit.is_none() {
                            spawn_ip_lookup(self.ip_slot.clone(), true);
                        }
                    } else if out.leak.as_ref().map(|l| l.leaking).unwrap_or(false) {
                        // Fail closed. A tunnel that exposes the real IP is not
                        // a successful connection, even when TCP/DNS passed.
                        self.fail(
                            "Connection refused: WebRTC can still reach the real IP over direct UDP. Browser and system protection could not be verified.",
                        );
                    } else {
                        self.advance_or_fail("Tunnel started, but the end-to-end self-test failed");
                    }
                } else if !self.engine.is_alive() {
                    self.advance_or_fail("The engine stopped during verification");
                }
            }
            ConnectionState::Connected => {
                // v1.2.0 watchdog: every 30s run three end-to-end probes in a
                // worker thread. Three consecutive failed rounds are required
                // before restarting, so short network jitter is tolerated.
                let watchdog_result = { self.watchdog_slot.lock().take() };
                if let Some(result) = watchdog_result {
                    if result {
                        self.watchdog_failures = 0;
                        DiagnosticsLog::i(TAG, "Watchdog probe passed (at least 2 of 3 targets reachable through SOCKS5).");
                    } else {
                        self.watchdog_failures = self.watchdog_failures.saturating_add(1);
                        DiagnosticsLog::w(TAG, &format!("Watchdog probe failed ({}/{})", self.watchdog_failures, WATCHDOG_FAILURE_THRESHOLD));
                        if self.watchdog_failures >= WATCHDOG_FAILURE_THRESHOLD {
                            DiagnosticsLog::e(TAG, "Watchdog confirmed a persistent upstream failure — restarting the engine.");
                            self.cleanup_native(true);
                            self.watchdog_failures = 0;
                            self.connected_at = None;
                            self.reconnect_at = Some(Instant::now() + Duration::from_secs(2));
                            self.set_state(ConnectionState::Reconnecting, "Watchdog reconnect…");
                            return;
                        }
                    }
                }
                let watchdog_due = self.watchdog_probe_at
                    .map(|t| Instant::now() >= t)
                    .unwrap_or(true);
                let watchdog_busy = { self.watchdog_slot.lock().is_some() };
                if watchdog_due && !watchdog_busy {
                    self.watchdog_probe_at = Some(Instant::now() + Duration::from_secs(WATCHDOG_INTERVAL_SECS));
                    let slot = self.watchdog_slot.clone();
                    std::thread::Builder::new()
                        .name("aether-watchdog".into())
                        .spawn(move || {
                            let ok = probe::watchdog_probe();
                            *slot.lock() = Some(ok);
                        })
                        .ok();
                }

                // v16: پینگ نمایشی قبلاً فقط یک‌بار هنگام خودآزمای اتصال اندازه
                // گرفته می‌شد (شامل زمان دریافت HTTP در شلوغی لحظهٔ اتصال)
                // و دیگر به‌روز نمی‌شد — برای همین عددی مثل ۸۰۰۰ms می‌ماند.
                // حالا هر ۱۵ ثانیه یک اتصال TCP سبک از داخل تونل زمان‌گیری
                // می‌شود تا پینگ واقعی و زنده نمایش داده شود (بدون فریز UI).
                if let Some(ms) = self.latency_slot.lock().take() {
                    self.latency_ms = Some(ms);
                }
                let latency_due = self
                    .latency_probe_at
                    .map(|t| Instant::now() >= t)
                    .unwrap_or(true);
                if latency_due {
                    self.latency_probe_at =
                        Some(Instant::now() + Duration::from_secs(LATENCY_INTERVAL_SECS));
                    let slot = self.latency_slot.clone();
                    std::thread::Builder::new()
                        .name("aether-latency".into())
                        .spawn(move || {
                            // One keep-alive round trip on a warm session — no
                            // dial, no SSH channel open. See [`crate::ping`].
                            if let Some(ms) = ping::measure() {
                                *slot.lock() = Some(ms);
                            }
                        })
                        .ok();
                }
                // سوپروایز **هر دو** هاپ. یک نشست زنجیره‌ای فقط به‌قدر ضعیف‌ترین
                // استیجش زنده است، و استیج ۱ مرده یعنی استیج ۲ پروکسی‌ای در دست
                // دارد که نمی‌تواند dial کند — «متصل» با هیچ چیزی در حرکت.
                //
                // `is_alive` استیج ۲ در طول یک چرخشِ عمدی عمداً true می‌ماند
                // (نگاه کنید به psiphon.rs)، وگرنه واچ‌داگ همان نشستی را
                // می‌کشت که قرار بود نجاتش بدهد.
                if self.profile.is_chained() && !self.psiphon.is_alive() {
                    DiagnosticsLog::e(TAG, "The Psiphon stage died while connected — rebuilding the session.");
                    self.cleanup_native(true);
                    self.connected_at = None;
                    self.reconnect_at = Some(Instant::now() + Duration::from_secs(2));
                    self.set_state(ConnectionState::Reconnecting, "Rebuilding the chain…");
                    return;
                }
                if !self.engine.is_alive() {
                    // معادل superviseEngine: بک‌آف پلکانی ۲/۵/۱۰ ثانیه، حداکثر ۳ تلاش.
                    let max_retries = self.profile.reconnect_attempts.max(DEFAULT_MAX_RETRIES);
                    if self.attempts >= max_retries {
                        self.fail("The engine keeps dying — giving up after repeated restarts.");
                        return;
                    }
                    let backoff = BACKOFF_MS[(self.attempts as usize).min(BACKOFF_MS.len() - 1)];
                    self.attempts += 1;
                    self.connected_at = None;
                    self.reconnect_at = Some(Instant::now() + Duration::from_millis(backoff));
                    DiagnosticsLog::w(
                        TAG,
                        &format!("Engine died while connected — restarting in {}s.", backoff / 1000),
                    );
                    let detail = format!("Attempt {} of {}", self.attempts, max_retries);
                    self.set_state(ConnectionState::Reconnecting, &detail);
                }
            }
            ConnectionState::Reconnecting => {
                if let Some(at) = self.reconnect_at {
                    if Instant::now() >= at {
                        self.reconnect_at = None;
                        self.cleanup_native(true);
                        // همان پلهٔ برنده دوباره اجرا می‌شود — معادل restart در
                        // superviseEngine. The fingerprint and the port wait go
                        // to the prep thread, so a reconnect no longer freezes
                        // the UI for several seconds either.
                        let kind = if self.plan.is_empty() { Prep::Plan } else { Prep::Candidate };
                        self.set_state(ConnectionState::StartingEngine, "Reconnecting…");
                        self.begin_prep(kind);
                    }
                }
            }
            _ => {}
        }
    }

    fn past_deadline(&self) -> bool {
        self.deadline.map(|d| Instant::now() > d).unwrap_or(false)
    }

    fn fail(&mut self, why: &str) {
        DiagnosticsLog::e(TAG, why);
        self.cleanup_native(true);
        self.error = Some(why.to_string());
        self.connected_at = None;
        self.deadline = None;
        self.reconnect_at = None;
        self.verify_slot = None;
        self.prep_slot = None;
        self.pending_intent = None;
        self.latency_ms = None;
        ping::reset();
        self.set_state(ConnectionState::Failed, "Connection failed");
    }

    fn set_state(&mut self, state: ConnectionState, detail: &str) {
        let prev = self.state;
        self.state = state;
        self.detail = detail.to_string();
        DiagnosticsLog::i(TAG, &format!("{state:?} {detail}"));
        if prev != state {
            self.on_phase_change(state);
        }
    }

    /// معادل LaunchedEffect فازهای IP در MainActivity.kt:
    ///   connected → IP سرور از دل تونل — idle/failed → IP واقعی کاربر — busy → خالی.
    fn on_phase_change(&mut self, state: ConnectionState) {
        match state {
            ConnectionState::Connected => { /* خودآزما قبلاً IP را تحویل داده است */ }
            ConnectionState::Disconnected | ConnectionState::Failed => {
                spawn_ip_lookup(self.ip_slot.clone(), false);
            }
            _ => {
                let mut g = self.ip_slot.lock();
                g.session += 1;
                g.info = None;
                g.loading = false;
            }
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

impl Drop for AetherController {
    fn drop(&mut self) {
        // خروج برنامه هرگز نباید پروکسی سیستمی را فعال رها کند.
        self.cleanup_native(false);
    }
}

/// جست‌وجوی IP در ترد پس‌زمینه — همان تعداد تلاش/تأخیرهای NetProbe اندروید:
/// مستقیم ۶×۲۰۰۰ms، از دل تونل ۱۲×۱۰۰۰ms.
fn spawn_ip_lookup(slot: Arc<Mutex<IpSlot>>, via_tunnel: bool) {
    let session = {
        let mut g = slot.lock();
        g.session += 1;
        g.loading = true;
        if !via_tunnel {
            g.info = None;
        }
        g.session
    };
    std::thread::Builder::new()
        .name("aether-ipinfo".into())
        .spawn(move || {
            let result = if via_tunnel {
                probe::fetch_ip_via_socks_retry(12, 1_000, 6_000)
            } else {
                probe::fetch_ip_direct_retry(6, 2_000, 6_000)
            };
            let mut g = slot.lock();
            if g.session != session {
                return; // نتیجهٔ کهنه — فاز عوض شده است.
            }
            g.info = result.map(|i| IpEndpoint {
                ip: i.ip,
                country_code: i.country_code,
                via_tunnel,
            });
            g.loading = false;
        })
        .ok();
}
