from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[3]
NOTIFIER = ROOT / "scripts/publish-stable-discord-update.py"
WORKFLOW = ROOT / ".github/workflows/release-publish.yml"


class RecordingServer(ThreadingHTTPServer):
    responses: list[int]
    requests: list[tuple[str, bytes]]


class RecordingHandler(BaseHTTPRequestHandler):
    def do_POST(self) -> None:  # noqa: N802
        body = self.rfile.read(int(self.headers["Content-Length"]))
        self.server.requests.append((self.headers["Content-Type"], body))  # type: ignore[attr-defined]
        status = self.server.responses.pop(0) if self.server.responses else 204  # type: ignore[attr-defined]
        self.send_response(status)
        self.end_headers()

    def log_message(self, _format: str, *_args: object) -> None:
        return


class StableReleaseDiscordNotification(unittest.TestCase):
    def setUp(self) -> None:
        self.server = RecordingServer(("127.0.0.1", 0), RecordingHandler)
        self.server.responses = []
        self.server.requests = []
        self.thread = threading.Thread(target=self.server.serve_forever)
        self.thread.start()

    def tearDown(self) -> None:
        self.server.shutdown()
        self.thread.join()
        self.server.server_close()

    def run_notifier(
        self,
        version: str = "0.9.5",
        summary_text: str | None = None,
        validate_only: bool = False,
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            summary = root / "summary.md"
            summary.write_text(
                summary_text
                or "# Aigent Hive v0.9.5 업데이트 내역:\n\n- 설치 확인 개선\n- 지식 보호 강화\n",
                encoding="utf-8",
            )
            banner = root / "hive-readme-banner-ko.png"
            banner.write_bytes(b"\x89PNG\r\n\x1a\nfixture")
            environment = os.environ | {
                "AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL": (
                    f"http://127.0.0.1:{self.server.server_port}/api/webhooks/test/token"
                )
            }
            command = [
                sys.executable,
                str(NOTIFIER),
                "--product-version",
                version,
                "--summary",
                str(summary),
                "--banner",
                str(banner),
                "--allow-insecure-test-webhook",
            ]
            if validate_only:
                command.append("--validate-only")
            return subprocess.run(
                command,
                cwd=ROOT,
                check=False,
                capture_output=True,
                text=True,
                env=environment,
            )

    def test_sends_banner_before_the_korean_subscriber_summary(self) -> None:
        result = self.run_notifier()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.server.requests), 2)
        banner_type, banner = self.server.requests[0]
        self.assertTrue(banner_type.startswith("multipart/form-data; boundary="))
        self.assertIn(b'hive-readme-banner-ko.png', banner)
        self.assertIn(b"\x89PNG\r\n\x1a\nfixture", banner)
        message_type, message = self.server.requests[1]
        self.assertEqual(message_type, "application/json")
        self.assertEqual(
            json.loads(message.decode("utf-8"))["content"],
            "# Aigent Hive v0.9.5 업데이트 내역:\n\n- 설치 확인 개선\n- 지식 보호 강화",
        )

    def test_banner_failure_blocks_the_subscriber_summary(self) -> None:
        self.server.responses = [500]
        result = self.run_notifier()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("release banner request returned HTTP 500", result.stderr)
        self.assertNotIn("http://", result.stderr)
        self.assertNotIn("/api/webhooks/", result.stderr)
        self.assertEqual(len(self.server.requests), 1)

    def test_nested_examples_reach_the_local_receiver_without_reformatting(self) -> None:
        summary = (
            "# Aigent Hive v0.10.0 업데이트 내역:\n\n"
            "- **새 기능 추가**\n"
            "  - 사용 예시와 선택 사항 안내\n"
            "  - 원문·출처 보존\n\n"
            "- **기존 기능 개선**\n"
            "  - 운영체제별 사용 예시\n"
        )
        result = self.run_notifier("0.10.0", summary)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.server.requests), 2)
        self.assertEqual(json.loads(self.server.requests[1][1])["content"], summary.strip())

    def test_malformed_lists_are_rejected_before_any_request(self) -> None:
        for body in (
            "  - parent missing\n",
            "- main\n - wrong indent\n",
            "- main\n    - deeper nesting\n",
            "- main\n  - \n",
            "- main\nparagraph outside list\n",
        ):
            with self.subTest(body=body):
                result = self.run_notifier(summary_text="# Aigent Hive v0.9.5 업데이트 내역:\n\n" + body)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.server.requests, [])

    def test_nested_lists_keep_the_discord_length_limit(self) -> None:
        prefix = "# Aigent Hive v0.9.5 업데이트 내역:\n\n- main\n  - "
        summary = prefix + "가" * (2_000 - len(prefix))
        accepted = self.run_notifier(summary_text=summary, validate_only=True)
        self.assertEqual(accepted.returncode, 0, accepted.stderr)
        rejected = self.run_notifier(summary_text=summary + "가", validate_only=True)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("Discord message limit", rejected.stderr)
        self.assertEqual(self.server.requests, [])

    def test_rejects_nonstable_versions_before_sending(self) -> None:
        result = self.run_notifier("0.9.5-test.1")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("allowed only for stable", result.stderr)
        self.assertEqual(self.server.requests, [])

    def test_validation_sends_no_webhook_request(self) -> None:
        result = self.run_notifier(validate_only=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.server.requests, [])

    def test_accepts_the_v094_update_summary_payload(self) -> None:
        summary = (ROOT / "docs/releases/0.9.4.subscriber.ko.md").read_text(encoding="utf-8")
        result = self.run_notifier("0.9.4", summary)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.server.requests), 2)

    def test_accepts_the_v093_subscriber_payload(self) -> None:
        summary = (ROOT / "docs/releases/0.9.3.subscriber.ko.md").read_text(encoding="utf-8")
        result = self.run_notifier("0.9.3", summary)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.server.requests), 2)

    def test_publication_workflow_invokes_the_notifier_only_after_release_creation(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL", workflow)
        self.assertIn("if: ${{ inputs.channel == 'stable' }}", workflow)
        self.assertIn("--validate-only", workflow)
        self.assertIn("scripts/publish-stable-discord-update.py", workflow)
        self.assertLess(workflow.index("--validate-only"), workflow.index("npm publish"))
        self.assertLess(workflow.index("gh release create"), workflow.index("Send stable Discord"))
        steps = yaml.safe_load(workflow)["jobs"]["publish"]["steps"]
        release = next(step for step in steps if step.get("name") == "Create annotated channel tag and GitHub Release")
        notifier = next(step for step in steps if step.get("name") == "Send stable Discord banner and subscriber update")
        self.assertIn("gh release create", release["run"])
        self.assertNotIn("publish-stable-discord-update.py", release["run"])
        self.assertEqual(notifier["if"], "${{ inputs.channel == 'stable' }}")
