// پورت از ui/AdvancedPanel.kt (+ SegmentedSelector.kt ، DropdownSelector.kt ، LtrInput.kt ، AppPickerDialog.kt)
// توجه: گزینهٔ Proxy Mode عمداً حذف شده — در ویندوز کاربردی ندارد.
// v8:
//   * پروتکل «Auto» به «Smart» تغییر نام داد — دقیقاً هم‌نام نسخهٔ موبایل.
//   * انتخاب زبان برنامه (English/فارسی) + دکمهٔ «بازنشانی به تنظیمات پیش‌فرض».
import { invoke } from '@tauri-apps/api/core'
import { app, saveProfile, rerender } from '../main.js'
import { t, getLang, setLang, LANGS } from '../i18n.js'

const PROTOCOLS = [
  ['SMART', 'Smart'],
  ['MASQUE', 'MASQUE'],
  ['WIREGUARD', 'WireGuard'],
  ['GOOL', 'WARP×2'],
]
const SCAN_MODES = [
  ['TURBO', 'Turbo'],
  ['BALANCED', 'Balanced'],
  ['THOROUGH', 'Thorough'],
  ['STEALTH', 'Stealth'],
  ['IRONCLAD', 'Ironclad'],
]
const IP_VERSIONS = [['V4', 'IPv4'], ['V6', 'IPv6'], ['BOTH', 'Both']]
const NOIZE = [
  ['OFF', 'Off'], ['LIGHT', 'Light'], ['FIREWALL', 'Firewall'],
  ['BALANCED', 'Balanced'], ['GFW', 'GFW'], ['AGGRESSIVE', 'Aggressive'],
]
const ENDPOINT_MODES = [
  ['AUTO', 'Automatic'],
  ['MANUAL_PEER', 'Manual peer'],
  ['MANUAL_RANGE', 'Manual range'],
]
const SPLIT_MODES = [['OFF', 'Off'], ['INCLUDE', 'Only these apps'], ['EXCLUDE', 'All except these']]
const MTU_PRESETS = [1280, 1380, 1420, 1500]
const KEEPALIVE_PRESETS = [0, 10, 25, 45]

// v10 — هستهٔ 1.5.0: روش‌های ورود Zero Trust (همان گزینه‌های موبایل/هسته).
const ACCESS_MODES = [
  ['OFF', 'Off'],
  ['EMAIL', 'Email code'],
  ['SERVICE_TOKEN', 'Service token'],
  ['TOKEN', 'Access token'],
]

function segmented(label, key, options, current) {
  return `
    <section class="field">
      <span class="field__label">${label}</span>
      <div class="seg" role="radiogroup" data-key="${key}">
        ${options.map(([v, tx]) => `
          <button type="button" role="radio" class="seg__item ${current === v ? 'is-active' : ''}"
                  data-value="${v}" aria-checked="${current === v}">${t(tx)}</button>`).join('')}
      </div>
    </section>`
}

function dropdown(label, key, options, current) {
  return `
    <section class="field field--row">
      <span class="field__label">${label}</span>
      <select class="select" data-key="${key}">
        ${options.map(([v, tx]) => `<option value="${v}" ${current === v ? 'selected' : ''}>${t(tx)}</option>`).join('')}
      </select>
    </section>`
}

function toggle(label, key, hint, on) {
  return `
    <section class="field field--row">
      <div>
        <span class="field__label">${label}</span>
        ${hint ? `<span class="field__hint">${hint}</span>` : ''}
      </div>
      <button type="button" class="switch ${on ? 'is-on' : ''}" data-key="${key}" role="switch" aria-checked="${on}">
        <span class="switch__knob"></span>
      </button>
    </section>`
}

function textField(label, key, value, placeholder, opts = {}) {
  const type = opts.secret ? 'password' : 'text'
  const hint = opts.hint ? `<span class=\"field__hint\">${opts.hint}</span>` : ''
  return `
    <section class="field">
      <span class="field__label">${label}</span>
      <input class="input ltr" dir="ltr" type="${type}" data-key="${key}" value="${value ?? ''}" placeholder="${placeholder}" ${opts.secret ? 'autocomplete="off"' : ''}>
      ${hint}
    </section>`
}

