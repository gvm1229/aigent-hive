#!/usr/bin/env python3
"""Build the self-contained marketing page.

Reads website/src/index.html, embeds the canonical logo PNG as a data URI in place of
{{HIVE_LOGO}}, and writes website/dist/index.html. The only remaining network reference is the
Pretendard web-font stylesheet; offline, the page falls back to system fonts.
"""

from __future__ import annotations

import base64
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "website" / "src" / "index.html"
LOGO = ROOT / "docs" / "assets" / "branding" / "hive-logo-mark.png"
OUTPUT = ROOT / "website" / "dist" / "index.html"
PLACEHOLDER = "{{HIVE_LOGO}}"


def main() -> int:
    template = SOURCE.read_text(encoding="utf-8")
    if PLACEHOLDER not in template:
        print(f"error: {SOURCE} has no {PLACEHOLDER}", file=sys.stderr)
        return 1
    logo = LOGO.read_bytes()
    data_uri = "data:image/png;base64," + base64.b64encode(logo).decode("ascii")
    page = template.replace(PLACEHOLDER, data_uri)
    if "{{" in page:
        print("error: unresolved placeholder remains", file=sys.stderr)
        return 1
    embedded = base64.b64decode(data_uri.split(",", 1)[1])
    if hashlib.sha256(embedded).digest() != hashlib.sha256(logo).digest():
        print("error: embedded logo differs from source bytes", file=sys.stderr)
        return 1
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(page, encoding="utf-8")
    print(f"wrote {OUTPUT.relative_to(ROOT)} ({len(page.encode('utf-8')):,} bytes)")
    print(f"logo sha256 {hashlib.sha256(logo).hexdigest()}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
