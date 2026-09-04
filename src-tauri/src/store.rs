//! پورت از `data/ProfileStore.kt`.
//!
//! در اندروید DataStore بود؛ در ویندوز یک فایل JSON در %LOCALAPPDATA%\Aether.
//! رفتار یکسان است: خواندن هرگز خطا نمی‌دهد — فایل خراب یعنی پیش‌فرض‌ها.

use crate::log::DiagnosticsLog;
use crate::profile::{ConnectionProfile, SETTINGS_REV};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct ProfileStore {
    path: PathBuf,
}

impl ProfileStore {
    pub fn new(data_dir: &Path) -> Self {
        Self { path: data_dir.join("profile.json") }
    }

    pub fn load(&self) -> ConnectionProfile {
        let mut profile = match std::fs::read_to_string(&self.path) {
            Ok(raw) => match serde_json::from_str::<ConnectionProfile>(&raw) {
                Ok(p) => p,
                Err(e) => {
                    DiagnosticsLog::w("store", &format!("Profile unreadable ({e}); using defaults."));
                    ConnectionProfile::default()
                }
            },
            Err(_) => ConnectionProfile::default(),
        };
        if migrate(&mut profile) {
            let _ = self.save(&profile);
        }
        profile
    }

    /// نوشتن اتمیک: اول فایل موقت، بعد rename. قطع برق وسط ذخیره
    /// نباید تنظیمات کاربر را نابود کند.
    pub fn save(&self, profile: &ConnectionProfile) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(profile)?)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

/// Brings a profile written by an older build up to the current defaults.
///
/// Returns `true` when something changed, so the caller can persist it once.
///
/// ## rev 2 - the MASQUE carrier toggle
///
/// `MASQUE over HTTP/2` is a fallback for networks that block HTTP/3, and it is
/// the one data plane whose flow control is a single HTTP/2 window shared by the
/// whole machine. Two things could leave it stuck on: the user ticking it once
/// while chasing a connection problem, and - until 1.2.3-p2 - Smart Auto's
/// hardened pass, which used to turn it on unconditionally and then wrote the
/// profile back. Either way the setting long outlived the network condition that
/// justified it, and every later session paid for it in download speed.
///
/// Turning it back off is safe: the pre-connect probe now measures UDP for real,
/// so a network that genuinely needs the TCP carrier gets it anyway, on evidence
/// rather than on a stale checkbox.
fn migrate(profile: &mut ConnectionProfile) -> bool {
    if profile.settings_rev >= SETTINGS_REV {
        return false;
    }

    if profile.masque_http2 {
        profile.masque_http2 = false;
        DiagnosticsLog::i(
            "store",
            "Settings migration: 'MASQUE over HTTP/2' was left on from an earlier session and has been turned off. \
             The HTTP/3 carrier is faster, and the network probe now selects HTTP/2 by itself when UDP is blocked.",
        );
    }

    profile.settings_rev = SETTINGS_REV;
    true
}
