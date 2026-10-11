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

    The override replaces the map rather than merging into it — json_patch's
    object merge is per-key, but a resource map is a whole unit here: listing
    the Linux names on Windows would put a file in the package that does not
    exist and miss the one that does.
    """
    with open(os.path.join(CONFIG_DIR, "tauri.conf.json")) as fh:
        config = json.load(fh)
    resources = dict(config["bundle"]["resources"])
    override = OVERRIDES.get(platform)
    if override:
        path = os.path.join(CONFIG_DIR, override)
        if os.path.exists(path):
            with open(path) as fh:
                resources = dict(json.load(fh)["bundle"]["resources"])
    return resources


def main() -> int:
    platform = sys.argv[1] if len(sys.argv) > 1 else "linux"
    if platform not in OVERRIDES:
        print(f"unknown platform '{platform}'", file=sys.stderr)
        return 2

    resources = resources_for(platform)
    missing = [
        p for p in resources if not os.path.exists(os.path.join(CONFIG_DIR, p))
    ]

    if missing:
        print(f"missing ({platform}): " + ", ".join(sorted(missing)))
        for p in sorted(resources):
            exists = os.path.exists(os.path.join(CONFIG_DIR, p))
            print(f"  {'OK     ' if exists else 'ABSENT '} {p}")
        return 1

    print(f"all {len(resources)} resources present ({platform} names)")
    return 0


if __name__ == "__main__":
    sys.exit(main())