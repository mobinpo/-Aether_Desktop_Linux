// =============================================================================
//  پورت ۱:۱ از `ui/components/ConnectionCard.kt` — بلوکِ پایینیِ صفحهٔ اصلی.
// -----------------------------------------------------------------------------
//  تا نسخهٔ ۱.۲.۳ دسکتاپ، ناحیهٔ زیر دکمهٔ پاور چهار سطحِ شناورِ جدا بود —
//  متن وضعیت، نشان آی‌پی، سنجهٔ ترافیک و ردیف پروتکل/اندپوینت/تأخیر — هر یک
//  با رنگ، شعاع و padding خودش (`.ipbadge`, `.meta`, `.traffic` در app.css).
//  روی یک پنجره شبیه شلختگی خوانده می‌شد و هیچ‌چیز آن‌ها را به هم وصل نمی‌کرد.
//
//  حالا یک کارتِ گلس‌مورفیک واحد است، با یک نظام رنگ و یک سلسله‌مراتب عمودی
//  ثابت — دقیقاً همان هفت گام مخزن موبایل:
//
//    ۱. وضعیت اتصال   (درشت، mint، با یک سطر آرام «برای قطع لمس کنید»)
//    ۲. تایمر نشست    («مدت اتصال» + HH:MM:SS با چهرهٔ مونو/دیجیتال)
//    ۳. پیل آی‌پی سرور (برچسب + پرچم کشور + نشانی)
//    ۴. نوار سرعت      (نرخ زندهٔ دانلود/آپلود و مجموع نشست)
//    ۵. اسلاید پروتکل  (تمام‌عرض)
//    ۶. اسلاید اندپوینت (تمام‌عرض)
//    ۷. اسلاید تأخیر   (تمام‌عرض، همراه متر زندهٔ قدرت پینگ)
//
//  هیچ‌چیز بیرون بلوک شناور نیست: هر زیربخش، فرزندِ همان کارت است و از همان
//  پالت کشیده می‌شود.
//
//  رنگ‌ها عمداً از توکن‌های تم گرفته نمی‌شوند و در `app.css` به همان مقادیر
//  برند پین شده‌اند (navy عمیق #0A0E1A، لهجهٔ mint #3EDBB0، یک سطح شیشهٔ
//  اسلیت برای همه‌چیزِ درونش) — همان دلیل موبایل: کارت باید در هر تمی مثل
//  اِتِر به نظر برسد.
//
//  یک اسلاید در هر واقعیت. سه گام آخر در نسخهٔ موبایل قبلاً یک نوار سه‌ستونی
//  بودند و `WIREGUARD → PSIPHON` یا یک `ip:port` هرگز در یک‌سومِ عرض جا
//  نمی‌شد؛ برچسبی که مقدارش خوانده نمی‌شود تزئین است. اینجا هم هر کدام تمام
//  عرض کارت را دارند: برچسب چپ، مقدار راست.
// =============================================================================

import { flagHtml } from '../flags.js'
import { t } from '../i18n.js'
import {
  GlowCycle,
  drawGlowCycle,
  fitCanvas,
  frameLoop,
  roundRectPath,
  TWO_PI,
} from '../ui/glowcycle.js'

const CARD_RADIUS = 26
const IDLE_ACCENT = '#4C8DFF'
const ERROR_ACCENT = '#FF5C7A'
const AETHER_MINT = '#3EDBB0'

// ---- تنظیم متر پینگ -------------------------------------------------------
// آستانه‌های درجه و رمپِ ارتفاع میله‌ها با هم اعلام می‌شوند تا خوانندهٔ بعدی
// ببیند که قرار است بر هم منطبق باشند: EXCELLENT هم سقفِ رمپ ارتفاع است و
// FAIR کفِ آن، پس مرزِ mint/کهربایی در رنگ و مرزِ Good/Fair در برچسب، به
// جای تصادف، *ساختاراً* یک عددند. این همان اشکالی است که در موبایل دو جدولِ
// آستانهٔ مستقل ساخته بود و هر خواندن بین ۱۶۱ تا ۱۸۹ میلی‌ثانیه، میلهٔ سبز
// زیر کلمهٔ «متوسط» نشان می‌داد.
const PING_EXCELLENT_MS = 80
const PING_GOOD_MS = 190
const PING_FAIR_MS = 320
const PING_BEST_MS = PING_EXCELLENT_MS
const PING_WORST_MS = 400
const PING_BARS = 26
const PING_BAR_GAP = 3
/** سقف پهنای هر میله — رجوع به توضیح در `paintWave`. */
const PING_BAR_MAX_W = 11
const PING_WAVE_MS = 1500
const PING_FAIR = '#FFB43C'
const PING_POOR = '#FF5C7A'
const CARD_TEXT_DIM = 'rgba(255,255,255,0.34)'

