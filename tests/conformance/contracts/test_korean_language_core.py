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
            check=False,
            timeout=20,
        )
        try:
            result = json.loads(process.stdout)
        except json.JSONDecodeError as error:
            self.fail(f"invalid CLI JSON: {error}\nstdout={process.stdout!r}\nstderr={process.stderr!r}")
        return process, result

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
