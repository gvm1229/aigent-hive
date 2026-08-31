#!/usr/bin/env python3
"""Send the stable-release banner before its Korean subscriber update."""

from __future__ import annotations

import argparse
import json
import mimetypes
import os
import re
import secrets
import sys
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import ProxyHandler, Request, build_opener


STABLE_VERSION = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
DISCORD_HOSTS = frozenset({"discord.com", "discordapp.com"})
LOCAL_TEST_HOSTS = frozenset({"127.0.0.1", "localhost", "::1"})


class NotificationError(ValueError):
    """A safe-to-report stable-release notification error."""


def fail(message: str) -> None:
    raise NotificationError(message)


def read_summary(path: Path, product_version: str) -> str:
    try:
        contents = path.read_text(encoding="utf-8").strip()
    except OSError as error:
        fail(f"cannot read subscriber summary: {error.filename or path.name}")
    expected_title = f"# Aigent Hive v{product_version} 업데이트 내역:"
    lines = contents.splitlines()
    if not lines or lines[0] != expected_title:
        fail("subscriber summary title does not match the stable product version")
    bullets = [line for line in lines[1:] if line.strip()]
    if (
        not bullets
        or not bullets[0].startswith("- ")
        or any(re.fullmatch(r"(?:- |  - )\S.*", line) is None for line in bullets)
    ):
        fail("subscriber summary requires main Markdown bullets with optional two-space child bullets")
    if len(contents) > 2_000:
        fail("subscriber summary exceeds the Discord message limit")
    return contents


def validate_webhook_url(value: str, allow_insecure_test_webhook: bool) -> str:
    if not value or value == "null":
        fail("required Discord webhook secret is unavailable")
    parsed = urlsplit(value)
    if parsed.username or parsed.password or parsed.query or parsed.fragment:
        fail("Discord webhook secret has an invalid URL form")
    parts = parsed.path.split("/")
    if parts[:3] != ["", "api", "webhooks"] or len(parts) != 5 or not all(parts[3:]):
        fail("Discord webhook secret has an invalid webhook path")
    if parsed.scheme == "https" and parsed.hostname in DISCORD_HOSTS:
        return value
    if (
        allow_insecure_test_webhook
        and parsed.scheme == "http"
        and parsed.hostname in LOCAL_TEST_HOSTS
    ):
        return value
    fail("Discord webhook secret must use an approved HTTPS Discord endpoint")
    raise AssertionError("unreachable")


def post(url: str, payload: bytes, content_type: str, label: str) -> None:
    request = Request(
        url,
        data=payload,
        headers={
            "Content-Type": content_type,
            "User-Agent": "Aigent-Hive-stable-release-notifier/1",
        },
        method="POST",
    )
    opener = build_opener(ProxyHandler({}))
    try:
        with opener.open(request, timeout=15) as response:
            if not 200 <= response.status < 300:
                fail(f"{label} request returned HTTP {response.status}")
    except HTTPError as error:
        fail(f"{label} request returned HTTP {error.code}")
    except (URLError, OSError):
        fail(f"{label} request failed")


def banner_payload(path: Path) -> tuple[bytes, str]:
    try:
        image = path.read_bytes()
    except OSError as error:
        fail(f"cannot read release banner: {error.filename or path.name}")
    if not image:
        fail("release banner is empty")
    boundary = f"----AigentHive{secrets.token_hex(16)}"
    content_type = mimetypes.guess_type(path.name)[0] or "application/octet-stream"
    payload = bytearray()
    payload.extend(f"--{boundary}\r\n".encode())
    payload.extend(b'Content-Disposition: form-data; name="payload_json"\r\n')
    payload.extend(b"Content-Type: application/json\r\n\r\n")
    payload.extend(b'{"allowed_mentions":{"parse":[]}}\r\n')
    payload.extend(f"--{boundary}\r\n".encode())
    payload.extend(
        f'Content-Disposition: form-data; name="files[0]"; filename="{path.name}"\r\n'.encode()
    )
    payload.extend(f"Content-Type: {content_type}\r\n\r\n".encode())
    payload.extend(image)
    payload.extend(f"\r\n--{boundary}--\r\n".encode())
    return bytes(payload), f"multipart/form-data; boundary={boundary}"


def prepare_delivery(
    *, product_version: str, summary_path: Path, banner_path: Path
) -> tuple[str, bytes, str]:
    if STABLE_VERSION.fullmatch(product_version) is None:
        fail("Discord subscriber updates are allowed only for stable X.Y.Z versions")
    summary = read_summary(summary_path, product_version)
    banner, banner_content_type = banner_payload(banner_path)
    return summary, banner, banner_content_type


def send(
    *, summary: str, banner: bytes, banner_content_type: str, webhook_url: str
) -> None:
    post(webhook_url, banner, banner_content_type, "release banner")
    message = json.dumps(
        {"content": summary, "allowed_mentions": {"parse": []}}, ensure_ascii=False
    ).encode("utf-8")
    post(webhook_url, message, "application/json", "subscriber summary")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product-version", required=True)
    parser.add_argument("--summary", required=True, type=Path)
    parser.add_argument("--banner", required=True, type=Path)
    parser.add_argument(
        "--webhook-url-env",
        default="AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL",
    )
    parser.add_argument("--allow-insecure-test-webhook", action="store_true")
    parser.add_argument("--validate-only", action="store_true")
    arguments = parser.parse_args()
    try:
        summary, banner, banner_content_type = prepare_delivery(
            product_version=arguments.product_version,
            summary_path=arguments.summary,
            banner_path=arguments.banner,
        )
        webhook_url = validate_webhook_url(
            os.environ.get(arguments.webhook_url_env, ""),
            arguments.allow_insecure_test_webhook,
        )
        if arguments.validate_only:
            return 0
        send(
            summary=summary,
            banner=banner,
            banner_content_type=banner_content_type,
            webhook_url=webhook_url,
        )
    except NotificationError as error:
        print(f"stable Discord notification error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
