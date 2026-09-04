//! پورت از `core/SmartAuto.kt` + منطق نردبان `AetherVpnService.directPlan/buildPlan`.
//!
//! ریشهٔ باگ قبلی دسکتاپ: فقط «یک» پروتکل انتخاب می‌شد و هیچ نردبان
//! تلاشِ چندمرحله‌ای وجود نداشت؛ در اندروید هر اتصال یک «برنامه» چند
//! کاندیدایی است که یکی‌یکی امتحان می‌شوند تا اولینِ قبول‌شده در خودآزما
//! برنده شود. همان منطق این‌جا پیاده شده:
//!
//!  * پروتکل دستی  ← دو پاس (معادل directPlan): اول همان تنظیمات کاربر
//!    (سقف ۷۵ ثانیه)، بعد پاس ضد-DPI سخت‌شده — پروتکل هرگز عوض نمی‌شود.
//!  * Smart Auto ← نردبان MASQUE → MASQUE سخت‌شده → GOOL → WireGuard
//!    (همان ترتیب ترجیح SmartAuto.kt).

use crate::log::DiagnosticsLog;
use crate::profile::{ConnectionProfile, IpVersion, Noize, Protocol};

const TAG: &str = "auto";

/// سقف پاس اول — همان `FIRST_PASS_MAX_MS` اندروید.
const FIRST_PASS_MAX_MS: u64 = 35_000;

/// ترتیب ترجیح — همان ترتیبی که SmartAuto.kt دارد.
const PREFERENCE: [Protocol; 3] = [Protocol::Masque, Protocol::Gool, Protocol::Wireguard];

/// What the pre-connect probes learned about this network.
///
/// Until 1.2.3-p2 this was a single `hostile: bool` derived from one TCP:80
/// dial, so the planner had no idea whether UDP worked - and the MASQUE carrier
/// choice is entirely a question about UDP. See [`harden`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetFingerprint {
    /// Direct TCP egress looks blocked: lead with the hardened anti-DPI pass.
    pub filtered: bool,
    /// A real UDP DNS round trip completed, so QUIC/HTTP-3 is viable.
    pub udp_ok: bool,
}

impl Default for NetFingerprint {
    fn default() -> Self {
        // Absent evidence, assume UDP works: HTTP/3 is the fast carrier and the
        // ladder still falls through to HTTP/2 if the attempt fails.
        Self { filtered: false, udp_ok: true }
    }
}

/// معادل `AutoCandidate` اندروید — یک استراتژی آمادهٔ اجرا.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub profile: ConnectionProfile,
    pub timeout_ms: u64,
    pub label: String,
}

/// ساخت نسخهٔ سخت‌شدهٔ ضد-DPI — معادل پاس دوم `directPlan` اندروید.
///
/// ## 1.2.3-p2: hardening no longer means "make the download slow"
///
/// This used to set `masque_http2 = true` unconditionally. That one line is how
/// almost every desktop session ended up on the HTTP/2 carrier: MASQUE is the
/// first rung of the Smart Auto ladder, so a single flaky first attempt promoted
/// the hardened pass, the hardened pass demoted the carrier from HTTP/3 to
/// HTTP/2, and the session then stayed on the one data plane with a 64 KB
/// flow-control window - for the rest of its life, on a network where UDP was
/// perfectly healthy.
///
/// The mobile build never did this: `SmartAuto.kt` only passes `h2 = true` in
/// the `UDP_THROTTLED` and `HOSTILE` branches, i.e. only after a real UDP probe
/// has failed. Same rule here now. A user who ticked the toggle themselves is
/// still honoured.
fn harden(p: &ConnectionProfile, fp: NetFingerprint) -> ConnectionProfile {
    let mut h = p.clone();
    if h.noize == Noize::Off {
        h.noize = Noize::Firewall;
    }
    if h.protocol == Protocol::Masque {
        h.masque_http2 = p.masque_http2 || !fp.udp_ok;
        h.fragment = true;
        h.ech = true;
    }
    h
}

/// The profile with the MASQUE carrier forced onto HTTP/2 when the pre-connect
/// probe proved QUIC cannot work on this network.
///
/// ## 1.2.3-p3: the FIRST rung must not ride a carrier already known to be dead
///
/// [`harden`] has done this since p2, but only for the hardened pass. The plain
/// "as configured" rung - the first thing the ladder tries, and the one that owns
/// the first-pass window - kept riding HTTP/3 even after the probe had proved
/// QUIC was filtered. On the network in the field log that guaranteed the first
/// 35 seconds of every connect went to a rung that could not possibly succeed,
/// and because a filtered network leads with the hardened pass, the second rung
/// then burnt another 60 seconds exactly the same way. Three quarters of the
/// connect time the user complained about was spent on carriers the app had
/// already measured as unusable.
///
/// This only ever turns the HTTP/2 carrier ON. A user who ticked the toggle
/// themselves is untouched, and on a healthy-UDP network nothing changes at all.
fn carrier_for(p: &ConnectionProfile, fp: NetFingerprint) -> ConnectionProfile {
    let mut out = p.clone();
    if out.protocol == Protocol::Masque && !fp.udp_ok {
        out.masque_http2 = true;
    }
    out
}

