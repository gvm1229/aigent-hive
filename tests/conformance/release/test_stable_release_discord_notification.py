from __future__ import annotations

import hashlib
import json
import os
import shutil
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
    def correction(self) -> None:
        body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
        payload = json.loads(body) if body else None
        self.server.correction_requests.append((self.command, self.path, payload))
        status, response = self.server.correction_responses.pop(0)
        response = dict(response)
        if response.pop("echo_content", False):
            response["content"] = payload["content"]
        self.send_response(status)
        if status == 302:
            self.send_header("Location", "/redirected")
        self.end_headers()
        if status != 204:
            self.wfile.write(json.dumps(response).encode())

    def do_GET(self) -> None:  # noqa: N802
        self.correction()

    def do_PATCH(self) -> None:  # noqa: N802
        self.correction()

    def do_DELETE(self) -> None:  # noqa: N802
        self.correction()

    def do_POST(self) -> None:  # noqa: N802
        if getattr(self.server, "correction_mode", False):
            self.correction()
            return
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
        approval_text: str | None = None,
        approval_value: str | None = "main-pr",
        validate_only: bool = False,
        correction: bool = False,
        message_id: str = "",
    ) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            summary = root / "summary.md"
            summary.write_text(
                summary_text
                or "# Aigent Hive v0.9.5 업데이트 내역:\n\n- 설치 확인 개선\n- 지식 보호 강화\n",
                encoding="utf-8",
            )
            approval = root / "summary.md.sha256"
            approval.write_text(
                approval_text
                or f"sha256:{hashlib.sha256(summary.read_bytes()).hexdigest()}  summary.md\n",
                encoding="utf-8",
            )
            banner = root / "hive-readme-banner-ko.png"
            banner.write_bytes(b"\x89PNG\r\n\x1a\nfixture")
            environment = os.environ | {
                "AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL": (
                    f"http://127.0.0.1:{self.server.server_port}/api/webhooks/test/token"
                ),
            }
            environment.pop("AIGENT_HIVE_SUBSCRIBER_APPROVAL", None)
            if approval_value is not None:
                environment["AIGENT_HIVE_SUBSCRIBER_APPROVAL"] = approval_value
            command = [
                sys.executable,
                str(NOTIFIER),
                "--product-version",
                version,
                "--summary",
                str(summary),
                "--summary-approval",
                str(approval),
                "--banner",
                str(banner),
                "--allow-insecure-test-webhook",
            ]
            if correction:
                command.extend(["--correction", "--message-id", message_id, "--receipt", str(root / "receipt.json")])
            if validate_only:
                command.append("--validate-only")
            result = subprocess.run(
                command,
                cwd=ROOT,
                check=False,
                capture_output=True,
                text=True,
                env=environment,
            )
            receipt = root / "receipt.json"
            self.last_receipt = json.loads(receipt.read_text(encoding="utf-8")) if receipt.is_file() else None
            return result

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

    def test_tampered_approved_summary_blocks_before_the_banner(self) -> None:
        result = self.run_notifier(
            summary_text="# Aigent Hive v0.9.5 업데이트 내역:\n\n- 승인 문구 변경\n",
            approval_text="sha256:" + "0" * 64 + "  summary.md\n",
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("approved digest", result.stderr)
        self.assertEqual(self.server.requests, [])

    def test_missing_or_disabled_approval_blocks_before_the_banner(self) -> None:
        for value in (None, "", "disabled", "MAIN-PR", " main-pr", "main-pr ", "sha256:" + "0" * 64):
            with self.subTest(value=value):
                result = self.run_notifier(approval_value=value)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("notification is disabled", result.stderr)
                self.assertEqual(self.server.requests, [])

    def test_multiple_digest_lines_block_before_the_banner(self) -> None:
        result = self.run_notifier(approval_text="first\nsecond\n")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("one digest line", result.stderr)
        self.assertEqual(self.server.requests, [])

    def test_approval_must_bind_the_selected_summary_filename(self) -> None:
        result = self.run_notifier(
            approval_text="sha256:" + "0" * 64 + "  another-summary.md\n",
            validate_only=True,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("approval does not bind", result.stderr)
        self.assertEqual(self.server.requests, [])

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

    def test_change_sections_and_separators_reach_receiver_without_reformatting(self) -> None:
        summary = (
            "# Aigent Hive v0.12.0 업데이트 내역:\n\n"
            "## 새 기능\n\n- 새 보호 기능\n  - 적용 범위 안내\n\n"
            "---\n\n## 수정\n\n- 잘못된 버전 구분 수정\n\n"
            "---\n\n## 개선\n\n- 오류 안내 개선\n  - 기존 자료 보존\n"
        )
        result = self.run_notifier("0.12.0", summary)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(self.server.requests), 2)
        self.assertEqual(json.loads(self.server.requests[1][1])["content"], summary.strip())

    def test_sectioned_summary_omits_categories_without_changes(self) -> None:
        for body in (
            "## 개선\n\n- 안내 개선\n",
            "## 새 기능\n\n- 기능 추가\n\n---\n\n## 개선\n\n- 안내 개선\n",
        ):
            with self.subTest(body=body):
                result = self.run_notifier("0.12.0", "# Aigent Hive v0.12.0 업데이트 내역:\n\n" + body, validate_only=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(self.server.requests, [])

    def test_malformed_sections_are_rejected_before_any_request(self) -> None:
        for body in (
            "## 수정\n- 수정\n---\n## 새 기능\n- 추가",
            "## 새 기능\n- 추가\n---\n## 새 기능\n- 중복",
            "## 새 기능\n- 추가\n## 개선\n- 누락된 구분선",
            "## 새 기능\n---\n## 수정\n- 비어 있는 앞 섹션",
            "## 새 기능\n- 추가\n---",
            "## 새 기능\n- 추가\n---\n---\n## 수정\n- 수정",
            "## 변경\n- 알 수 없는 범주",
            "## 개선\n  - 부모 없는 하위 목록",
            "## 개선\n- 안내\n목록 밖 문장",
        ):
            with self.subTest(body=body):
                result = self.run_notifier("0.12.0", "# Aigent Hive v0.12.0 업데이트 내역:\n\n" + body)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.server.requests, [])

    def test_section_headings_and_separators_count_toward_message_limit(self) -> None:
        prefix = "# Aigent Hive v0.12.0 업데이트 내역:\n\n## 새 기능\n- 추가\n\n---\n\n## 개선\n- "
        summary = prefix + "가" * (2_000 - len(prefix))
        accepted = self.run_notifier("0.12.0", summary, validate_only=True)
        self.assertEqual(accepted.returncode, 0, accepted.stderr)
        rejected = self.run_notifier("0.12.0", summary + "가", validate_only=True)
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

    def test_approved_v010_payload_is_bound_to_its_checked_in_digest(self) -> None:
        summary = ROOT / "docs/releases/0.10.0.subscriber.ko.md"
        approval = ROOT / "docs/releases/0.10.0.subscriber.ko.sha256"
        expected = (
            "sha256:ce658d7a5addabc93d69c99d3bea80fd0137c61d3141c9880c05fa1e50d4e426"
            "  0.10.0.subscriber.ko.md\n"
        )
        self.assertEqual(approval.read_text(encoding="utf-8"), expected)
        namespace = __import__("runpy").run_path(str(NOTIFIER))
        self.assertEqual(
            namespace["read_summary"](summary, "0.10.0", approval),
            summary.read_text(encoding="utf-8").strip(),
        )

    def test_publication_workflow_invokes_the_notifier_only_after_release_creation(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL", workflow)
        self.assertIn("if: ${{ inputs.channel == 'stable' }}", workflow)
        self.assertIn("--validate-only", workflow)
        self.assertIn("--summary-approval", workflow)
        self.assertIn("AIGENT_HIVE_SUBSCRIBER_APPROVAL", workflow)
        self.assertIn("scripts/publish-stable-discord-update.py", workflow)
        self.assertLess(workflow.index("--validate-only"), workflow.index("npm publish"))
        self.assertLess(workflow.index("gh release create"), workflow.index("Send stable Discord"))
        steps = yaml.safe_load(workflow)["jobs"]["publish"]["steps"]
        release = next(step for step in steps if step.get("name") == "Create annotated channel tag and GitHub Release")
        notifier = next(step for step in steps if step.get("name") == "Send stable Discord banner and subscriber update")
        self.assertIn("gh release create", release["run"])
        self.assertNotIn("publish-stable-discord-update.py", release["run"])
        self.assertEqual(
            notifier["if"],
            "${{ inputs.channel == 'stable' && inputs.send_subscriber_update == true }}",
        )

    def test_publication_requires_explicit_notification_opt_in(self) -> None:
        workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
        # PyYAML's YAML 1.1 parser treats the GitHub Actions `on` key as True.
        events = workflow.get("on", workflow.get(True))
        notification = events["workflow_dispatch"]["inputs"]["send_subscriber_update"]
        self.assertEqual(notification["type"], "boolean")
        self.assertIs(notification["default"], False)
        self.assertIs(notification["required"], False)
        steps = workflow["jobs"]["publish"]["steps"]
        notifier = next(step for step in steps if step.get("name") == "Send stable Discord banner and subscriber update")
        self.assertEqual(
            notifier["if"],
            "${{ inputs.channel == 'stable' && inputs.send_subscriber_update == true }}",
        )
        release = next(step for step in steps if step.get("name") == "Create annotated channel tag and GitHub Release")
        self.assertNotIn("send_subscriber_update", release.get("if", ""))

    def test_stable_publication_rejects_missing_or_disabled_notification_before_publish(self) -> None:
        workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
        job = workflow["jobs"]["publish"]
        contract = job["steps"][0]["run"]
        if os.name == "nt":
            git = shutil.which("git")
            shell = Path(git).parent.parent / "bin/bash.exe" if git else None
            if shell is None or not shell.is_file():
                self.skipTest("Git Bash is required for the publication shell contract")
        else:
            shell = shutil.which("bash")
            if not shell:
                self.skipTest("Bash is required for the publication shell contract")
        for channel, selected, allowed in (
            ("stable", "", False), ("stable", "false", False),
            ("stable", "true", True), ("test", "false", True),
        ):
            with self.subTest(channel=channel, selected=selected):
                environment = dict(os.environ, PRODUCT_VERSION="0.12.0",
                    PACKAGE_VERSION="0.12.0" if channel == "stable" else "0.12.0-test.3",
                    RUN_ID="1", CHANNEL=channel, SEND_SUBSCRIBER_UPDATE=selected,
                    GITHUB_OUTPUT="/dev/null")
                result = subprocess.run([str(shell), "-e", "-c", contract],
                    env=environment, capture_output=True, text=True, check=False, timeout=10)
                self.assertEqual(result.returncode == 0, allowed)
        steps = job["steps"]
        approval = next(step for step in steps if step.get("name") == "Validate stable Discord subscriber update")
        self.assertEqual(approval["if"], "${{ inputs.channel == 'stable' }}")
        self.assertIn("AIGENT_HIVE_SUBSCRIBER_APPROVAL", approval["env"])
        self.assertLess(steps.index(approval), next(i for i, step in enumerate(steps)
            if step.get("name") == "Publish npm package family through trusted publishing"))


class StableNotificationRecoveryWorkflow(unittest.TestCase):
    def test_recovery_binds_main_published_stable_and_successfully_skipped_send(self) -> None:
        path = ROOT / ".github/workflows/release-discord-notification.yml"
        workflow = yaml.safe_load(path.read_text(encoding="utf-8"))
        job = workflow["jobs"]["notify"]
        self.assertEqual(job["environment"]["name"], "release-publication")
        self.assertIs(workflow["concurrency"]["cancel-in-progress"], False)
        contract = job["steps"][0]["run"]
        for boundary in (
            'test "$GITHUB_REF" = refs/heads/main',
            'test "$(jq -r .isPrerelease <<<"$release")" = false',
            'test "$(jq -r .conclusion <<<"$publication")" = success',
            'test "$(jq -r .head_sha <<<"$publication")" = "$source_sha"',
            '.conclusion == "skipped"',
            'duplicate send refused',
        ):
            self.assertIn(boundary, contract)
        checkout = job["steps"][1]
        self.assertEqual(checkout["with"]["ref"], "${{ steps.contract.outputs.summary_source_sha }}")

    def test_recovery_always_uses_released_source_and_version_scoped_receipt(self) -> None:
        workflow = yaml.safe_load((ROOT / ".github/workflows/release-discord-notification.yml").read_text(encoding="utf-8"))
        events = workflow.get("on", workflow.get(True))
        self.assertEqual(set(events["workflow_dispatch"]["inputs"]), {"product_version", "publication_run_id", "operation", "message_id"})
        steps = workflow["jobs"]["notify"]["steps"]
        contract = steps[0]["run"]
        self.assertIn('summary_source_sha=$source_sha', contract)
        self.assertNotIn('summary_source_sha=$GITHUB_SHA', contract)
        self.assertIn('receipt_name="stable-discord-notification-$PRODUCT_VERSION"', contract)
        self.assertIn('test "$skipped" = 1', contract)
        self.assertEqual(steps[-1]["with"]["name"], "${{ steps.contract.outputs.receipt_name }}")
        self.assertIn('test "$(git rev-parse HEAD)" = "$SUMMARY_SOURCE_SHA"', steps[2]["run"])

    def test_both_workflows_use_environment_variable_for_validation_and_send(self) -> None:
        for path, job_name in ((WORKFLOW, "publish"), (ROOT / ".github/workflows/release-discord-notification.yml", "notify")):
            with self.subTest(path=path.name):
                job = yaml.safe_load(path.read_text(encoding="utf-8"))["jobs"][job_name]
                self.assertEqual(job["environment"]["name"], "release-publication")
                notification_steps = [step for step in job["steps"] if "scripts/publish-stable-discord-update.py" in step.get("run", "")]
                self.assertEqual(len(notification_steps), 2)
                for step in notification_steps:
                    self.assertEqual(step["env"]["AIGENT_HIVE_SUBSCRIBER_APPROVAL"], "${{ vars.AIGENT_HIVE_SUBSCRIBER_APPROVAL }}")
                    self.assertEqual(step["env"]["AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL"], "${{ secrets.AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL }}")

    def test_recovery_validates_before_sending_and_never_republishes(self) -> None:
        workflow = yaml.safe_load((ROOT / ".github/workflows/release-discord-notification.yml").read_text(encoding="utf-8"))
        steps = workflow["jobs"]["notify"]["steps"]
        validation = steps[2]["run"]
        delivery = steps[3]["run"]
        self.assertIn("--validate-only", validation)
        self.assertIn("--summary-approval", delivery)
        self.assertNotIn("--validate-only", delivery)
        for step in steps:
            command = step.get("run", "")
            for forbidden in ("npm publish", "gh release create", "git push", "gh secret"):
                self.assertNotIn(forbidden, command)


class StableAnnouncementContext(unittest.TestCase):
    def test_each_new_notice_has_a_valid_complete_approved_payload(self) -> None:
        import importlib.util
        spec = importlib.util.spec_from_file_location("subscriber_context_notifier", NOTIFIER)
        notifier = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(notifier)
        for version in ("0.15.0", "0.15.1"):
            with self.subTest(version=version):
                text = notifier.read_summary(
                    ROOT / f"docs/releases/{version}.subscriber.ko.md", version,
                    ROOT / f"docs/releases/{version}.subscriber.ko.sha256")
                self.assertLessEqual(len(text), 2000)
                self.assertNotRegex(text, r"OV15[01]-|ADR-[0-9]|sha256:|control_epoch|dispatch-uncertain")

    def test_correction_names_and_explains_every_shipped_god_with_a_call(self) -> None:
        import tomllib
        gods = tomllib.loads((ROOT / "harness/gods.toml").read_text(encoding="utf-8"))["gods"]
        text = (ROOT / "docs/releases/0.15.0.subscriber.ko.md").read_text(encoding="utf-8")
        self.assertEqual(len(gods), 9)
        self.assertIn("0.15.0 안내 정정", text)
        self.assertIn("기존 기능", text)
        for god in gods:
            with self.subTest(god=god["name"]):
                line = next(line for line in text.splitlines() if f"·{god['korean']}**" in line)
                self.assertIn(god["name"], line.casefold())
                self.assertIn(f"`{god['korean']}, ", line)
                self.assertIn(":", line)
        self.assertIn("이번 버전에서 켜는 경로 없음", text)
        self.assertIn("Gemini", text)
        self.assertIn("미검증", text)

    def test_151_copy_explains_detection_and_output_without_a_release_claim(self) -> None:
        text = (ROOT / "docs/releases/0.15.1.subscriber.ko.md").read_text(encoding="utf-8")
        for phrase in ("한 번 확인", "직접 선택", "모델 호출", "한국어·영어", "--output json",
                       "코드 개발·문서 작성·일반 작업·사용자 지정", "자동 축소 없음"):
            self.assertIn(phrase, text)
        self.assertNotIn("출시 완료", text)


class StableNotificationCorrection(unittest.TestCase):
    setUp_server = StableReleaseDiscordNotification.setUp
    tearDown = StableReleaseDiscordNotification.tearDown
    run_notifier = StableReleaseDiscordNotification.run_notifier
    OLD_ID = "123456789012345678"
    NEW_ID = "987654321098765432"

    def setUp(self) -> None:
        self.setUp_server()
        self.server.correction_mode = True
        self.server.correction_requests = []
        self.server.correction_responses = []
        self.summary = (ROOT / "docs/releases/0.15.0.subscriber.ko.md").read_text(encoding="utf-8").strip()

    # Reuse the server harness, without repeating the parent's ordinary-send tests.
    def run_correction(self, responses, *, message_id="", **options):
        self.server.correction_responses = responses
        return self.run_notifier(version="0.15.0", summary_text=self.summary,
                                 correction=True, message_id=message_id, **options)

    def original(self, content=None):
        return {"id": self.OLD_ID, "content": content or "# Aigent Hive v0.15.0 업데이트 내역:\n\n- 원래 안내"}

    def test_known_id_edits_without_posting_a_duplicate(self) -> None:
        result = self.run_correction([(200, self.original()), (200, {"id": self.OLD_ID, "echo_content": True})], message_id=self.OLD_ID)
        self.assertEqual(result.returncode, 0, result.stderr)
        receipt = json.loads(result.stdout)
        self.assertEqual(receipt["operation"], "edited")
        self.assertFalse(receipt["manual_delete_required"])
        self.assertEqual([r[0] for r in self.server.correction_requests], ["GET", "PATCH"])
        payload = self.server.correction_requests[-1][2]
        self.assertEqual(payload["content"], self.summary)
        self.assertEqual(payload["allowed_mentions"], {"parse": []})

    def test_known_id_already_corrected_needs_no_write(self) -> None:
        result = self.run_correction([(200, self.original(self.summary))], message_id=self.OLD_ID)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["operation"], "already-corrected")
        self.assertEqual([r[0] for r in self.server.correction_requests], ["GET"])

    def test_patch_unavailable_deletes_then_replaces(self) -> None:
        result = self.run_correction([(200, self.original()), (405, {}), (204, {}),
            (200, {"id": self.NEW_ID, "echo_content": True})], message_id=self.OLD_ID)
        self.assertEqual(result.returncode, 0, result.stderr)
        receipt = json.loads(result.stdout)
        self.assertEqual(receipt["operation"], "replaced")
        self.assertFalse(receipt["manual_delete_required"])
        self.assertEqual([r[0] for r in self.server.correction_requests], ["GET", "PATCH", "DELETE", "POST"])

    def test_missing_id_posts_marked_correction_and_records_new_id_and_manual_delete(self) -> None:
        result = self.run_correction([(200, {"id": self.NEW_ID, "echo_content": True})])
        self.assertEqual(result.returncode, 0, result.stderr)
        receipt = json.loads(result.stdout)
        self.assertEqual(receipt["message_id"], self.NEW_ID)
        self.assertTrue(receipt["manual_delete_required"])
        self.assertEqual(receipt["operation"], "posted-correction")
        method, path, payload = self.server.correction_requests[0]
        self.assertEqual(method, "POST")
        self.assertTrue(path.endswith("?wait=true"))
        self.assertIn("0.15.0 안내 정정", payload["content"])
        for secret in ("/api/webhooks", "127.0.0.1", self.summary):
            self.assertNotIn(secret, result.stdout + result.stderr)

    def test_unaddressable_or_forbidden_old_message_records_manual_deletion(self) -> None:
        cases = [[(404, {}), (200, {"id": self.NEW_ID, "echo_content": True})],
                 [(200, self.original()), (403, {}), (403, {}),
                  (200, {"id": self.NEW_ID, "echo_content": True})]]
        for responses in cases:
            with self.subTest(responses=responses):
                self.server.correction_requests = []
                result = self.run_correction(responses, message_id=self.OLD_ID)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertTrue(json.loads(result.stdout)["manual_delete_required"])

    def test_wrong_notice_id_never_modifies_or_posts(self) -> None:
        result = self.run_correction([(200, self.original("# Aigent Hive v0.14.2 업데이트 내역:"))], message_id=self.OLD_ID)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([r[0] for r in self.server.correction_requests], ["GET"])

    def test_transient_edit_failure_never_deletes_or_posts(self) -> None:
        for status in (429, 500):
            with self.subTest(status=status):
                self.server.correction_requests = []
                result = self.run_correction([(200, self.original()), (status, {})], message_id=self.OLD_ID)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual([r[0] for r in self.server.correction_requests], ["GET", "PATCH"])

    def test_redirects_are_not_followed_and_post_is_not_retried(self) -> None:
        result = self.run_correction([(302, {})])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(len(self.server.correction_requests), 1)

    def test_invalid_post_receipt_is_not_success_and_not_retried(self) -> None:
        result = self.run_correction([(200, {"id": "invalid"})])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("inspect delivery before retrying", result.stderr)
        self.assertEqual(len(self.server.correction_requests), 1)

    def test_uncertain_write_leaves_an_unknown_receipt_that_blocks_a_blind_rerun(self) -> None:
        for responses in ([(302, {})], [(200, {"id": "invalid"})]):
            with self.subTest(responses=responses):
                self.server.correction_requests = []
                result = self.run_correction(responses)
                self.assertNotEqual(result.returncode, 0)
                self.assertIsNotNone(self.last_receipt)
                self.assertEqual(self.last_receipt["status"], "unknown")
                self.assertEqual(self.last_receipt["attempts"][-1]["method"], "POST")
                self.assertIn("Inspect Discord before any retry", self.last_receipt["manual_action"])
                text = json.dumps(self.last_receipt)
                for secret in ("/api/webhooks", "127.0.0.1", self.summary):
                    self.assertNotIn(secret, text)

    def test_refused_write_or_lookup_failure_leaves_no_receipt(self) -> None:
        for responses in ([(200, self.original()), (429, {})], [(500, {})]):
            with self.subTest(responses=responses):
                self.server.correction_requests = []
                result = self.run_correction(responses, message_id=self.OLD_ID)
                self.assertNotEqual(result.returncode, 0)
                self.assertIsNone(self.last_receipt)

    def test_validation_approval_digest_marker_and_id_fail_before_network(self) -> None:
        self.assertEqual(self.run_correction([], validate_only=True).returncode, 0)
        for options in ({"approval_value": None}, {"approval_text": "sha256:" + "0" * 64 + "  summary.md\n"},
                        {"message_id": "not-a-message"}):
            with self.subTest(options=options):
                self.assertNotEqual(self.run_correction([], **options).returncode, 0)
        self.summary = self.summary.replace("0.15.0 안내 정정", "생략된 설명 보완")
        self.assertNotEqual(self.run_correction([]).returncode, 0)
        self.assertEqual(self.server.correction_requests, [])


class StableNoticeOrdering(unittest.TestCase):
    def test_pending_correction_blocks_direct_send_and_newer_recovery(self) -> None:
        pending = (ROOT / "docs/releases/discord-pending-corrections.txt").read_text(encoding="utf-8").split()
        # 0.15.0 was corrected on 2026-11-14 (run 43554636724); listed versions must be valid.
        for version in pending:
            self.assertRegex(version, r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$")
        publish = yaml.safe_load((ROOT / ".github/workflows/release-publish.yml").read_text(encoding="utf-8"))
        steps = {step.get("name"): step for step in publish["jobs"]["publish"]["steps"]}
        guard = steps["Validate stable Discord subscriber update"]["run"]
        self.assertIn('"$SEND_SUBSCRIBER_UPDATE" = true', guard)
        self.assertIn("docs/releases/discord-pending-corrections.txt", guard)
        self.assertLess(guard.index("exit 3"), guard.index("publish-stable-discord-update.py"))
        names = [step.get("name") for step in publish["jobs"]["publish"]["steps"]]
        self.assertLess(names.index("Validate stable Discord subscriber update"),
                        names.index("Publish npm package family through trusted publishing"))
        recover = yaml.safe_load((ROOT / ".github/workflows/release-discord-notification.yml").read_text(encoding="utf-8"))
        contract = recover["jobs"]["notify"]["steps"][0]["run"]
        self.assertIn("discord-pending-corrections.txt?ref=$source_sha", contract)
        self.assertIn("stable-discord-correction-$previous&", contract)
        self.assertLess(contract.index("must be sent before this notice"), contract.index("GITHUB_OUTPUT"))


class StableCorrectionWorkflowContract(unittest.TestCase):
    def test_correction_uses_reviewed_main_source_after_the_original_publication(self):
        workflow = yaml.safe_load((ROOT / ".github/workflows/release-discord-notification.yml").read_text(encoding="utf-8"))
        job = workflow["jobs"]["correct"]
        self.assertEqual(job["if"], "${{ inputs.operation == 'correct' }}")
        self.assertEqual(job["environment"]["name"], "release-publication")
        contract = job["steps"][0]["run"]
        for required in ('refs/heads/main', 'test "$PRODUCT_VERSION" = 0.15.0',
                         'summary_source_sha=$GITHUB_SHA', 'stable-discord-correction-$PRODUCT_VERSION"',
                         '.isPrerelease', '.isDraft',
                         '.head_sha', '.conclusion == "success"', 'duplicate send refused'):
            self.assertIn(required, contract)
        self.assertEqual(job["steps"][1]["with"]["ref"], "${{ steps.contract.outputs.summary_source_sha }}")
        commands = [step for step in job["steps"] if "scripts/publish-stable-discord-update.py" in step.get("run", "")]
        self.assertEqual(len(commands), 2)
        self.assertIn("--validate-only", commands[0]["run"])
        self.assertNotIn("--validate-only", commands[1]["run"])
        for step in commands:
            self.assertEqual(step["env"]["AIGENT_HIVE_SUBSCRIBER_APPROVAL"], "${{ vars.AIGENT_HIVE_SUBSCRIBER_APPROVAL }}")
            self.assertEqual(step["env"]["AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL"], "${{ secrets.AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL }}")
            self.assertIn("--summary-approval", step["run"])
            self.assertIn("--correction", step["run"])
        for step in job["steps"]:
            for forbidden in ("npm publish", "gh release create", "git push", "gh secret"):
                self.assertNotIn(forbidden, step.get("run", ""))
