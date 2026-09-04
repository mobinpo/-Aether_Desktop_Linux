use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy)]
pub struct Tuning {
    pub tier: Tier,
    pub cpus: usize,
    pub mem_mb: Option<u64>,
    pub scan_concurrency_cap: usize,
    /// `SO_RCVBUF` for every datagram socket in the data plane.
    ///
    /// Large on purpose, and it must stay large. Until 1.2.3-p1 only the
    /// MASQUE/QUIC path ever applied a figure from this profile; every WireGuard
    /// socket - which is to say the whole WARP and WARP*2 data plane, and the
    /// first hop of the chained backend - ran on the Windows default of 64 KB.
    /// At a 1280-byte tunnel MTU that is about fifty datagrams: the first time
    /// the reader was a few hundred microseconds late, the kernel started
    /// discarding inbound datagrams, TCP read the loss as congestion and
    /// collapsed its window, and the user read that as "the download is slow".
    /// Receive buffers cost latency to nobody.
    pub udp_socket_rcv_buf: usize,
    /// `SO_SNDBUF` for every datagram socket in the data plane.
    ///
    /// Deliberately much smaller than the receive side, and for the opposite
    /// reason: this is the LAST queue between the congestion controller and the
    /// NIC, and `send()` on a datagram socket is only ever backpressure once
    /// this buffer is full. Size it to a latency budget, never to RAM - a send
    /// buffer big enough never to fill silently disables every throttle above
    /// it and shows the congestion controller a link with infinite capacity and
    /// zero loss.
    pub udp_socket_snd_buf: usize,
    /// Per-flow TCP SEND buffer: app->network data waiting for CUBIC to clock it
    /// out. Feeds the congestion controller, so it may be generous; it is still
    /// bounded, because in chained mode there is effectively one flow and this
    /// queue is paid by every other flow on the machine.
    pub netstack_tcp_tx_buf: usize,
    /// Per-flow TCP RECEIVE buffer, i.e. **the advertised receive window**.
    ///
    /// ## 1.2.3-p1: this is the hard ceiling on download throughput
    ///
    /// smoltcp derives the window it advertises from the free space in THIS
    /// buffer, so its size caps how many bytes a remote server is allowed to
    /// have in flight towards this PC. Throughput per flow can never exceed
    /// window / RTT, whatever the line can do.
    ///
    /// It used to be the same number as the send buffer, and on a 4-core PC that
    /// resolved to the Medium tier's 256 KB. Over a single-hop path at ~150 ms
    /// that is ~1.7 MB/s; through the `Aether -> Psiphon` chain, where the RTT
    /// is paid twice, it is half of that - on a line that can do ten times more.
    /// It is sized to a desktop bandwidth-delay product now, and kept separate
    /// from the send side on purpose: the two have opposite requirements and
    /// sharing one number is what hid the wrong one.
    pub netstack_tcp_rx_buf: usize,
    pub netstack_udp_buf: usize,
    pub channel_capacity: usize,

