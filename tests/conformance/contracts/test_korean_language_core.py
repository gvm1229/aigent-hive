"""Korean language core, projection, and pack-lifecycle contracts."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import jsonschema
import yaml


ROOT = Path(__file__).resolve().parents[3]
PACK = ROOT / "harness/language-packs/im-not-ai/2.3.2"
HIVE = Path(os.environ.get("HIVE_BIN", ROOT / "target/debug/hive.exe"))


def digest(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


class KoreanLanguageCoreContract(unittest.TestCase):
    def invoke(self, *arguments: str) -> tuple[subprocess.CompletedProcess[str], dict[str, object]]:
        process = subprocess.run(
            [str(HIVE), "korean", *arguments, "--output", "json"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=False,
            timeout=20,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"invalid CLI JSON: {error}\nstdout={process.stdout!r}\nstderr={process.stderr!r}")
        return process, result

    def candidate(self, root: Path, change) -> Path:
        candidate = root / "candidate"
        shutil.copytree(PACK, candidate)
        rules = json.loads((candidate / "rules.json").read_text("utf-8"))
        rules["pack_version"] = "2.3.3"
        rules = change(rules)
        payload = json.dumps(rules, ensure_ascii=False).encode("utf-8")
        (candidate / "rules.json").write_bytes(payload)
        manifest = json.loads((candidate / "manifest.json").read_text("utf-8"))
        manifest["pack_version"] = "2.3.3"
        manifest["rules_digest"] = digest(payload)
        (candidate / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
        return candidate

    def activate(self, target: Path, candidate: Path) -> None:
        process, preview = self.invoke("pack", "preview", "--target", str(target), "--candidate", str(candidate))
        self.assertEqual(process.returncode, 0, process.stderr)
        process, _ = self.invoke("pack", "activate", "--target", str(target), "--candidate", str(candidate),
                                 "--consent-digest", preview["data"]["consent_digest"], "--confirm-pack")
        self.assertEqual(process.returncode, 0, process.stderr)

    def test_activated_pack_changes_inspect_and_verify_and_tamper_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "consumer"
            target.mkdir()
            def tighten(rules):
                rules["rules"][0]["threshold"] = 1
                rules["profiles"]["response"]["max_change_rate"] = 0.0
                return rules
            candidate = self.candidate(root, tighten)
            self.activate(target, candidate)
            source = target / "source.md"
            after = target / "after.md"
            source.write_text("분석을 통해 결과를 확인했습니다.", encoding="utf-8")
            after.write_text("분석으로 결과를 확인했습니다.", encoding="utf-8")
            process, result = self.invoke("inspect", "--target", str(target), "--profile", "response", "--input", str(source))
            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertEqual(result["data"]["pack_version"], "2.3.3")
            self.assertIn("A-2", {item["rule_id"] for item in result["data"]["findings"]})
            process, result = self.invoke("verify", "--target", str(target), "--profile", "response", "--before", str(source), "--after", str(after))
            self.assertEqual(process.returncode, 5, process.stderr)
            self.assertIn("change-rate-exceeded", result["data"]["failures"])
            pointer = json.loads((target / ".hive/language-packs/current.json").read_text("utf-8"))
            (target / pointer["relative"] / "rules.json").write_text("{}", encoding="utf-8")
            process, _ = self.invoke("inspect", "--target", str(target), "--profile", "response", "--input", str(source))
            self.assertNotEqual(process.returncode, 0)
            process, _ = self.invoke("pack", "status", "--target", str(target))
            self.assertNotEqual(process.returncode, 0)

    def test_rules_structure_is_checked_before_preview_or_mutation(self) -> None:
        mutations = {
            "empty": lambda r: {},
            "unknown-field": lambda r: {**r, "execute": "untrusted"},
            "wrong-version": lambda r: {**r, "schema_version": 99},
            "missing-profile": lambda r: {**r, "profiles": {}},
            "unknown-kind": lambda r: {**r, "rules": [{**r["rules"][0], "kind": "execute"}]},
            "zero-threshold": lambda r: {**r, "rules": [{**r["rules"][0], "threshold": 0}]},
            "weakened-integrity": lambda r: {**r, "protected_spans": []},
            "bad-identity": lambda r: {**r, "pack_version": "2.3.4"},
        }
        for name, change in mutations.items():
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                target = root / "consumer"
                target.mkdir()
                candidate = self.candidate(root, change)
                process, _ = self.invoke("pack", "preview", "--target", str(target), "--candidate", str(candidate))
                self.assertNotEqual(process.returncode, 0, name)
                self.assertEqual(list(target.iterdir()), [])

    def test_negation_and_its_subject_cannot_change_during_rewrite(self) -> None:
        pairs = (
            ("원본 v1.2는 삭제하지 않습니다.", "사본 v1.2는 삭제하지 않습니다."),
            ("이 설정은 자동 삭제가 아닐 것입니다.", "이 설정은 자동 삭제가 맞을 것입니다."),
            ("이 설정은 파일을 삭제 못합니다.", "이 설정은 파일을 삭제합니다."),
            ("이 설정은 파일을 삭제하지 않습니다.", "이 설정은 파일을 삭제합니다."),
            ("파일을 안 지웁니다.", "파일을 지웁니다."),
            ("원본은 삭제하지 않습니다. 사본은 삭제합니다.", "원본은 삭제합니다. 사본은 삭제하지 않습니다."),
        )
        prefix = "검토 결과와 저장 위치를 확인했습니다. 나머지 설정과 실행 순서는 그대로 유지합니다. "
        for before_text, after_text in pairs:
            with self.subTest(before=before_text), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                before, after = root / "before.md", root / "after.md"
                before.write_text(prefix + before_text, encoding="utf-8")
                after.write_text(prefix + after_text, encoding="utf-8")
                process, result = self.invoke("verify", "--profile", "response", "--before", str(before), "--after", str(after))
                self.assertEqual(process.returncode, 5)
                self.assertFalse(result["data"]["accepted"])
                self.assertIn("negation-context-changed", result["data"]["failures"])

    def test_rollback_rehashes_prior_generation_before_pointer_change(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "consumer"
            target.mkdir()
            self.activate(target, PACK)
            candidate = self.candidate(root, lambda rules: rules)
            self.activate(target, candidate)
            pointer_path = target / ".hive/language-packs/current.json"
            before = pointer_path.read_bytes()
            pointer = json.loads(before)
            (target / pointer["previous"]["relative"] / "rules.json").write_bytes(b"{}")
            process, _ = self.invoke("pack", "rollback", "--target", str(target))
            self.assertNotEqual(process.returncode, 0)
            self.assertEqual(pointer_path.read_bytes(), before)

    def test_corrupt_current_pack_can_restore_valid_prior_or_embedded_rules(self) -> None:
        for has_prior in (False, True):
            with self.subTest(prior=has_prior), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                target = root / "consumer"
                target.mkdir()
                if has_prior:
                    self.activate(target, PACK)
                candidate = self.candidate(root, lambda rules: rules)
                self.activate(target, candidate)
                pointer_path = target / ".hive/language-packs/current.json"
                pointer = json.loads(pointer_path.read_bytes())
                (target / pointer["relative"] / "rules.json").write_bytes(b"{}")
                process, restored = self.invoke("pack", "rollback", "--target", str(target))
                self.assertEqual(process.returncode, 0, process.stderr)
                self.assertEqual(restored["data"]["pack_version"], "2.3.2")
                process, status = self.invoke("pack", "status", "--target", str(target))
                self.assertEqual(process.returncode, 0, process.stderr)
                self.assertEqual(status["data"]["pack_version"], "2.3.2")

    def test_cwd_selection_and_repeat_activation_preserve_embedded_rollback(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "consumer"
            target.mkdir()
            candidate = self.candidate(root, lambda rules: rules)
            self.activate(target, candidate)
            self.activate(target, candidate)
            text = target / "input.md"
            text.write_text("검사할 한국어 문장입니다.", encoding="utf-8")
            process = subprocess.run([str(HIVE.resolve()), "korean", "inspect", "--profile", "response", "--input", str(text), "--output", "json"],
                                     cwd=target, capture_output=True, timeout=20)
            self.assertEqual(process.returncode, 0)
            self.assertEqual(json.loads(process.stdout)["data"]["pack_version"], "2.3.3")
            process, restored = self.invoke("pack", "rollback", "--target", str(target))
            self.assertEqual(process.returncode, 0, process.stderr)
            self.assertEqual(restored["data"]["pack_version"], "2.3.2")

    def test_pointer_path_escape_is_rejected_without_external_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "consumer"
            target.mkdir()
            outside = root / "outside.txt"
            outside.write_bytes(b"external sentinel")
            self.activate(target, PACK)
            pointer_path = target / ".hive/language-packs/current.json"
            original = json.loads(pointer_path.read_bytes())
            for relative in ("../../outside.txt", str(outside), ".hive/language-packs/packs/unknown"):
                pointer_path.write_text(json.dumps({**original, "relative": relative}), encoding="utf-8")
                before = pointer_path.read_bytes()
                for action in ("status", "rollback"):
                    process, _ = self.invoke("pack", action, "--target", str(target))
                    self.assertNotEqual(process.returncode, 0)
                    self.assertEqual(pointer_path.read_bytes(), before)
                    self.assertEqual(outside.read_bytes(), b"external sentinel")

    def test_pinned_manifest_matches_schema_rules_license_and_upstream_boundary(self) -> None:
        manifest = json.loads((PACK / "manifest.json").read_text("utf-8"))
        schema = json.loads((ROOT / "schemas/korean-language-pack.schema.json").read_text("utf-8"))
        jsonschema.Draft202012Validator(schema).validate(manifest)
        self.assertEqual(manifest["upstream_commit"], "0ac1e84f92334f9696e69184478f91c1c6f1dc5e")
        self.assertEqual(manifest["rules_digest"], digest((PACK / "rules.json").read_bytes()))
        self.assertEqual(
            manifest["shipped_license_digest"],
            digest((PACK / "UPSTREAM-LICENSE.txt").read_bytes()),
        )
        self.assertEqual(manifest["upstream_symlink_count"], 0)
        self.assertFalse(manifest["raw_install_allowed"])
        self.assertFalse(manifest["floating_ref_allowed"])
        self.assertFalse(manifest["automatic_update_allowed"])
        self.assertNotEqual(
            manifest["host_versions"]["gemini_extension"],
            manifest["host_versions"]["codex_plugin"],
        )

    def test_inspect_verify_and_sanitize_are_deterministic_and_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.md"
            safe = root / "safe.md"
            unsafe = root / "unsafe.md"
            sanitized = root / "sanitized.md"
            source.write_text(
                "분석을 통해 확인했다. 결과를 통해 비교했다. 자료를 통해 검증했다. "
                "`hive test` 결과는 12.5%였다. [근거](https://example.com)는 “유지해야 한다”라고 썼다.",
                encoding="utf-8",
            )
            safe.write_text(
                "분석으로 확인했다. 결과를 비교했다. 자료를 검증했다. "
                "`hive test` 결과는 12.5%였다. [근거](https://example.com)는 “유지해야 한다”라고 썼다.",
                encoding="utf-8",
            )
            unsafe.write_text(source.read_text("utf-8").replace("12.5%", "13%"), encoding="utf-8")
            first, inspection = self.invoke("inspect", "--profile", "response", "--input", str(source))
            second, repeated = self.invoke("inspect", "--profile", "response", "--input", str(source))
            self.assertEqual(first.returncode, 0, first.stderr)
            self.assertEqual(second.returncode, 0, second.stderr)
            self.assertEqual(inspection["data"], repeated["data"])
            self.assertIn("A-2", {item["rule_id"] for item in inspection["data"]["findings"]})
            accepted, accepted_result = self.invoke(
                "verify", "--profile", "response", "--before", str(source), "--after", str(safe)
            )
            self.assertEqual(accepted.returncode, 0, accepted.stderr)
            self.assertTrue(accepted_result["data"]["accepted"])
            rejected, rejected_result = self.invoke(
                "verify", "--profile", "response", "--before", str(source), "--after", str(unsafe)
            )
            self.assertEqual(rejected.returncode, 5, rejected.stderr)
            self.assertTrue(rejected_result["data"]["fallback_required"])
            controls = root / "controls.txt"
            controls.write_text("가\u200b나\u202e다", encoding="utf-8")
            cleaned, cleaned_result = self.invoke(
                "sanitize", "--input", str(controls), "--output-file", str(sanitized)
            )
            self.assertEqual(cleaned.returncode, 0, cleaned.stderr)
            self.assertEqual(sanitized.read_text("utf-8"), "가나다")
            self.assertFalse(cleaned_result["data"]["watermark_claim"])

    def test_humanize_skill_and_korean_directive_have_exact_projection_parity(self) -> None:
        canonical = (ROOT / "harness/skills/humanize-kor/SKILL.md").read_bytes()
        for relative in (
            "harness/plugins/aigent-hive/skills/humanize-kor/SKILL.md",
            "harness/template/.agents/skills/humanize-kor/SKILL.md",
            "harness/template/.claude/skills/humanize-kor/SKILL.md",
        ):
            self.assertEqual((ROOT / relative).read_bytes(), canonical)
        metadata = yaml.safe_load(
            (ROOT / "harness/skills/humanize-kor/agents/openai.yaml").read_text("utf-8")
        )
        self.assertTrue(metadata["policy"]["allow_implicit_invocation"])
        text = canonical.decode("utf-8")
        normalized = " ".join(text.split())
        for boundary in (
            "watermark evasion",
            "source concealment",
            "false claims of human authorship",
            "hive korean inspect",
            "hive korean verify",
        ):
            self.assertIn(boundary, normalized)

    def test_pack_activation_requires_preview_consent_and_rolls_back(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "consumer"
            target.mkdir()
            first = root / "first"
            shutil.copytree(PACK, first)
            preview_process, preview = self.invoke(
                "pack", "preview", "--target", str(target), "--candidate", str(first)
            )
            self.assertEqual(preview_process.returncode, 0, preview_process.stderr)
            denied, _ = self.invoke(
                "pack",
                "activate",
                "--target",
                str(target),
                "--candidate",
                str(first),
                "--consent-digest",
                preview["data"]["consent_digest"],
            )
            self.assertEqual(denied.returncode, 2)
            activated, result = self.invoke(
                "pack",
                "activate",
                "--target",
                str(target),
                "--candidate",
                str(first),
                "--consent-digest",
                preview["data"]["consent_digest"],
                "--confirm-pack",
            )
            self.assertEqual(activated.returncode, 0, activated.stderr)
            self.assertTrue(result["data"]["activated"])
            second = root / "second"
            shutil.copytree(PACK, second)
            rules = json.loads((second / "rules.json").read_text("utf-8"))
            rules["pack_version"] = "2.3.3"
            rules["transform_version"] = 2
            (second / "rules.json").write_text(
                json.dumps(rules, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
            manifest = json.loads((second / "manifest.json").read_text("utf-8"))
            manifest["pack_version"] = "2.3.3"
            manifest["transform_version"] = 2
            manifest["rules_digest"] = digest((second / "rules.json").read_bytes())
            (second / "manifest.json").write_text(
                json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
            _, second_preview = self.invoke(
                "pack", "preview", "--target", str(target), "--candidate", str(second)
            )
            second_activation, _ = self.invoke(
                "pack",
                "activate",
                "--target",
                str(target),
                "--candidate",
                str(second),
                "--consent-digest",
                second_preview["data"]["consent_digest"],
                "--confirm-pack",
            )
            self.assertEqual(second_activation.returncode, 0, second_activation.stderr)
            rollback, rollback_result = self.invoke("pack", "rollback", "--target", str(target))
            self.assertEqual(rollback.returncode, 0, rollback.stderr)
            self.assertEqual(rollback_result["data"]["pack_version"], "2.3.2")

    def test_gold_corpus_and_static_gate_cover_profiles_and_preservation(self) -> None:
        gold = json.loads((ROOT / "tests/fixtures/korean/gold.json").read_text("utf-8"))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for case in gold["cases"]:
                path = root / f"{case['id']}.md"
                path.write_text(case["text"], encoding="utf-8")
                process, result = self.invoke(
                    "inspect", "--profile", case["profile"], "--input", str(path)
                )
                self.assertEqual(process.returncode, 0, process.stderr)
                self.assertEqual(
                    {finding["rule_id"] for finding in result["data"]["findings"]},
                    set(case["expected_rule_ids"]),
                    case["id"],
                )
            for case in gold["verification_cases"]:
                before = root / f"{case['id']}-before.md"
                after = root / f"{case['id']}-after.md"
                before.write_text(case["before"], encoding="utf-8")
                after.write_text(case["after"], encoding="utf-8")
                process, result = self.invoke(
                    "verify",
                    "--profile",
                    case["profile"],
                    "--before",
                    str(before),
                    "--after",
                    str(after),
                )
                self.assertEqual(result["data"]["accepted"], case["accepted"], case["id"])
                self.assertEqual(process.returncode == 0, case["accepted"])
            good = root / "good.md"
            good.write_text(gold["cases"][0]["text"], encoding="utf-8")
            receipt = root / "receipt.json"
            gate = subprocess.run(
                [
                    os.sys.executable,
                    str(ROOT / "scripts/check-korean-output.py"),
                    "--hive-bin",
                    str(HIVE),
                    "--profile",
                    "response",
                    "--input",
                    str(good),
                    "--output",
                    str(receipt),
                ],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
                timeout=20,
            )
            self.assertEqual(gate.returncode, 0, gate.stderr)
            self.assertEqual(json.loads(receipt.read_text("utf-8"))["status"], "passed")


if __name__ == "__main__":
    unittest.main()
