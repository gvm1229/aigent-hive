#!/usr/bin/env python3
"""Send the stable-release banner before its Korean subscriber update."""

from __future__ import annotations

import argparse
import hashlib
import json
import mimetypes
import os
import re
import secrets
import sys
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener


STABLE_VERSION = re.compile(r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
SUMMARY_DIGEST = re.compile(r"^sha256:[0-9a-f]{64}$")
DISCORD_HOSTS = frozenset({"discord.com", "discordapp.com"})
LOCAL_TEST_HOSTS = frozenset({"127.0.0.1", "localhost", "::1"})
SUMMARY_SECTIONS = ("## 새 기능", "## 수정", "## 개선")


class NotificationError(ValueError):
    """A safe-to-report stable-release notification error."""


def fail(message: str) -> None:
    raise NotificationError(message)


def approval_digest(path: Path, summary_path: Path) -> str:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        fail(f"cannot read subscriber summary approval: {error.filename or path.name}")
    if len(lines) != 1:
        fail("subscriber summary approval must contain one digest line")
    digest, separator, filename = lines[0].partition("  ")
    if (
        separator != "  "
        or SUMMARY_DIGEST.fullmatch(digest) is None
        or filename != summary_path.name
    ):
        fail("subscriber summary approval does not bind the selected summary file")
    return digest


def read_summary(
    path: Path,
    product_version: str,
    approval_path: Path,
) -> str:
    try:
        raw_contents = path.read_bytes()
    except OSError as error:
        fail(f"cannot read subscriber summary: {error.filename or path.name}")
    approved_digest = approval_digest(approval_path, path)
    actual_digest = "sha256:" + hashlib.sha256(raw_contents).hexdigest()
    if actual_digest != approved_digest:
        fail("subscriber summary differs from its approved digest")
    try:
        contents = raw_contents.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n").strip()
    except UnicodeDecodeError:
        fail("subscriber summary must be UTF-8")
    expected_title = f"# Aigent Hive v{product_version} 업데이트 내역:"
    lines = contents.splitlines()
    if not lines or lines[0] != expected_title:
        fail("subscriber summary title does not match the stable product version")
    body = [line for line in lines[1:] if line.strip()]
    if body and body[0].startswith("## "):
        sections: list[list[str]] = [[]]
        for line in body:
            if line == "---":
                sections.append([])
            else:
                sections[-1].append(line)
        previous_rank = -1
        for section in sections:
            if not section or section[0] not in SUMMARY_SECTIONS:
                fail("subscriber summary has an unknown or empty section")
            rank = SUMMARY_SECTIONS.index(section[0])
            if rank <= previous_rank:
                fail("subscriber summary sections must be unique and ordered")
            if not valid_summary_bullets(section[1:]):
                fail("each subscriber section requires main bullets and optional child bullets")
            previous_rank = rank
    elif not valid_summary_bullets(body):
        fail("subscriber summary requires main Markdown bullets with optional two-space child bullets")
    if len(contents) > 2_000:
        fail("subscriber summary exceeds the Discord message limit")
    return contents


def valid_summary_bullets(lines: list[str]) -> bool:
    return bool(lines) and lines[0].startswith("- ") and all(
        re.fullmatch(r"(?:- |  - )\S.*", line) is not None for line in lines
    )


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
    *,
    product_version: str,
    summary_path: Path,
    summary_approval_path: Path,
    approval: str,
    banner_path: Path,
) -> tuple[str, bytes, str]:
    if STABLE_VERSION.fullmatch(product_version) is None:
        fail("Discord subscriber updates are allowed only for stable X.Y.Z versions")
    if approval != "main-pr":
        fail("subscriber notification is disabled: approval must be main-pr")
    summary = read_summary(
        summary_path,
        product_version,
        summary_approval_path,
    )
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



class NoWebhookRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def webhook_request(url: str, method: str, payload: dict | None = None) -> tuple[int, dict]:
    """Bounded response; neither error bodies nor secret URLs enter receipts/logs."""
    data = None if payload is None else json.dumps(payload, ensure_ascii=False).encode("utf-8")
    request = Request(url, data=data, method=method, headers={
        "Content-Type": "application/json", "User-Agent": "Aigent-Hive-stable-release-notifier/1"})
    try:
        with build_opener(ProxyHandler({}), NoWebhookRedirect()).open(request, timeout=15) as response:
            raw = response.read(1024 * 1024 + 1)
            if len(raw) > 1024 * 1024:
                fail("Discord response exceeds the response limit; inspect delivery before retrying")
            try:
                value = json.loads(raw) if raw else {}
            except (ValueError, UnicodeDecodeError):
                fail("Discord response is invalid; inspect delivery before retrying")
            if not isinstance(value, dict):
                fail("Discord response is invalid; inspect delivery before retrying")
            return response.status, value
    except HTTPError as error:
        return error.code, {}
    except (URLError, OSError):
        fail("Discord correction outcome is unknown; inspect delivery before retrying")
    raise AssertionError("unreachable")