/// Carrier suffix for a label, so the log says which data plane is being tried.
fn carrier(p: &ConnectionProfile) -> &'static str {
    if p.protocol == Protocol::Masque && p.masque_http2 {
        " · h2"
    } else if p.protocol == Protocol::Masque {
        " · h3"
    } else {
        ""
    }
}

/// معادل `directPlan` — پروتکل دستی، دو پاس، بدون تعویض پروتکل.
fn direct_plan(user: &ConnectionProfile, fp: NetFingerprint) -> Vec<Candidate> {
    let full = user.connect_timeout_ms();
    let hostile = fp.filtered;
    // 1.2.3-p3: the plain pass rides the carrier the probe says works, not the
    // one the panel happens to default to. See [`carrier_for`].
    let plain = carrier_for(user, fp);
    let hardened = harden(&plain, fp);
    let name = format!("{:?}", user.protocol).to_uppercase();
    if hardened == plain {
        return vec![Candidate {
            timeout_ms: full,
            label: format!("{name} · as configured{}", carrier(&plain)),
            profile: plain,
        }];
    }
    let as_configured = Candidate {
        timeout_ms: full.min(FIRST_PASS_MAX_MS),
        label: format!("{name} · as configured{}", carrier(&plain)),
        profile: plain,
    };
    let anti_dpi = Candidate {
        label: format!("{name} · hardened anti-DPI{}", carrier(&hardened)),
        profile: hardened,
        timeout_ms: full,
    };
    if hostile {
        // Filtered network: lead with the hardened pass so the plain pass
        // cannot burn the first-pass window (root cause of the slow connects)
        // or win with a data path that DPI then strangles mid-session.
        vec![anti_dpi, as_configured]
    } else {
        vec![as_configured, anti_dpi]
    }
}

/// نردبان Smart Auto — معادل `SmartAuto.buildPlan`.
fn auto_plan(user: &ConnectionProfile, fp: NetFingerprint) -> Vec<Candidate> {
    let full = user.connect_timeout_ms();
    let hostile = fp.filtered;
    let mut plan = Vec::new();

    // فقط IPv6: WireGuard پایدارتر است — همان قاعدهٔ اندروید.
    let order: Vec<Protocol> = if user.ip_version == IpVersion::V6 {
        vec![Protocol::Wireguard, Protocol::Masque, Protocol::Gool]
    } else {
        PREFERENCE.to_vec()
    };

    for (i, proto) in order.iter().enumerate() {
        let mut base = user.clone();
        base.protocol = *proto;
        // Same rule as `direct_plan`: never spawn a rung on a carrier the
        // pre-connect probe already proved cannot carry traffic here.
        let base = carrier_for(&base, fp);
        let name = format!("{proto:?}").to_uppercase();
        if i == 0 {
            let as_configured = Candidate {
                label: format!("{name} · as configured{}", carrier(&base)),
                profile: base.clone(),
                timeout_ms: full.min(FIRST_PASS_MAX_MS),
            };
            let hardened = harden(&base, fp);
            let anti_dpi = Candidate {
                label: format!("{name} · hardened anti-DPI{}", carrier(&hardened)),
                profile: hardened,
                timeout_ms: full.min(120_000),
            };
            if hostile {
                plan.push(anti_dpi);
                plan.push(as_configured);
            } else {
                plan.push(as_configured);
                plan.push(anti_dpi);
            }
            // 1.2.3-p2: on a healthy-UDP network the two MASQUE rungs above both
            // ride HTTP/3, so the ladder would never reach the TCP carrier at
            // all if QUIC turned out to be blocked mid-scan rather than at probe
            // time. One explicit HTTP/2 rung keeps that escape hatch, at the
            // END, where it belongs - instead of being the second thing tried.
            if *proto == Protocol::Masque && fp.udp_ok && !user.masque_http2 {
                let mut h2 = harden(&base, fp);
                h2.masque_http2 = true;
                plan.push(Candidate {
                    label: format!("{name} · hardened anti-DPI{}", carrier(&h2)),
                    profile: h2,
                    timeout_ms: full.min(120_000),
                });
            }
        } else {
            let hardened = harden(&base, fp);
            plan.push(Candidate {
                label: format!("{name} · hardened anti-DPI{}", carrier(&hardened)),
                profile: hardened,
                timeout_ms: if i + 1 == order.len() { full } else { full.min(120_000) },
            });
        }
    }
    plan
}

