#!/usr/bin/env python3
"""Check that every bundle resource exists, using the right names per platform.

One script for all three platforms, because the names are not the same and
keeping three copies in step is how they got out of step. `tauri.conf.json`
carries the extensionless names (Linux, macOS); `tauri.windows.conf.json`
overrides them with `.exe`. Tauri merges that override when the build target
is Windows — this reads the same two files, so the check sees what the build
will see.

Usage: check-bundle-resources.py [windows|macos|linux]
"""

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
CONFIG_DIR = os.path.join(os.path.dirname(HERE), "src-tauri")

# هر پلتفرم فایلِ overrideِ خودش را دارد. نبودنش خطا نیست — مک و لینوکس
# امروز نامِ یکسان دارند و به فایلِ جدا نیاز ندارند.
OVERRIDES = {
    "windows": "tauri.windows.conf.json",
    "macos": "tauri.macos.conf.json",
    "linux": "tauri.linux.conf.json",
}


def resources_for(platform: str) -> dict:
    """The resource map Tauri will use for this platform.

    json_patch merges per key, so the override cannot replace the map — it
    only overwrites keys it names. The platform files therefore set the base
    keys to "" to drop them (Tauri resolves an empty target to nothing) and
    add their own. This mirrors that exactly: merge, then drop the empties.
    """
    with open(os.path.join(CONFIG_DIR, "tauri.conf.json")) as fh:
        config = json.load(fh)
    resources = dict(config["bundle"]["resources"])
    override = OVERRIDES.get(platform)
    if override:
        path = os.path.join(CONFIG_DIR, override)
        if os.path.exists(path):
            with open(path) as fh:
                resources.update(json.load(fh)["bundle"]["resources"])
    return {src: dst for src, dst in resources.items() if dst}


def main() -> int:
    platform = sys.argv[1] if len(sys.argv) > 1 else "linux"
    if platform not in OVERRIDES:
        print(f"unknown platform '{platform}'", file=sys.stderr)
        return 2

    resources = resources_for(platform)
    # فقط منابعی که واقعاً در نقشهٔ نهایی‌اند — کلیدهای خالی که یک پلتفرم
    # برای حذفِ نام‌های پایه گذاشته، در `resources_for` حذف شده‌اند و اینجا هم
    # نباید دیده شوند.
    sources = sorted(resources)
    missing = [
        p for p in sources if not os.path.exists(os.path.join(CONFIG_DIR, p))
    ]

    if missing:
        print(f"missing ({platform}): " + ", ".join(missing))
        for p in sources:
            exists = os.path.exists(os.path.join(CONFIG_DIR, p))
            print(f"  {'OK     ' if exists else 'ABSENT '} {p}")
        return 1

    print(f"all {len(sources)} resources present ({platform} names)")
    for p in sources:
        print(f"  {p} -> {resources[p]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())