const DASH = '\u2014'

function formatBytes(v) {
  if (v < 1024) return `${Math.round(v)} B`
  const kb = v / 1024
  if (kb < 1024) return `${kb.toFixed(1)} KB`
  const mb = kb / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}

function formatRate(v) {
  return `${formatBytes(v)}/s`
}

/** `HH:MM:SS` — همان `String.format(Locale.US, "%02d:%02d:%02d", …)`. */
function formatClock(totalSecs) {
  const s = Math.max(0, Math.floor(totalSecs))
  const pad = (x) => String(x).padStart(2, '0')
  return `${pad(Math.floor(s / 3600))}:${pad(Math.floor((s % 3600) / 60))}:${pad(s % 60)}`
}

/**
 * تنها درجه‌ای که یک خواندنِ تأخیر دارد. رنگ *و* برچسب هر دو از همین می‌آیند.
 */
function pingGrade(connected, ms) {
  if (!connected) return 'offline'
  if (ms == null || ms < 0) return 'measuring'
  if (ms <= PING_EXCELLENT_MS) return 'excellent'
  if (ms <= PING_GOOD_MS) return 'good'
  if (ms <= PING_FAIR_MS) return 'fair'
  return 'poor'
}

function pingTint(grade) {
  switch (grade) {
    case 'excellent':
    case 'good':
      return AETHER_MINT
    case 'fair':
      return PING_FAIR
    case 'poor':
      return PING_POOR
    default:
      return CARD_TEXT_DIM
  }
}

function pingQualityLabel(grade) {
  switch (grade) {
    case 'excellent':
      return t('Excellent')
    case 'good':
      return t('Good')
    case 'fair':
      return t('Fair')
    case 'poor':
      return t('Poor')
    case 'measuring':
      return t('Measuring…')
    default:
      return t('Offline')
  }
}

/**
 * رمپ پیوستهٔ *ارتفاع* میله‌ها: ۱ = عالی، ۰ = بی‌استفاده.
 *
 * جدا از [pingGrade] نگه داشته شده چون یک انیمیشن مقدار پیوسته می‌خواهد؛ ولی
 * دیگر چیزی را تصمیم نمی‌گیرد که یک *کلمه* هم تصمیم می‌گیرد.
 */
function pingStrength(connected, ms) {
  if (!connected || ms == null || ms < 0) return 0
  if (ms <= PING_BEST_MS) return 1
  if (ms >= PING_WORST_MS) return 0.08
  return 1 - (ms - PING_BEST_MS) / (PING_WORST_MS - PING_BEST_MS)
}

/**
 * کارت را می‌سازد.
 *
 * @returns `{ node, paint(snapshot), destroy() }` — `paint` را جریان وضعیت
 *   صدا می‌زند و `destroy` هر دو حلقهٔ فریم را واقعاً لغو می‌کند.
 */