    // =====================================================================
    //  1.2.3-p2 - the HTTP/2 MASQUE carrier
    // =====================================================================
    /// SETTINGS_INITIAL_WINDOW_SIZE we advertise on the HTTP/2 carrier, i.e.
    /// **the per-stream receive window**.
    ///
    /// ## Why this is the real download ceiling on Windows
    ///
    /// The `Aether -> Cloudflare` MASQUE tunnel over HTTP/2 is **one single
    /// CONNECT-IP stream**. Every flow on the machine - every browser tab, every
    /// download, and in chained mode the whole Psiphon hop as well - is
    /// multiplexed into that one stream. So this window is not a per-flow
    /// figure like `netstack_tcp_rx_buf`: it is the ceiling for the entire
    /// machine.
    ///
    /// The `h2` crate defaults it to 65535 bytes, and until 1.2.3-p2 the carrier
    /// was built with `h2::client::handshake()`, i.e. with the library defaults
    /// untouched. Throughput can never exceed window / RTT:
    ///
    ///   65535 B / 130 ms  ~= 0.5 MB/s  (~4 Mbit/s)   Aether only
    ///   65535 B / 500 ms  ~= 0.13 MB/s (~1 Mbit/s)   Aether -> Psiphon
    ///
    /// which is the reported speed almost exactly, on a line that does far more,
    /// and it is why the 1.2.3-p1 work changed nothing: every fix in p1 landed
    /// on the WireGuard/QUIC data plane, which this carrier never executes.
    ///
    /// ## 1.2.3-p3: p2 fixed the ceiling and then overshot into bufferbloat
    ///
    /// p2 raised this from 64 KB to 8 MB, which removed the throughput ceiling
    /// and replaced it with a queueing one. A receive window is a licence for the
    /// far side to keep that many bytes IN FLIGHT towards this PC, so any window
    /// larger than the path's bandwidth-delay product buys no bandwidth at all -
    /// it only converts into standing queue at the Cloudflare edge, and standing
    /// queue is latency. At the throughput the field log actually measured
    /// (~760 KB/s) an 8 MB window is over TEN SECONDS of buffer:
    ///
    /// ```text
    ///   Latency spike: first round trip 4910 ms, session floor 129 ms
    ///   Latency spike: first round trip 5618 ms, session floor 129 ms
    ///   [h2] ... netstack waits=0 drops=0
    /// ```
    ///
    /// `waits=0 drops=0` is the proof: none of our own queues were full, because
    /// the queue was not ours - it was sitting at the edge, inside credit we had
    /// handed out. And an inflated RTT costs throughput directly, because a flow
    /// can never beat window / RTT: 1 MB of smoltcp receive buffer over the 2 s
    /// effective RTT that queue produced is ~500 KB/s, which is the speed that
    /// was reported.
    ///
    /// So it is sized to a real bandwidth-delay product now: 2 MB over the 130 ms
    /// floor in the log is ~123 Mbit/s of headroom for the whole machine, which
    /// is more than the line, while capping the queue at a quarter of what p2
    /// allowed. Fast enough to never be the limit, small enough never to become
    /// one.
    pub h2_stream_window: u32,
    /// Connection-level receive window on the HTTP/2 carrier.
    ///
    /// HTTP/2 flow control is enforced at BOTH levels and the effective limit is
    /// the smaller of the two, so raising only the stream window buys nothing -
    /// the `h2` default here is also 65535. Kept above the stream window so the
    /// stream is what governs, never the connection.
    pub h2_conn_window: u32,
    /// SETTINGS_MAX_FRAME_SIZE we advertise, i.e. the largest DATA frame the
    /// edge may send us. The default of 16 KB means a 1 Gbit burst arrives as
    /// tens of thousands of frames, each one a parse and a wakeup.
    pub h2_frame_size: u32,
    /// Outbound buffer the carrier may hold before `send_data` blocks.
    ///
    /// 1.2.3-p3: this is a latency budget in front of the carrier, not storage,
    /// so it follows the same rule as `udp_socket_snd_buf` - size it to a delay,
    /// never to RAM. It was 2 MB, which at the measured uplink is minutes of
    /// queue and is what the "waiting behind data already queued in the tunnel's
    /// send buffer" spikes were describing. 512 KB over a 130 ms path is still
    /// ~4 MB/s of uplink headroom, roughly a hundred times the ~21 KB/s of ACKs a
    /// saturated download actually generates.
    pub h2_send_buffer: usize,
    /// How many tunnelled IP packets may be coalesced into one HTTP/2 DATA
    /// frame. The WireGuard path got batching in p1; the HTTP/2 carrier did not,
    /// so every single packet - and on a download most of them are bare ACKs -
    /// became its own DATA frame, its own TLS record and, because the carrier
    /// sets `TCP_NODELAY`, its own TCP segment.
    pub h2_batch_packets: usize,
}

static TUNING: OnceLock<Tuning> = OnceLock::new();

fn detected_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

#[cfg(target_os = "linux")]
fn total_mem_mb() -> Option<u64> {
    let data = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in data.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
            return Some(kb / 1024);
        }
    }
    None
}