def validate_correction(summary: str, product_version: str, message_id: str) -> None:
    if f"{product_version} 안내 정정" not in summary:
        fail("replacement notice must clearly identify the old version's correction")
    if message_id and re.fullmatch(r"[1-9][0-9]{16,21}", message_id) is None:
        fail("Discord message ID must be a numeric message snowflake")


def correct_notice(*, summary: str, product_version: str, message_id: str, webhook_url: str) -> dict:
    validate_correction(summary, product_version, message_id)
    title = f"# Aigent Hive v{product_version} 업데이트 내역:"
    payload = {"content": summary, "allowed_mentions": {"parse": []}}
    old_deleted = False
    attempts = []
    if message_id:
        endpoint = f"{webhook_url}/messages/{message_id}"
        status, original = webhook_request(endpoint, "GET")
        attempts.append({"method": "GET", "status": status})
        if status == 200:
            content = original.get("content")
            if original.get("id") != message_id or not isinstance(content, str) or not content.startswith(title):
                fail("message does not match the selected release notice; no changes made")
            if original.get("content") == summary:
                return {"operation": "already-corrected", "message_id": message_id,
                        "manual_delete_required": False, "attempts": attempts}
            status, edited = webhook_request(endpoint, "PATCH", payload)
            attempts.append({"method": "PATCH", "status": status})
            if status == 200:
                if edited.get("id") != message_id or edited.get("content") != summary:
                    fail("Discord edit could not be verified; inspect delivery before retrying")
                return {"operation": "edited", "message_id": message_id,
                        "manual_delete_required": False, "attempts": attempts}
            if status not in (403, 404, 405):
                fail(f"Discord edit returned HTTP {status}; no automatic replacement attempted")
            status, _ = webhook_request(endpoint, "DELETE")
            attempts.append({"method": "DELETE", "status": status})
            if status == 204:
                old_deleted = True
            elif status not in (403, 404, 405):
                fail(f"Discord deletion returned HTTP {status}; inspect the old notice before retrying")
        elif status not in (403, 404):
            fail(f"Discord message lookup returned HTTP {status}; no automatic replacement attempted")
    status, posted = webhook_request(f"{webhook_url}?wait=true", "POST", payload)
    attempts.append({"method": "POST", "status": status})
    if status != 200 or re.fullmatch(r"[1-9][0-9]{16,21}", str(posted.get("id", ""))) is None:
        fail(f"Discord correction POST is unverified (HTTP {status}); inspect delivery before retrying")
    if posted.get("content") != summary:
        fail("Discord correction content is unverified; inspect delivery before retrying")
    return {"operation": "replaced" if old_deleted else "posted-correction",
            "message_id": posted["id"], "manual_delete_required": not old_deleted, "attempts": attempts}

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--product-version", required=True)
    parser.add_argument("--summary", required=True, type=Path)
    parser.add_argument("--summary-approval", required=True, type=Path)
    parser.add_argument(
        "--approval-env",
        default="AIGENT_HIVE_SUBSCRIBER_APPROVAL",
    )
    parser.add_argument("--banner", required=True, type=Path)
    parser.add_argument(
        "--webhook-url-env",
        default="AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL",
    )
    parser.add_argument("--allow-insecure-test-webhook", action="store_true")
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--correction", action="store_true")
    parser.add_argument("--message-id", default="")
    parser.add_argument("--receipt", type=Path)
    arguments = parser.parse_args()
    try:
        summary, banner, banner_content_type = prepare_delivery(
            product_version=arguments.product_version,
            summary_path=arguments.summary,
            summary_approval_path=arguments.summary_approval,
            approval=os.environ.get(arguments.approval_env, ""),
            banner_path=arguments.banner,
        )
        webhook_url = validate_webhook_url(
            os.environ.get(arguments.webhook_url_env, ""),
            arguments.allow_insecure_test_webhook,
        )
        if arguments.correction:
            validate_correction(summary, arguments.product_version, arguments.message_id)
            if arguments.receipt is None:
                fail("a correction requires a receipt path")
        elif arguments.message_id or arguments.receipt is not None:
            fail("message ID and receipt path require correction mode")
        if arguments.validate_only:
            return 0
        if arguments.correction:
            receipt = correct_notice(summary=summary, product_version=arguments.product_version,
                                     message_id=arguments.message_id, webhook_url=webhook_url)
            receipt.update({"schema_version": 1, "status": "accepted", "product_version": arguments.product_version,
                            "summary_sha256": approval_digest(arguments.summary_approval, arguments.summary),
                            "webhook_recorded": False, "recipient_reading_verified": False,
                            "original_message_id": arguments.message_id or None,
                            "manual_action": ("Maintainer must delete the old notice manually."
                                              if receipt["manual_delete_required"] else None)})
            try:
                arguments.receipt.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
            except OSError:
                fail("Discord accepted the correction but its receipt could not be saved; do not resend blindly")
            print(json.dumps(receipt))
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
