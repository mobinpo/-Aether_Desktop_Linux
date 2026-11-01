#!/usr/bin/env python3
"""Static compatibility guard for the Psiphon and engine regressions of 1.3.0.

Not a replacement for a Windows build or a connection test. Run in CI preflight.
"""
from pathlib import Path
import json
import re
import sys

root = Path(__file__).resolve().parent.parent
pin = '83aa73b9b982e7421e00117f5b0c5aceb5dda452'

def read(rel):
    return (root / rel).read_text(encoding='utf-8')

def require(ok, why):
    if not ok:
        print('FAIL:', why, file=sys.stderr)
        raise SystemExit(1)
    print('OK:', why)

require(read('CORE_VERSION').strip() == read('native/aether/CORE_VERSION').strip() == '2.4.0', 'both engine version files are 2.4.0')
require(re.search(r'^version\s*=\s*"2\.4\.0"', read('native/aether/aether/Cargo.toml'), re.M), 'vendored engine Cargo version is 2.4.0')
require(json.loads(read('package.json'))['version'] == json.loads(read('src-tauri/tauri.conf.json'))['version'] == '1.4.0', 'desktop version stays 1.4.0')
# فورکِ لینوکسی ورک‌فلوها را اسپلیت کرده است؛ هر نامی که هست خوانده می‌شود.
for _wf in ('.github/workflows/build.yml', '.github/workflows/build-windows.yml', '.github/workflows/build-linux.yml'):
    if Path(_wf).exists():
        workflow = read(_wf)
        break
else:
    raise SystemExit('FAIL: no workflow file found')
script = read('scripts/build-psiphon.ps1')
engine = read('src-tauri/src/engine.rs')
core = read('native/aether/aether/src/psiphon.rs')
# فورکِ لینوکسی استیج ۲ سایفون را از بستهٔ رسمی می‌گیرد (fetch-engine.sh، گزارش
# در همین مخزن)؛ build-psiphon.ps1 بالادستی اینجا فقط به‌عنوانِ شاهدِ پین نگه
# داشته می‌شود، پس چک‌ها را با زنجیرهٔ واقعی فورک هم‌سنج می‌کنیم.
require(pin in script, 'build-psiphon.ps1 (upstream keeper) still pins the reviewed commit')
require('AETHER_PSIPHON_SERVER_ENTRIES' in engine and 'AETHER_PSIPHON_SERVER_ENTRIES' in core and 'client_arguments(&config_path, state, entries.as_deref())' in core, 'the supplied server list reaches the core-managed Psiphon child')
require('FRONTED-MEEK-CDN-OSSH' in core, 'the name rejected by the old Psiphon binary is covered by the matching fork')
require("'Waiting for engine-managed Psiphon…': 'در انتظار راه‌اندازی سایفون توسط هسته…'" in read('src/i18n.js'), 'Persian status line has an exact translation')
require('fetch-engine.sh' in workflow, 'CI stages stage-2 Psiphon through the official engine package (fetch-engine.sh)')
print('Psiphon and core static guard passed; runtime connection testing remains necessary.')