#[cfg(target_os = "android")]
fn total_mem_mb() -> Option<u64> {
    let data = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in data.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
            return Some(kb / 1024);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn total_mem_mb() -> Option<u64> {
    let mut size: u64 = 0;
    let mut len = std::mem::size_of::<u64>();
    let name = b"hw.memsize\0";
    let ret = unsafe {
        libc::sysctlbyname(
            name.as_ptr() as *const libc::c_char,
            &mut size as *mut u64 as *mut libc::c_void,
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 {
        Some(size / 1024 / 1024)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn total_mem_mb() -> Option<u64> {
    #[repr(C)]
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalMemoryStatusEx(buf: *mut MemoryStatusEx) -> i32;
    }

    let mut status = MemoryStatusEx {
        length: std::mem::size_of::<MemoryStatusEx>() as u32,
        memory_load: 0,
        total_phys: 0,
        avail_phys: 0,
        total_page_file: 0,
        avail_page_file: 0,
        total_virtual: 0,
        avail_virtual: 0,
        avail_extended_virtual: 0,
    };

    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    if ok != 0 {
        Some(status.total_phys / 1024 / 1024)
    } else {
        None
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "windows"
)))]
fn total_mem_mb() -> Option<u64> {
    None
}

fn detect_tier(cpus: usize, mem_mb: Option<u64>) -> Tier {
    if let Ok(v) = std::env::var("AETHER_PERF_PROFILE") {
        match v.trim().to_lowercase().as_str() {
            "low" => return Tier::Low,
            "medium" | "mid" => return Tier::Medium,
            "high" => return Tier::High,
            _ => {}
        }
    }

    let mem_low = mem_mb.map(|m| m <= 384).unwrap_or(false);
    let mem_medium = mem_mb.map(|m| m <= 1536).unwrap_or(false);

    if cpus <= 2 || mem_low {
        Tier::Low
    } else if cpus <= 4 || mem_medium {
        Tier::Medium
    } else {
        Tier::High
    }
}

fn build_tuning() -> Tuning {
    let cpus = detected_cpus();
    let mem_mb = total_mem_mb();
    let tier = detect_tier(cpus, mem_mb);

    // 1.2.3-p1: the two figures that used to be one each become two, because
    // each pair has opposite requirements.
    //
    //   udp rcv  - only ever prevents loss, so it stays generous
    //   udp snd  - a latency budget in front of the NIC, so it is small
    //   tcp tx   - queues for CUBIC, so it may be generous but is still bounded
    //   tcp rx   - the ADVERTISED WINDOW, i.e. the download throughput ceiling,
    //              so it is sized to a desktop bandwidth-delay product
    //
    // The receive window is what changed most: 1 MB over a 250 ms two-hop
    // chained path is ~4 MB/s per flow, and a browser opens many flows. It is
    // not sized to available RAM either - past a real BDP the extra only buys
    // standing queue, which is latency, not speed.
    let (
        scan_concurrency_cap,
        udp_socket_rcv_buf,
        udp_socket_snd_buf,
        netstack_tcp_tx_buf,
        netstack_tcp_rx_buf,
        netstack_udp_buf,
        channel_capacity,
        h2_stream_window,
        h2_conn_window,
        h2_frame_size,
        h2_send_buffer,
        h2_batch_packets,
    ) = match tier {
        Tier::Low => (
            4usize,
            512 * 1024,
            64 * 1024,
            128 * 1024,
            256 * 1024,
            32 * 1024,
            128usize,
            512 * 1024u32,
            1024 * 1024u32,
            64 * 1024u32,
            256 * 1024usize,
            8usize,
        ),
        Tier::Medium => (
            10usize,
            2 * 1024 * 1024,
            192 * 1024,
            256 * 1024,
            512 * 1024,
            64 * 1024,
            512usize,
            1024 * 1024u32,
            2 * 1024 * 1024u32,
            128 * 1024u32,
            384 * 1024usize,
            16usize,
        ),
        Tier::High => (
            usize::MAX,
            7 * 1024 * 1024,
            384 * 1024,
            384 * 1024,
            1024 * 1024,
            128 * 1024,
            1024usize,
            2 * 1024 * 1024u32,
            4 * 1024 * 1024u32,
            256 * 1024u32,
            512 * 1024usize,
            32usize,
        ),
    };

    Tuning {
        tier,
        cpus,
        mem_mb,
        scan_concurrency_cap,
        udp_socket_rcv_buf,
        udp_socket_snd_buf,
        netstack_tcp_tx_buf,
        netstack_tcp_rx_buf,
        netstack_udp_buf,
        channel_capacity,
        h2_stream_window: env_u32("AETHER_H2_STREAM_WINDOW", h2_stream_window)
            .clamp(64 * 1024, H2_MAX_WINDOW),
        h2_conn_window: env_u32("AETHER_H2_CONN_WINDOW", h2_conn_window)
            .clamp(64 * 1024, H2_MAX_WINDOW),
        h2_frame_size: env_u32("AETHER_H2_FRAME_SIZE", h2_frame_size)
            .clamp(16 * 1024, 16 * 1024 * 1024 - 1),
        h2_send_buffer: env_usize("AETHER_H2_SEND_BUFFER", h2_send_buffer)
            .clamp(128 * 1024, 16 * 1024 * 1024),
        h2_batch_packets: env_usize("AETHER_H2_BATCH", h2_batch_packets).clamp(1, 256),
    }
}

/// Largest window HTTP/2 flow control can express (RFC 9113 6.9.1).
const H2_MAX_WINDOW: u32 = (1u32 << 31) - 1;

fn env_u32(key: &str, fallback: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(fallback)
}

fn env_usize(key: &str, fallback: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(fallback)
}

pub fn tuning() -> &'static Tuning {
    TUNING.get_or_init(build_tuning)
}

pub fn log_summary() {
    let t = tuning();
    let mem = t
        .mem_mb
        .map(|m| format!("{m}MB"))
        .unwrap_or_else(|| "unknown".to_string());
    let cap = if t.scan_concurrency_cap == usize::MAX {
        "unlimited".to_string()
    } else {
        t.scan_concurrency_cap.to_string()
    };
    // This exact string is a BUILD FINGERPRINT. `udp socket rcv/snd=` is the
    // 1.2.3-p1 wording; a log that still shows `udp socket buffer=NNNNKB` (one
    // figure) or `netstack buffers=` is an engine from before the throughput
    // work, so nothing diagnosed since is being tested. Do not reword casually.
    log::info!(
        "[*] performance profile: {:?} (cpus={} mem={}); scan concurrency cap={}, udp socket rcv/snd={}KB/{}KB (snd = uplink queue budget), netstack tcp tx/rx={}KB/{}KB (rx = advertised window), netstack udp={}KB, channel capacity={}",
        t.tier,
        t.cpus,
        mem,
        cap,
        t.udp_socket_rcv_buf / 1024,
        t.udp_socket_snd_buf / 1024,
        t.netstack_tcp_tx_buf / 1024,
        t.netstack_tcp_rx_buf / 1024,
        t.netstack_udp_buf / 1024,
        t.channel_capacity,
    );
    // 1.2.3-p2 BUILD FINGERPRINT for the HTTP/2 carrier. If a log shows the
    // `performance profile:` line above but NOT this one, the engine predates
    // the carrier work and the 64 KB window is still in force.
    log::info!(
        "[*] h2 carrier profile: stream window={}KB, connection window={}KB, max frame={}KB, send buffer={}KB, packet batch={}",
        t.h2_stream_window / 1024,
        t.h2_conn_window / 1024,
        t.h2_frame_size / 1024,
        t.h2_send_buffer / 1024,
        t.h2_batch_packets,
    );
}

pub fn cap_concurrency(requested: usize) -> usize {
    requested.min(tuning().scan_concurrency_cap)
}

/// `SO_RCVBUF` to request on data-plane datagram sockets.
pub fn udp_socket_rcv_buf_bytes() -> usize {
    tuning().udp_socket_rcv_buf
}

/// `SO_SNDBUF` to request on data-plane datagram sockets.
///
/// Deliberately much smaller than the receive side. See
/// [`Tuning::udp_socket_snd_buf`].
pub fn udp_socket_snd_buf_bytes() -> usize {
    tuning().udp_socket_snd_buf
}

/// Per-flow smoltcp TCP send buffer.
pub fn netstack_tcp_tx_buf_bytes() -> usize {
    tuning().netstack_tcp_tx_buf
}

/// Per-flow smoltcp TCP receive buffer, i.e. the advertised receive window.
pub fn netstack_tcp_rx_buf_bytes() -> usize {
    tuning().netstack_tcp_rx_buf
}

pub fn netstack_udp_buf_bytes() -> usize {
    tuning().netstack_udp_buf
}

pub fn channel_capacity() -> usize {
    tuning().channel_capacity
}

/// Per-stream receive window to advertise on the HTTP/2 MASQUE carrier.
pub fn h2_stream_window() -> u32 {
    tuning().h2_stream_window
}

/// Connection-level receive window to advertise on the HTTP/2 MASQUE carrier.
pub fn h2_conn_window() -> u32 {
    tuning().h2_conn_window
}

/// Largest DATA frame the HTTP/2 MASQUE carrier will accept from the edge.
pub fn h2_frame_size() -> u32 {
    tuning().h2_frame_size
}

/// Outbound buffer budget on the HTTP/2 MASQUE carrier.
pub fn h2_send_buffer() -> usize {
    tuning().h2_send_buffer
}

/// Packets that may share one HTTP/2 DATA frame.
pub fn h2_batch_packets() -> usize {
    tuning().h2_batch_packets
}