/// نقطهٔ ورود: برنامهٔ کامل اتصال برای پروفایل کاربر.
pub fn build_plan(user: &ConnectionProfile, fp: NetFingerprint) -> Vec<Candidate> {
    if fp.filtered {
        DiagnosticsLog::w(TAG, "Network fingerprint: this network looks filtered - anti-DPI attempts run first.");
    }
    // Say it out loud: this single bit decides HTTP/3 (QUIC) versus HTTP/2 (TCP)
    // for the MASQUE carrier, and the two have very different throughput.
    DiagnosticsLog::i(
        TAG,
        &format!(
            "Network fingerprint: udp={} → MASQUE carrier prefers {}",
            if fp.udp_ok { "ok" } else { "blocked/throttled" },
            if fp.udp_ok { "HTTP/3 (QUIC)" } else { "HTTP/2 (TCP)" },
        ),
    );
    let plan = if user.protocol == Protocol::Smart {
        DiagnosticsLog::i(TAG, "Smart Auto: building the strategy ladder…");
        auto_plan(user, fp)
    } else {
        direct_plan(user, fp)
    };
    let summary: Vec<String> = plan.iter().map(|c| c.label.clone()).collect();
    DiagnosticsLog::i(TAG, &format!("Plan ready ({} attempt(s)): {}", plan.len(), summary.join(" → ")));
    plan
}

/// معادل `SmartAuto.choose()` — برای سازگاری با کد/تست‌های قبلی حفظ شده.
pub fn pick(profile: &ConnectionProfile) -> Protocol {
    if profile.protocol != Protocol::Smart {
        return profile.protocol;
    }
    if profile.has_manual_peer() {
        return Protocol::Masque;
    }
    if profile.ip_version == IpVersion::V6 {
        return Protocol::Wireguard;
    }
    PREFERENCE[0]
}

/// ترتیب تلاش مجدد پس از شکست — معادل `nextCandidate()`.
pub fn next_after(failed: Protocol) -> Option<Protocol> {
    let idx = PREFERENCE.iter().position(|p| *p == failed)?;
    PREFERENCE.get(idx + 1).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_never_reaches_the_engine() {
        let p = ConnectionProfile::default();
        assert_ne!(pick(&p), Protocol::Smart);
        for c in build_plan(&p, NetFingerprint::default()) {
            assert_ne!(c.profile.protocol, Protocol::Smart);
        }
    }

    #[test]
    fn explicit_protocol_is_respected() {
        let p = ConnectionProfile { protocol: Protocol::Wireguard, ..Default::default() };
        assert_eq!(pick(&p), Protocol::Wireguard);
        for c in build_plan(&p, NetFingerprint::default()) {
            assert_eq!(c.profile.protocol, Protocol::Wireguard);
        }
    }

    #[test]
    fn fallback_order_matches_android() {
        assert_eq!(next_after(Protocol::Masque), Some(Protocol::Gool));
        assert_eq!(next_after(Protocol::Gool), Some(Protocol::Wireguard));
        assert_eq!(next_after(Protocol::Wireguard), None);
    }

    /// 1.2.3-p3 regression guard. On a network where the QUIC probe failed,
    /// EVERY MASQUE rung has to be HTTP/2 - including the plain first pass,
    /// which is the one that owns the first-pass window.
    #[test]
    fn no_masque_rung_rides_quic_when_quic_is_dead() {
        let p = ConnectionProfile::default();
        let fp = NetFingerprint { filtered: false, udp_ok: false };
        let plan = build_plan(&p, fp);
        let masque: Vec<&Candidate> =
            plan.iter().filter(|c| c.profile.protocol == Protocol::Masque).collect();
        assert!(!masque.is_empty());
        for c in &masque {
            assert!(c.profile.masque_http2, "rung `{}` still rides QUIC", c.label);
            assert!(c.label.ends_with(" · h2"), "label lies about the carrier: {}", c.label);
        }
    }

    /// The mirror image: a healthy network must keep leading with HTTP/3, which
    /// is the fast carrier. The p2 behaviour is preserved exactly.
    #[test]
    fn healthy_udp_still_leads_with_quic() {
        let p = ConnectionProfile::default();
        let plan = build_plan(&p, NetFingerprint::default());
        assert!(!plan[0].profile.masque_http2);
        assert!(plan[0].label.ends_with(" · h3"));
    }

    /// A user who chose HTTP/2 in the panel keeps it on a healthy network too:
    /// `carrier_for` only ever turns the carrier on.
    #[test]
    fn an_explicit_http2_choice_is_never_undone() {
        let p = ConnectionProfile { masque_http2: true, ..Default::default() };
        for c in build_plan(&p, NetFingerprint::default()) {
            if c.profile.protocol == Protocol::Masque {
                assert!(c.profile.masque_http2);
            }
        }
    }

    #[test]
    fn direct_plan_has_a_hardened_second_pass() {
        let p = ConnectionProfile { protocol: Protocol::Gool, ..Default::default() };
        let plan = build_plan(&p, NetFingerprint::default());
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].profile.noize, Noize::Off);
        assert_eq!(plan[1].profile.noize, Noize::Firewall);
        assert_eq!(plan[0].profile.protocol, plan[1].profile.protocol);
    }
}