export function createConnectionCard() {
  const node = document.createElement('section')
  node.className = 'ccard'
  node.innerHTML = `
    <canvas class="ccard__edge" aria-hidden="true"></canvas>
    <div class="ccard__body">
      <div class="ccard__status">
        <p class="ccard__title" id="cc-title">${t('Disconnected')}</p>
        <p class="ccard__caption" id="cc-caption">${t('Tap to connect securely')}</p>
      </div>

      <div class="ccard__timer">
        <span class="ccard__timer-k">${t('Connected for')}</span>
        <span class="ccard__timer-v ltr" dir="ltr" id="cc-clock">00:00:00</span>
      </div>

      <div class="ccard__pill ltr" dir="ltr" id="cc-pill">
        <span class="ccard__pill-k" id="cc-ip-label">${t('Your IP')}</span>
        <span class="ccard__pill-flag" id="cc-ip-flag">${flagHtml(null)}</span>
        <span class="ccard__pill-v" id="cc-ip-value">${t('Checking IP…')}</span>
      </div>

      <div class="ccard__speeds">
        <div class="ccard__speed">
          <span class="ccard__speed-badge ccard__speed-badge--down" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor"
                 stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 5v13M6.5 12.5 12 18l5.5-5.5"/>
            </svg>
          </span>
          <span class="ccard__speed-text">
            <span class="ccard__speed-rate ltr" dir="ltr" id="cc-rx-rate">0 B/s</span>
            <span class="ccard__speed-total" id="cc-rx-total">${t('Total')} <bdi>0 B</bdi></span>
          </span>
        </div>
        <span class="ccard__divider" aria-hidden="true"></span>
        <div class="ccard__speed">
          <span class="ccard__speed-badge ccard__speed-badge--up" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor"
                 stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 19V6M6.5 11.5 12 6l5.5 5.5"/>
            </svg>
          </span>
          <span class="ccard__speed-text">
            <span class="ccard__speed-rate ltr" dir="ltr" id="cc-tx-rate">0 B/s</span>
            <span class="ccard__speed-total" id="cc-tx-total">${t('Total')} <bdi>0 B</bdi></span>
          </span>
        </div>
      </div>

      <div class="ccard__slide">
        <span class="ccard__slide-k">${t('Protocol')}</span>
        <span class="ccard__slide-v ltr" dir="ltr" id="cc-proto">${DASH}</span>
      </div>

      <div class="ccard__slide">
        <span class="ccard__slide-k">${t('Endpoint')}</span>
        <span class="ccard__slide-v ccard__slide-v--wrap ltr" dir="ltr" id="cc-endpoint">${DASH}</span>
      </div>

      <div class="ccard__slide ccard__slide--stack">
        <div class="ccard__slide-row">
          <span class="ccard__slide-k">${t('Latency')}</span>
          <span class="ccard__slide-v ltr" dir="ltr" id="cc-latency">${DASH}</span>
        </div>
        <div class="ccard__ping">
          <div class="ccard__ping-head">
            <span class="ccard__ping-k">${t('Ping strength')}</span>
            <span class="ccard__ping-q" id="cc-ping-q">${t('Offline')}</span>
          </div>
          <canvas class="ccard__ping-wave" id="cc-wave" aria-hidden="true"></canvas>
        </div>
      </div>
    </div>
  `

  const el = (id) => node.querySelector(`#${id}`)
  const edge = node.querySelector('.ccard__edge')
  const wave = el('cc-wave')

  // ---- وضعیتِ زندهٔ ترسیم ------------------------------------------------
  let accent = IDLE_ACCENT
  let connected = false
  // چرخهٔ نور: فقط وقتی متصل‌ایم ساخته می‌شود — بی‌هیچ اشتراکِ فریم در حالت
  // قطع، همان قاعدهٔ `if (connected) rememberGlowCycle() else null`.
  let cycle = null
  // آخرین خواندنِ *خوب*. حالت‌های گذار مقدار null می‌دهند و باور کردنشان
  // باعث می‌شد مقدار تأخیر و متر، هر چند ثانیه یک‌بار به «…» پلک بزنند.
  let lastMs = -1
  // ارتفاع میله‌ها به مقدار تازه سُر می‌خورد و نمی‌پرد (tween 700ms موبایل).
  let level = 0
  let levelTarget = 0
  let levelAt = performance.now()

  // نرخ لحظه‌ای از دلتای بایت‌ها روی ساعت یکنواخت — یک جهش ساعت دیواری
  // نباید بتواند یک اسپایک جعلی بسازد.
  let lastRx = -1
  let lastTx = -1
  let lastSampleAt = 0

  // ---- لبهٔ کارت -----------------------------------------------------------
  // یک تابش نرم درونی، یک حاشیهٔ مویی، و — تا وقتی متصل‌ایم — نور متحرک،
  // یک رنگ در هر دور.
  const drawEdgeFrame = () => {
    const box = node.getBoundingClientRect()
    if (box.width <= 0 || box.height <= 0) return
    fitCanvas(edge, box.width, box.height)
    const ctx = edge.getContext('2d')
    ctx.clearRect(0, 0, box.width, box.height)

    const hairline = 1
    // نور متحرک کمی پهن‌تر از حاشیهٔ ساختاری کشیده می‌شود: در ۱ پیکسل، هستهٔ
    // آن بین دو پیکسل دستگاه می‌افتد و ضدپله‌گذاری آن را به خطی گل‌آلود پهن
    // می‌کند. ۱.۵ بیشتر اوقات روی مرز پیکسل می‌نشیند و مثل رشتهٔ نور تیز
    // خوانده می‌شود.
    const lightStroke = 1.5
    const { path, perimeter } = roundRectPath(box.width, box.height, CARD_RADIUS, hairline / 2)

    const grad = ctx.createRadialGradient(
      box.width / 2, 0, 0,
      box.width / 2, 0, box.width * 0.95,
    )
    grad.addColorStop(0, hexAlpha(accent, 0.10))
    grad.addColorStop(1, hexAlpha(accent, 0))
    ctx.fillStyle = grad
    ctx.fill(path)

    ctx.strokeStyle = hexAlpha(accent, cycle ? 0.16 : 0.10)
    ctx.lineWidth = hairline
    ctx.setLineDash([])
    ctx.stroke(path)

    if (cycle) drawGlowCycle(ctx, path, perimeter, cycle, lightStroke)
  }

  // ---- متر قدرت پینگ ------------------------------------------------------
  // فضای خالیِ اسلاید تأخیر، خودِ اندازه‌گیری را نشان می‌دهد: موجی رونده از
  // میله‌های گرد که ارتفاع، روشنایی و رنگش کیفیت آخرین کاوش را دنبال می‌کند.
  // سرعتِ حرکت عمداً ثابت است: در موبایل مشتق‌کردنش از تأخیر باعث می‌شد spec
  // در هر کاوش عوض شود و موج هر چهار ثانیه یک‌بار به‌چشم بپرد.
  const drawWaveFrame = () => {
    const box = wave.getBoundingClientRect()
    if (box.width <= 0 || box.height <= 0) return
    fitCanvas(wave, box.width, box.height)
    const ctx = wave.getContext('2d')
    ctx.clearRect(0, 0, box.width, box.height)

    const now = performance.now()
    // میان‌یابی ساعت‌محور به سمت هدف — معادل animateFloatAsState(tween(700)).
    const dt = Math.min(1, (now - levelAt) / 700)
    levelAt = now
    level += (levelTarget - level) * dt

    const grade = pingGrade(connected, lastMs)
    const tint = pingTint(grade)
    const phase = connected ? (now % PING_WAVE_MS) / PING_WAVE_MS : 0
    const gap = PING_BAR_GAP
    // ۱.۲.۴-p4 — میله‌ها سقف پهنا دارند و ردیف وسط‌چین می‌شود.
    //
    // پیش از این پهنا فقط تقسیمِ عرضِ کارت بر ۲۶ بود، پس هر پیکسلی که کارت
    // پهن‌تر می‌شد مستقیماً به میله‌ها می‌رفت و دایره‌های موبایل روی دسکتاپ به
    // کپسولِ کشیده تبدیل می‌شدند. ۱۱ پیکسل همان پهنایی است که ۲۶ میله در کارت
    // گوشی دارند؛ سقف می‌گذاریم و نه مقدار ثابت، تا در پنجرهٔ باریک باز هم جا شود.
    const barWidth = Math.min(PING_BAR_MAX_W, (box.width + gap) / PING_BARS - gap)
    if (barWidth <= 0) return
    const pitch = barWidth + gap
    const rowWidth = PING_BARS * pitch - gap
    const originX = (box.width - rowWidth) / 2
    const radius = barWidth / 2
    const floorH = box.height * 0.16

    for (let i = 0; i < PING_BARS; i++) {
      const x = i / (PING_BARS - 1)
      // دو هارمونیک در دو جهت مخالف: قلهٔ موج هرگز ثابت نمی‌ماند و الگو به
      // شکلی تکرار نمی‌شود که چشم رویش قفل کند.
      const a = 0.5 + 0.5 * Math.sin(TWO_PI * (2.0 * x - phase))
      const b = 0.5 + 0.5 * Math.sin(TWO_PI * (3.7 * x + 1.6 * phase))
      const w = 0.58 * a + 0.42 * b
      const h = floorH + (box.height - floorH) * level * w
      const top = (box.height - h) / 2
      const left = originX + i * pitch
      const alpha = 0.30 + 0.70 * w * (0.25 + 0.75 * level)

      ctx.fillStyle = hexAlpha(tint, Math.min(1, Math.max(0, alpha)))
      roundBar(ctx, left, top, barWidth, h, radius)
    }
  }

  // حلقه‌های فریم، قابل شروع و توقف.
  //
  // نماها در `main.js` کَش می‌شوند و تعویض تب فقط جدا/وصلشان می‌کند؛ اگر این دو
  // حلقه به عمر *نما* گره نخورند، کارتِ نامرئی تا پایان عمر برنامه شصت فریم در
  // ثانیه می‌کشد — همان هزینهٔ پس‌زمینه‌ای که ۱.۲.۳ برای امیت‌کردن snapshotِ
  // بی‌تغییر می‌داد و ریشه‌اش کنده شد.
  let stopEdge = null
  let stopWave = null

  function start() {
    if (!stopEdge) stopEdge = frameLoop(drawEdgeFrame)
    if (!stopWave) stopWave = frameLoop(drawWaveFrame)
  }

  function stop() {
    if (stopEdge) { stopEdge(); stopEdge = null }
    if (stopWave) { stopWave(); stopWave = null }
  }

  /**
   * یک بار رنگ‌آمیزی از روی یک snapshot.
   *
   * سرعت‌ها *فقط* اینجا نمونه‌برداری می‌شوند و نه در حلقهٔ فریم: نرخ باید از
   * دلتای بایت‌های واقعیِ موتور بیاید، و موتور هر ۲۰۰ میلی‌ثانیه یک snapshot
   * می‌فرستد.
   */
  function paint(snapshot, opts) {
    const busy = !!(opts && opts.busy)
    const failed = snapshot.state === 'FAILED'
    connected = snapshot.state === 'CONNECTED'
    accent = failed ? ERROR_ACCENT : connected ? AETHER_MINT : IDLE_ACCENT

    if (connected && !cycle) cycle = new GlowCycle()
    if (!connected && cycle) cycle = null

    node.style.setProperty('--cc-accent', accent)
    node.classList.toggle('is-on', connected)
    node.classList.toggle('is-error', failed)

    // ۱. وضعیت
    const title = el('cc-title')
    title.textContent = opts.title
    title.style.color = accent
    el('cc-caption').textContent = opts.caption

    // ۲. تایمر — در حالت قطع، خاموش و کم‌رنگ می‌ماند و صفر نشان می‌دهد؛
    //    ردیف هرگز از کارت حذف نمی‌شود تا ارتفاع کارت نلرزد.
    const clock = el('cc-clock')
    clock.textContent = formatClock(connected ? snapshot.uptimeSecs : 0)
    clock.classList.toggle('is-dim', !connected)

    // ۳. پیل آی‌پی
    const info = snapshot.ipInfo
    const viaTunnel = !!(info && info.viaTunnel)
    el('cc-ip-label').textContent = viaTunnel ? t('Server IP') : t('Your IP')
    const pill = el('cc-pill')
    pill.classList.toggle('is-tunnel', viaTunnel)
    const flagBox = el('cc-ip-flag')
    // پرچم SVG درون‌ساخت است چون ویندوز فونت ایموجی پرچم ندارد و کد کشور
    // به‌صورت دو حرف («DE») رندر می‌شد.
    flagBox.hidden = !info
    if (info) flagBox.innerHTML = flagHtml(info.countryCode)
    el('cc-ip-value').textContent = busy
      ? t('Checking IP…')
      : info
        ? info.ip
        : snapshot.ipLoading
          ? t('Checking IP…')
          : t('IP unavailable')

    // ۴. سرعت‌ها
    if (connected) {
      const now = performance.now()
      if (lastSampleAt > 0 && now > lastSampleAt && lastRx >= 0) {
        const seconds = (now - lastSampleAt) / 1000
        const rx = Math.max(0, (snapshot.rxBytes - lastRx) / seconds)
        const tx = Math.max(0, (snapshot.txBytes - lastTx) / seconds)
        el('cc-rx-rate').textContent = formatRate(rx)
        el('cc-tx-rate').textContent = formatRate(tx)
      }
      lastRx = snapshot.rxBytes
      lastTx = snapshot.txBytes
      lastSampleAt = now
      el('cc-rx-total').innerHTML = `${t('Total')} <bdi>${formatBytes(snapshot.rxBytes)}</bdi>`
      el('cc-tx-total').innerHTML = `${t('Total')} <bdi>${formatBytes(snapshot.txBytes)}</bdi>`
    } else {
      lastRx = -1
      lastTx = -1
      lastSampleAt = 0
      el('cc-rx-rate').textContent = '0 B/s'
      el('cc-tx-rate').textContent = '0 B/s'
      el('cc-rx-total').innerHTML = `${t('Total')} <bdi>0 B</bdi>`
      el('cc-tx-total').innerHTML = `${t('Total')} <bdi>0 B</bdi>`
    }

    // ۵–۷. اسلایدهای متا
    el('cc-proto').textContent = connected ? (snapshot.protocol ?? DASH) : DASH
    el('cc-endpoint').textContent = connected ? (snapshot.endpoint ?? '\u2026') : DASH

    lastMs = !connected ? -1 : snapshot.latencyMs != null ? snapshot.latencyMs : lastMs
    el('cc-latency').textContent = !connected
      ? DASH
      : lastMs >= 0
        ? `${lastMs} ms`
        : '\u2026'

    const grade = pingGrade(connected, lastMs)
    const quality = el('cc-ping-q')
    quality.textContent = pingQualityLabel(grade)
    quality.style.color = connected ? pingTint(grade) : CARD_TEXT_DIM
    levelTarget = pingStrength(connected, lastMs)
  }

  start()

  return { node, paint, start, stop }
}