// v10: تری‌ایریای چندخطی برای فهرست‌ها (routing/dns) — هر خط یک قاعده.
function listArea(label, key, values, placeholder, hint) {
  return `
    <section class="field">
      <span class="field__label">${label}</span>
      <textarea class="input input--area ltr" dir="ltr" data-key="${key}"
        placeholder="${placeholder}">${(values || []).join('\n')}</textarea>
      ${hint ? `<span class=\"field__hint\">${hint}</span>` : ''}
    </section>`
}

export function renderAdvanced() {
  const p = app.profile
  const root = document.createElement('div')
  root.className = 'view view--advanced'
  root.innerHTML = `
    <h2 class="view__title">${t('Advanced')}</h2>

    ${segmented(t('Language'), '__lang', LANGS, getLang())}

    ${segmented(t('Protocol'), 'protocol', PROTOCOLS, p.protocol)}
    ${segmented(t('Scan mode'), 'scanMode', SCAN_MODES, p.scanMode)}
    ${segmented(t('IP version'), 'ipVersion', IP_VERSIONS, p.ipVersion)}
    ${dropdown(t('Noize'), 'noize', NOIZE, p.noize)}
    ${dropdown(t('Endpoint'), 'endpointMode', ENDPOINT_MODES, p.endpointMode)}

    <div id="endpoint-extra">
      ${p.endpointMode === 'MANUAL_PEER' ? textField(t('Peer address'), 'manualPeer', p.manualPeer, '1.2.3.4:443') : ''}
      ${p.endpointMode === 'MANUAL_RANGE' ? textField(t('Address range'), 'manualRange', p.manualRange, '162.159.192.0/24') : ''}
    </div>

    ${dropdown('MTU', 'mtu', MTU_PRESETS.map((v) => [String(v), String(v)]), String(p.mtu))}
    ${dropdown('Keepalive', 'keepalive', KEEPALIVE_PRESETS.map((v) => [String(v), v === 0 ? t('Off') : `${v}s`]), String(p.keepalive))}

    ${toggle(t('Quick reconnect'), 'quickReconnect', t('Reconnect instantly after a drop'), p.quickReconnect)}
    ${toggle(t('MASQUE over HTTP/2'), 'masqueHttp2', t('Helps on networks that block HTTP/3'), p.masqueHttp2)}
    ${toggle(t('Packet fragmentation'), 'fragment', t('Splits the handshake to evade filtering'), p.fragment)}
    ${toggle('ECH', 'ech', t('Encrypted Client Hello (auto)'), p.ech)}
    ${toggle(t('Share over LAN'), 'lanShare', t('Let other devices on your network use this tunnel'), p.lanShare)}

    ${dropdown(t('Split tunneling'), 'splitMode', SPLIT_MODES, p.splitMode)}
    <section class="field" id="split-apps" ${p.splitMode === 'OFF' ? 'hidden' : ''}>
      <span class="field__label">${t('Applications')}</span>
      <textarea class="input input--area ltr" dir="ltr" data-key="splitApps"
        placeholder="chrome.exe&#10;telegram.exe">${(p.splitApps || []).join('\n')}</textarea>
      <span class="field__hint">${t('One executable name per line.')}</span>
    </section>

    <h3 class="view__subtitle">${t('Zero Trust')}</h3>
    <section class="field" id="caps-note" hidden>
      <span class="field__hint" id="caps-note-text"></span>
    </section>
    <div id="v15-zt">
    ${textField(t('Team name'), 'team', p.team, 'your-team', { hint: t('Connect as a managed device of a Cloudflare Zero Trust organization. Leave empty for normal WARP.') })}
    <div id="zt-extra" ${(p.team || '').trim() ? '' : 'hidden'}>
      ${segmented(t('Sign-in method'), 'accessMode', ACCESS_MODES, p.accessMode)}
      <div id="zt-fields">
        ${p.accessMode === 'EMAIL' ? textField(t('Access email'), 'accessEmail', p.accessEmail, 'user@example.com', { hint: t('A one-time code is sent to this mailbox on connect.') }) : ''}
        ${p.accessMode === 'SERVICE_TOKEN' ? textField('Access ID', 'accessId', p.accessId, 'xxxxxxxx.access', {}) : ''}
        ${p.accessMode === 'SERVICE_TOKEN' ? textField('Access Secret', 'accessSecret', '', '••••••', { secret: true, hint: t('Stored in memory only — never written to disk.') }) : ''}
        ${p.accessMode === 'TOKEN' ? textField('Access Token (JWT)', 'accessToken', '', '••••••', { secret: true, hint: t('Stored in memory only — never written to disk.') }) : ''}
      </div>
      ${toggle(t('Gateway proxy'), 'gateway', t('Route HTTP/HTTPS through your organization\'s Gateway (adds a hop and logs browsing)'), p.gateway)}
    </div>
    </div>

    <div id="v15-routing">
    <h3 class="view__subtitle">${t('Routing rules')}</h3>
    ${listArea(t('Blocked destinations'), 'routeBlock', p.routeBlock, 'ads.example.com&#10;203.0.113.0/24', t('One rule per line — domain, IP or CIDR. These connections are refused.'))}
    ${listArea(t('Direct destinations'), 'routeDirect', p.routeDirect, 'bank-domain.ir&#10;192.168.0.0/16', t('One rule per line. These bypass the tunnel — for banking apps, LAN services and domestic sites.'))}
    </div>

    <div id="v15-dns">
    <h3 class="view__subtitle">DNS</h3>
    ${listArea(t('In-tunnel DNS servers'), 'dns', p.dns, '1.1.1.1&#10;9.9.9.9', t('Resolvers used inside the tunnel. Empty = engine defaults.'))}
    </div>

    <section class="field field--row">
      <div>
        <span class="field__label">${t('Reset to defaults')}</span>
        <span class="field__hint">${t('Restores every setting above to its factory value')}</span>
      </div>
      <button type="button" class="btn btn--danger" id="reset-defaults">${t('Reset')}</button>
    </section>
  `

  // --- سیم‌کشی — هر تغییر فوراً ذخیره می‌شود (مثل DataStore در اندروید)
  root.querySelectorAll('.seg__item').forEach((b) => {
    b.addEventListener('click', async () => {
      const key = b.closest('.seg').dataset.key
      // زبان برنامه عضو profile نیست؛ جدا ذخیره و کل پوسته دوباره رندر می‌شود.
      if (key === '__lang') {
        setLang(b.dataset.value)
        rerender()
        return
      }
      await saveProfile({ [key]: b.dataset.value })
      b.closest('.seg').querySelectorAll('.seg__item').forEach((x) => {
        x.classList.toggle('is-active', x === b)
        x.setAttribute('aria-checked', String(x === b))
      })
      // v10: تغییر روش ورود Zero Trust فیلدهای متفاوتی می‌خواهد — بازرندر.
      if (key === 'accessMode') {
        const host = root.parentElement
        host.innerHTML = ''
        host.appendChild(renderAdvanced())
      }
    })
  })

  root.querySelectorAll('.select').forEach((s) => {
    s.addEventListener('change', async () => {
      const key = s.dataset.key
      const raw = s.value
      const value = ['mtu', 'keepalive'].includes(key) ? Number(raw) : raw
      await saveProfile({ [key]: value })
      if (key === 'endpointMode' || key === 'splitMode') {
        const host = root.parentElement
        host.innerHTML = ''
        host.appendChild(renderAdvanced())
      }
    })
  })

  root.querySelectorAll('.switch').forEach((tg) => {
    tg.addEventListener('click', async () => {
      const on = !tg.classList.contains('is-on')
      tg.classList.toggle('is-on', on)
      tg.setAttribute('aria-checked', String(on))
      await saveProfile({ [tg.dataset.key]: on })
    })
  })

  root.querySelectorAll('.input').forEach((i) => {
    i.addEventListener('change', async () => {
      const key = i.dataset.key
      // v10: فیلدهای فهرستی — هر خط یک مقدار (مثل splitApps).
      const LIST_KEYS = ['splitApps', 'routeBlock', 'routeDirect', 'dns']
      const value = LIST_KEYS.includes(key)
        ? i.value.split('\n').map((x) => x.trim()).filter(Boolean)
        : i.value.trim()
      await saveProfile({ [key]: value })
      // v10: پاک/پر شدن نام تیم، بخش Zero Trust را نشان/پنهان می‌کند.
      if (key === 'team') {
        const host = root.parentElement
        host.innerHTML = ''
        host.appendChild(renderAdvanced())
      }
      // سخت‌سازی امنیتی: مقدار محرمانه بعد از ذخیره از DOM پاک می‌شود تا
      // در اسکرین‌شات/بازرسی DOM نماند (ذخیره فقط در حافظهٔ بک‌اند است).
      if (key === 'accessSecret' || key === 'accessToken') {
        i.value = ''
        i.placeholder = '•••••• (saved)'
      }
    })
  })

  // بازنشانی به تنظیمات کارخانه — دستور reset_profile سمت Rust پروفایل
  // پیش‌فرض را ذخیره و همان را برمی‌گرداند؛ زبان کاربر دست نمی‌خورد.
  root.querySelector('#reset-defaults').addEventListener('click', async () => {
    const fresh = await invoke('reset_profile')
    app.profile = fresh
    rerender()
  })

  // v10: قابلیت‌سنجی هسته — اگر هستهٔ همراه قدیمی‌تر از 1.5.0 باشد (مثلاً
  // وقتی کاربر نسخهٔ هسته را در پایپ‌لاین پین کرده)، این بخش‌ها غیرفعال و
  // با توضیح نشان داده می‌شوند تا کاربر تنظیمی را پر نکند که بی‌اثر است.
  // خطای این فراخوانی هرگز صفحه را نمی‌شکند.
  invoke('core_caps')
    .then((caps) => {
      const gate = (id, enabled) => {
        const el = root.querySelector(id)
        if (!el || enabled) return
        el.querySelectorAll('input, textarea, button, select').forEach((c) => {
          c.disabled = true
        })
        el.style.opacity = '0.45'
      }
      gate('#v15-zt', caps.zeroTrust)
      gate('#v15-routing', caps.routing)
      gate('#v15-dns', caps.customDns)
      if (!caps.zeroTrust || !caps.routing || !caps.customDns) {
        const note = root.querySelector('#caps-note')
        const text = root.querySelector('#caps-note-text')
        if (note && text) {
          text.textContent = t('These features need engine core 1.5.0 or newer. The bundled core is older, so they are disabled.')
          note.hidden = false
        }
      }
    })
    .catch(() => {})

  return root
}
