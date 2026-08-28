"""No-download CLI gates for the optional vector runtime."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[3]


class VectorCli(unittest.TestCase):
    def setUp(self):
        (ROOT / "tests/work").mkdir(exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="vector-cli-", dir=ROOT / "tests/work")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        for language in ("en", "ko"):
            (self.root / "docs/facts" / language).mkdir(parents=True)
        (self.root / "hive-source.json").write_text(json.dumps({"schema_version": 1, "kind": "aigent-hive-source-workspace", "consumer_setup_allowed": False}), encoding="utf-8")
        self.binary = os.environ.get("HIVE_BIN", str(ROOT / "target/debug" / ("hive.exe" if os.name == "nt" else "hive")))

    def run_cli(self, action, *arguments):
        process = subprocess.run([self.binary, "source-wiki", "vector", action,
            "--target", str(self.root), "--language", "ko", *arguments, "--output", "json"], capture_output=True, text=True, encoding="utf-8", timeout=30)
        value = json.loads(process.stdout)
        Draft202012Validator(json.loads((ROOT / "schemas/action-result.schema.json").read_text("utf-8"))).validate(value)
        self.assertEqual(process.returncode, value["exit_code"])
        return value

    def test_missing_runtime_status_is_read_only(self):
        result = self.run_cli("status")
        self.assertEqual(result["exit_code"], 0)
        self.assertFalse(result["data"]["enabled"])
        self.assertFalse((self.root / ".hive").exists())
        self.assertFalse((self.root / ".agents").exists())

    def test_missing_python_or_consent_never_installs(self):
        self.assertNotEqual(self.run_cli("preview")["exit_code"], 0)
        self.assertNotEqual(self.run_cli("enable")["exit_code"], 0)
        self.assertFalse((self.root / ".agents").exists())

    def test_preview_and_wrong_consent_have_no_writes(self):
        result = self.run_cli("preview", "--python", str(Path(sys.executable).resolve()))
        if sys.version_info[:2] not in ((3, 12), (3, 13)) or (sys.platform == "darwin" and os.uname().machine != "arm64"):
            self.assertNotEqual(result["exit_code"], 0)
            return
        self.assertEqual(result["exit_code"], 0, result)
        data = result["data"]
        self.assertTrue(data["consent_digest"].startswith("sha256:"))
        self.assertFalse(data["provider_api"])
        self.assertFalse(data["python_install"])
        self.assertEqual(data["cpu_policy"], "highest-efficiency-class-within-existing-affinity-and-default-cpu-sets" if os.name == "nt" else "host-default")
        self.assertEqual(data["writes_under"], [".agents/work/vector", ".agents/work/vector-control"])
        self.assertGreater(data["identity"]["download_bytes"], 250_000_000)
        self.assertNotEqual(self.run_cli("enable", "--python", str(Path(sys.executable).resolve()), "--consent-digest", "sha256:" + "0"*64)["exit_code"], 0)
        self.assertFalse((self.root / ".agents").exists())

    def test_consumer_options_cannot_redirect_source_vectors(self):
        self.assertNotEqual(self.run_cli("status", "--user-root", str(self.root))["exit_code"], 0)
        self.assertFalse((self.root / ".hive").exists())

    def consumer(self):
        user = self.root / "user"
        (user / ".hive/config").mkdir(parents=True)
        preferences = {"schema_version":1,"interface_language":"en","wiki":{"enabled":True,"language":"both"},
            "profile":{"id":"web-developer"},"persona":{"id":"balanced"},"selected_hosts":["codex"],
            "skills":{"mode":"individual","selected":["setup-hive"]},
            "usage_guard":{"enabled":False,"stop_remaining_percent":20,"codexbar_fallback_enabled":False}}
        (user / ".hive/config/user-setup.yml").write_text(json.dumps(preferences),encoding="utf-8")
        return user

    def test_shared_list_rejects_invalid_scopes_without_installing_or_changing_canonical_files(self):
        user = self.consumer()
        refresh = subprocess.run([self.binary,"knowledge","refresh","--user-root",str(user),"--output","json"],capture_output=True,text=True,encoding="utf-8",timeout=30)
        self.assertEqual(refresh.returncode, 0, refresh.stdout)
        before = {str(path.relative_to(user)):path.read_bytes() for path in user.rglob("*") if path.is_file()}
        cases = [("[]", []), ('["user-root","user-root"]', []), ('["user-root"]', ["--collection","user-root"]),
                 ('["user-root"]', ["--authorization-id","unused"]), ('["unknown"]', []),
                 ('["user-root"]', []), ('{"collection":"user-root"}', []), (json.dumps(["user-root"]*101), [])]
        for raw, extra in cases:
            with self.subTest(raw=raw[:70], extra=extra):
                result = subprocess.run([self.binary,"knowledge","vector","rebuild","--user-root",str(user),"--target",str(user),
                    "--visibility","shared","--collections",raw,*extra,"--output","json"],capture_output=True,text=True,encoding="utf-8",timeout=30)
                self.assertNotEqual(result.returncode, 0)
                value = json.loads(result.stdout)
                Draft202012Validator(json.loads((ROOT/"schemas/action-result.schema.json").read_text("utf-8"))).validate(value)
        for visibility in ("project-private", "confidential"):
            result = subprocess.run([self.binary,"knowledge","vector","rebuild","--user-root",str(user),"--target",str(user),
                "--visibility",visibility,"--collections",'["user-root"]',"--output","json"],capture_output=True,text=True,encoding="utf-8",timeout=30)
            self.assertNotEqual(result.returncode, 0)
        self.assertNotEqual(self.run_cli("rebuild", "--collections", '["user-root"]')["exit_code"], 0)
        self.assertEqual(before, {str(path.relative_to(user)):path.read_bytes() for path in user.rglob("*") if path.is_file()})
        self.assertFalse((user/".hive/index/vector").exists())

    def test_semantic_mode_without_runtime_preserves_fts_results(self):
        user = self.consumer()
        def knowledge(action,*options):
            process = subprocess.run([self.binary,"knowledge",action,"--user-root",str(user),*options,"--output","json"],capture_output=True,text=True,encoding="utf-8",timeout=30)
            result = json.loads(process.stdout)
            self.assertEqual(process.returncode,0,result)
            return result["data"]
        knowledge("remember","--user-statement","Use concise explanations for technical guidance.","--claim-key","concise-guidance","--kind","preference")
        arguments = ["--target",str(user),"--query","concise","--scope","global"]
        lexical = knowledge("retrieve",*arguments)
        semantic = knowledge("retrieve",*arguments,"--mode","semantic")
        Draft202012Validator(json.loads((ROOT/"schemas/knowledge-retrieval-result.schema.json").read_text("utf-8"))).validate(semantic)
        search = semantic.pop("search")
        self.assertEqual(search["used"],["fts"])
        self.assertEqual(search["fusion"]["ranking_policy"], "literal-fts-order-v1")
        self.assertFalse(search["fusion"]["fts_order_preserved"])
        for hit in semantic["hits"]:
            self.assertEqual(hit.pop("matched_lanes"),["fts"])
            hit.pop("fusion_rank")
        self.assertEqual(semantic,lexical)
        self.assertFalse((user/".hive/index/vector").exists())


if __name__ == "__main__":
    unittest.main()