/** مستطیل گردگوشه با شعاع بریده‌شده — معادل `drawRoundRect` سمت Compose. */
function roundBar(ctx, x, y, w, h, r) {
  const radius = Math.min(r, w / 2, h / 2)
  ctx.beginPath()
  ctx.moveTo(x + radius, y)
  ctx.lineTo(x + w - radius, y)
  ctx.arcTo(x + w, y, x + w, y + radius, radius)
  ctx.lineTo(x + w, y + h - radius)
  ctx.arcTo(x + w, y + h, x + w - radius, y + h, radius)
  ctx.lineTo(x + radius, y + h)
  ctx.arcTo(x, y + h, x, y + h - radius, radius)
  ctx.lineTo(x, y + radius)
  ctx.arcTo(x, y, x + radius, y, radius)
  ctx.closePath()
  ctx.fill()
}

/**
 * `#RRGGBB` را با آلفا به `rgba()` تبدیل می‌کند.
 *
 * عمداً و نه `color-mix()`: این رنگ‌ها داخل canvas مصرف می‌شوند، و canvas
 * توابع رنگ CSS را حساب نمی‌کند — یک رشتهٔ نامعتبر در `fillStyle` بی‌صدا
 * نادیده گرفته می‌شود و نتیجه‌اش کارتی است بدون هیچ نور.
 */
function hexAlpha(hex, alpha) {
  if (hex.startsWith('rgba')) return hex
  const n = parseInt(hex.slice(1), 16)
  const r = (n >> 16) & 0xff
  const g = (n >> 8) & 0xff
  const b = n & 0xff
  return `rgba(${r},${g},${b},${Math.min(1, Math.max(0, alpha))})`
}
