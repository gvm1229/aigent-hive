"""File-only runtime policy and deterministic resumable vector worker contracts.

These tests use no network, provider, model download, or real embedding package.
Real package/model execution belongs to the separately consented qualification lane.
"""

from __future__ import annotations

import base64
from contextlib import closing
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path, PureWindowsPath
import sqlite3
import stat
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / "crates/hive-cli/src"
LOCK = json.loads((SOURCE / "vector-runtime-lock.json").read_text("utf-8"))


def module(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, SOURCE / filename)
    loaded = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loaded)
    return loaded


RUNTIME = module("hive_vector_runtime", "vector_runtime.py")
WORKER = module("hive_vector_worker", "vector_helper.py")


class InlinePool:
    def __init__(self, **_kwargs):
        pass

    def __enter__(self):
        return self

    def __exit__(self, *_args):
        pass

    def map(self, _function, batches):
        vector = struct.pack("<384f", 1.0, *([0.0] * 383))
        return ([vector] * len(batch) for batch in batches)


class SimulatedVectorConnection(sqlite3.Connection):
    def execute(self, sql, parameters=()):
        if sql.startswith("CREATE VIRTUAL TABLE vectors USING vec0"):
            sql = "CREATE TABLE vectors(rowid INTEGER PRIMARY KEY, embedding BLOB NOT NULL)"
        return super().execute(sql, parameters)


class VectorRuntimeContract(unittest.TestCase):
    @staticmethod
    def cpu_record(identity, logical, efficiency, group=0, flags=0):
        raw = bytearray(32)
        struct.pack_into("<IIIH", raw, 0, 32, 0, identity, group)
        raw[14], raw[18], raw[19] = logical, efficiency, flags
        return bytes(raw)

    def test_cpu_profile_selects_one_class_without_expanding_affinity_or_default_sets(self):
        raw = self.cpu_record(10, 0, 1) + self.cpu_record(11, 1, 1) + self.cpu_record(12, 2, 0)
        self.assertEqual(WORKER.windows_performance_mask(raw, 7), 3)
        self.assertEqual(WORKER.windows_performance_mask(raw, 6), 2)
        self.assertEqual(WORKER.windows_performance_mask(raw, 7, {11}), 2)
        for mask, selected in ((4, None), (7, {12}), (7, set())):
            with self.subTest(mask=mask, selected=selected), self.assertRaises(ValueError):
                WORKER.windows_performance_mask(raw, mask, selected)
        reserved = self.cpu_record(10, 0, 1, flags=2) + self.cpu_record(11, 1, 1, flags=6) + self.cpu_record(12, 2, 0)
        self.assertEqual(WORKER.windows_performance_mask(reserved, 7), 2)

    def test_cpu_profile_keeps_homogeneous_hosts_and_rejects_malformed_hybrid_topology(self):
        self.assertEqual(WORKER.windows_performance_mask(self.cpu_record(1, 0, 0, group=1), 5), 5)
        unknown = struct.pack("<II", 8, 99)
        self.assertEqual(WORKER.windows_performance_mask(unknown + self.cpu_record(1, 0, 0), 1), 1)
        for raw in (b"x", struct.pack("<II", 0, 0), struct.pack("<II", 500, 0), struct.pack("<II", 8, 0), unknown,
                    self.cpu_record(1, 0, 1, group=1) + self.cpu_record(2, 1, 0)):
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                WORKER.windows_performance_mask(raw, 3)
        with patch.object(WORKER.sys, "platform", "linux"):
            self.assertIsNone(WORKER.stabilize_cpu_profile())

    def test_cpu_profile_is_applied_before_loading_a_model(self):
        with patch.object(WORKER, "stabilize_cpu_profile", side_effect=ValueError("unsupported CPU")), \
             patch.object(WORKER, "load_runtime") as load:
            with self.assertRaisesRegex(ValueError, "unsupported CPU"):
                WORKER.initialize_encoder("not-a-runtime")
            load.assert_not_called()

    @unittest.skipUnless(sys.platform == "win32", "actual Windows worker affinity; other hosts retain their policy")
    def test_windows_cpu_profile_changes_only_its_child_process(self):
        import ctypes as c
        from ctypes import wintypes as w
        api = c.WinDLL("kernel32", use_last_error=True)
        api.GetCurrentProcess.restype = w.HANDLE
        api.GetProcessAffinityMask.argtypes = [w.HANDLE, c.POINTER(c.c_size_t), c.POINTER(c.c_size_t)]
        def mask():
            value, system = c.c_size_t(), c.c_size_t()
            self.assertTrue(api.GetProcessAffinityMask(api.GetCurrentProcess(), c.byref(value), c.byref(system)))
            return value.value
        before = mask()
        code = ("import runpy,sys,json,ctypes as c\nfrom ctypes import wintypes as w\n"
                "api=c.WinDLL('kernel32',use_last_error=True)\napi.GetCurrentProcess.restype=w.HANDLE\n"
                "api.GetProcessAffinityMask.argtypes=[w.HANDLE,c.POINTER(c.c_size_t),c.POINTER(c.c_size_t)]\n"
                "a,s=c.c_size_t(),c.c_size_t()\nassert api.GetProcessAffinityMask(api.GetCurrentProcess(),c.byref(a),c.byref(s))\n"
                "before=a.value\nselected=runpy.run_path(sys.argv[1])['stabilize_cpu_profile']()\n"
                "assert api.GetProcessAffinityMask(api.GetCurrentProcess(),c.byref(a),c.byref(s))\n"
                "print(json.dumps({'before':before,'selected':selected,'after':a.value}))")
        result = subprocess.run([sys.executable, "-I", "-S", "-B", "-c", code, str(SOURCE / "vector_helper.py")],
                                capture_output=True, text=True, timeout=10, check=True)
        observed = json.loads(result.stdout)
        self.assertEqual(observed["selected"], observed["after"])
        self.assertEqual(observed["after"] & observed["before"], observed["after"])
        self.assertEqual(mask(), before)

    def test_native_loader_path_keeps_long_drive_and_unc_names_without_changing_posix(self):
        with patch.object(WORKER.sys, "platform", "win32"):
            for normal, expected in (
                (r"C:\긴 경로\site", r"\\?\C:\긴 경로\site"),
                (r"\\server\share\site", r"\\?\UNC\server\share\site"),
                (r"\\?\C:\site", r"\\?\C:\site"),
                (r"\\?\UNC\server\share\site", r"\\?\UNC\server\share\site"),
            ):
                self.assertEqual(WORKER.native_loader_path(PureWindowsPath(normal)), expected)
        with patch.object(WORKER.sys, "platform", "linux"):
            self.assertEqual(WORKER.native_loader_path(Path("/tmp/site")), str(Path("/tmp/site")))

    def test_runtime_uses_native_loader_spelling_only_for_the_import_path(self):
        runtime = self.root / "runtime"
        site = runtime / "site"
        site.mkdir(parents=True)
        with patch.object(WORKER, "native_loader_path", return_value="native-site") as native, \
             patch.object(WORKER, "offline_environment"), patch.object(WORKER.sys, "path", []):
            self.assertEqual(WORKER.load_runtime(str(runtime)), runtime)
            native.assert_called_once_with(site)
            self.assertEqual(WORKER.sys.path, ["native-site"])

    def setUp(self):
        work = ROOT / "tests/work"
        work.mkdir(exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="vector-contract-", dir=work)
        self.root = Path(self.temporary.name)

    def tearDown(self):
        self.temporary.cleanup()

    def test_all_platform_locks_pin_seven_file_only_distributions(self):
        expected = {f"{host}-cp{python}" for host in ("windows-x64", "linux-x64", "macos-arm64") for python in ("312", "313")}
        self.assertEqual(set(LOCK["platforms"]), expected)
        for key in expected:
            with patch.object(RUNTIME, "platform_key", return_value=key):
                packages = RUNTIME.validate_lock(LOCK)
                self.assertEqual({item["name"] for item in packages}, RUNTIME.PACKAGES)
                self.assertTrue(all(item["version"] for item in packages))
        self.assertEqual(LOCK["omitted_declared_dependencies"][0]["distribution"], "tokenizers")
        self.assertIn("not a complete pip environment", LOCK["omitted_declared_dependencies"][0]["reason"])
        self.assertNotIn("huggingface_hub", {item["name"] for item in LOCK["packages"].values()})

    def test_model_inventory_and_wheel_origin_are_bounded(self):
        for mutate in (
            lambda value: value["model"]["files"][0].update(filename="../../outside"),
            lambda value: value["model"].update(revision="main"),
            lambda value: value["model"].update(repository="../outside"),
            lambda value: value["model"].update(dimension=1),
        ):
            value = json.loads(json.dumps(LOCK))
            mutate(value)
            with patch.object(RUNTIME, "platform_key", return_value="windows-x64-cp312"):
                with self.assertRaises(ValueError):
                    RUNTIME.validate_lock(value)

    def test_bootstrap_describe_is_read_only_and_does_not_load_site_packages(self):
        request = {"action": "describe", "lock": LOCK}
        process = subprocess.run([sys.executable, "-I", "-S", "-B", str(SOURCE / "vector_runtime.py")], input=json.dumps(request).encode(), capture_output=True, cwd=self.root, timeout=15)
        value = json.loads(process.stdout)
        if sys.version_info[:2] in ((3, 12), (3, 13)) and RUNTIME.platform_key() in LOCK["platforms"]:
            self.assertEqual(process.returncode, 0, value)
            self.assertFalse(value["complete_pip_environment"])
            self.assertEqual(value["lock_digest"], RUNTIME.digest_bytes(RUNTIME.canonical(LOCK)))
        else:
            self.assertEqual(process.returncode, 10)
        self.assertEqual(list(self.root.iterdir()), [])

    def test_wheel_paths_reject_traversal_and_activation_files(self):
        for path in ("../outside", "/absolute", "C:/outside", "a\\b", "module.pth", "module.pyc"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                RUNTIME.wheel_path(path)
        self.assertIsNone(RUNTIME.wheel_path("p.data/scripts/entrypoint"))
        self.assertEqual(RUNTIME.wheel_path("p.data/platlib/module.py"), "module.py")

    def test_wheel_metadata_is_preserved_and_links_are_rejected(self):
        for mode in (stat.S_IFREG, stat.S_IFLNK):
            content = b"original package metadata"
            digest = base64.urlsafe_b64encode(hashlib.sha256(content).digest()).decode().rstrip("=")
            stream = io.BytesIO()
            with zipfile.ZipFile(stream, "w") as archive:
                entry = zipfile.ZipInfo("p.dist-info/METADATA")
                entry.external_attr = (mode | 0o644) << 16
                archive.writestr(entry, content)
                archive.writestr("p.dist-info/RECORD", f"p.dist-info/METADATA,sha256={digest},{len(content)}\np.dist-info/RECORD,,\n")
            with zipfile.ZipFile(io.BytesIO(stream.getvalue())) as archive:
                if mode == stat.S_IFLNK:
                    with self.assertRaises(ValueError):
                        RUNTIME.wheel_files(archive)
                else:
                    files = RUNTIME.wheel_files(archive)
                    self.assertEqual(files["p.dist-info/METADATA"][1], hashlib.sha256(content).hexdigest())

    def test_corrupt_cached_download_fails_without_network(self):
        digest = hashlib.sha256(b"expected").hexdigest()
        (self.root / digest).write_bytes(b"corrupt")
        with patch.object(RUNTIME.urllib.request, "build_opener", side_effect=AssertionError("network forbidden")):
            with self.assertRaises(ValueError):
                RUNTIME.fetch("https://files.pythonhosted.org/example.whl", digest, self.root)

    def test_verifier_checks_every_root_and_nested_file_after_path_reuse(self):
        site = self.root / "site"
        payloads = {"root.py": b"root bytes", "nested/한글 space/module.py": b"nested bytes", "p.dist-info/METADATA": b"metadata"}
        records = []
        for relative, content in payloads.items():
            encoded = base64.urlsafe_b64encode(hashlib.sha256(content).digest()).decode().rstrip("=")
            records.append(f"{relative},sha256={encoded},{len(content)}")
        payloads["p.dist-info/RECORD"] = ("\n".join([*records, "p.dist-info/RECORD,,"]) + "\n").encode()
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, "w") as archive:
            for relative, content in payloads.items():
                archive.writestr(relative, content)
                path = site / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(content)
        wheels = self.root / "wheels"
        wheels.mkdir()
        wheel_digest = hashlib.sha256(stream.getvalue()).hexdigest()
        (wheels / (wheel_digest + ".whl")).write_bytes(stream.getvalue())
        helper = b"pass\n"
        helper_digest = RUNTIME.digest_bytes(helper)
        (self.root / "vector_helper.py").write_bytes(helper)
        (self.root / "receipt.json").write_text(json.dumps({"helper_digest": helper_digest}), encoding="utf-8")
        request = {"root": str(self.root), "helper_digest": helper_digest, "lock": {"model": {"files": []}}}
        with patch.object(RUNTIME, "describe", return_value={}), patch.object(RUNTIME, "validate_lock", return_value=[{"size": len(stream.getvalue()), "sha256": wheel_digest}]):
            self.assertTrue(RUNTIME.verify(request)["verified"])
            for relative in ("root.py", "nested/한글 space/module.py", "p.dist-info/RECORD"):
                path = site / relative
                path.write_bytes(b"changed")
                with self.subTest(relative=relative), self.assertRaises(ValueError):
                    RUNTIME.verify(request)
                path.write_bytes(payloads[relative])
            extra = site / "nested/한글 space/extra.py"
            extra.write_bytes(b"unknown")
            with self.assertRaises(ValueError):
                RUNTIME.verify(request)
            extra.unlink()
            (site / "root.py").unlink()
            with self.assertRaises(ValueError):
                RUNTIME.verify(request)

    def test_successful_runtime_cleanup_removes_only_exact_pinned_downloads(self):
        artifacts = []
        for content in (b"wheel", b"model"):
            digest = hashlib.sha256(content).hexdigest()
            (self.root / digest).write_bytes(content)
            artifacts.append({"sha256": digest, "size": len(content)})
        note = self.root / "user-note.txt"
        note.write_bytes(b"preserve")
        partial = self.root / "unfinished.partial"
        partial.write_bytes(b"retain failure evidence")
        RUNTIME.cleanup_verified_downloads(self.root, artifacts)
        self.assertEqual({path.name for path in self.root.iterdir()}, {note.name, partial.name})
        self.assertEqual(note.read_bytes(), b"preserve")
        self.assertEqual(partial.read_bytes(), b"retain failure evidence")

    def test_corrupt_or_hardlinked_download_prevents_cleanup_of_the_entire_set(self):
        good = b"good"
        digest = hashlib.sha256(good).hexdigest()
        path = self.root / digest
        path.write_bytes(good)
        bad_digest = hashlib.sha256(b"expected").hexdigest()
        (self.root / bad_digest).write_bytes(b"modified")
        with self.assertRaises(ValueError):
            RUNTIME.cleanup_verified_downloads(self.root, [{"sha256": digest, "size": 4}, {"sha256": bad_digest, "size": 8}])
        self.assertEqual(path.read_bytes(), good)
        link = self.root / "other-owned-reference"
        os.link(path, link)
        with self.assertRaises(ValueError):
            RUNTIME.cleanup_verified_downloads(self.root, [{"sha256": digest, "size": 4}])
        self.assertEqual(path.read_bytes(), good)

    def test_install_verifies_before_cleanup_and_failure_retains_downloads(self):
        for fails in (False, True):
            runtime = self.root / str(fails)
            runtime.mkdir()
            events = []
            def verify(_request):
                events.append("verify")
                if fails:
                    raise ValueError("deliberate verification failure")
                return {"verified": True}
            request = {"root": str(runtime), "cache": str(self.root / "cache"), "helper": "pass", "lock": {"model": {"files": []}}}
            with patch.object(RUNTIME, "describe", return_value={}), patch.object(RUNTIME, "validate_lock", return_value=[]), \
                 patch.object(RUNTIME, "verify", side_effect=verify), \
                 patch.object(RUNTIME, "cleanup_verified_downloads", side_effect=lambda *_: events.append("cleanup")):
                if fails:
                    with self.assertRaises(ValueError):
                        RUNTIME.install(request)
                    self.assertEqual(events, ["verify"])
                else:
                    self.assertEqual(RUNTIME.install(request), {"verified": True})
                    self.assertEqual(events, ["verify", "cleanup"])

    def test_worker_rejects_unknown_fields_without_echoing_text(self):
        sentinel = "private-query-never-echo"
        request = {"schema_version": 1, "action": "execute", "code": sentinel}
        process = subprocess.run([sys.executable, "-I", "-S", "-B", str(SOURCE / "vector_helper.py")], input=json.dumps(request).encode(), capture_output=True, timeout=15)
        self.assertEqual(process.returncode, 10)
        self.assertNotIn(sentinel.encode(), process.stdout + process.stderr)

    def test_worker_rejects_boolean_schema_before_runtime_access(self):
        request = {"schema_version": True, "action": "self-test", "runtime": "not-a-runtime"}
        process = subprocess.run([sys.executable, "-I", "-S", "-B", str(SOURCE / "vector_helper.py")], input=json.dumps(request).encode(), capture_output=True, timeout=15)
        self.assertEqual(process.returncode, 10)
        self.assertEqual(json.loads(process.stdout)["error_type"], "ValueError")

    def test_bad_manifest_digest_is_rejected_before_database_access(self):
        with patch.object(WORKER, "load_runtime", side_effect=AssertionError("must not read runtime")):
            with self.assertRaises(ValueError):
                WORKER.build({"contract_digest": "sha256:" + "a" * 64, "manifest_digest": "not-a-digest"})

    def build_window(self):
        base = self.root / ".hive/index/vector"
        runtime = base / "runtimes" / ("a" * 64)
        (runtime / "site").mkdir(parents=True)
        databases = []
        for number, letter in enumerate(("b", "c")):
            scope = base / "scopes" / (letter * 64)
            scope.mkdir(parents=True)
            databases.append({"database":str(scope / "staging.sqlite3"),"chunks":[{
                "chunk_id":f"scope-{number}","digest":"sha256:"+"d"*64,"title":"Example","text":f"scope {number}"}],
                "manifest_digest":"sha256:"+letter*64,"expected_database_digest":None})
        return runtime, {"schema_version":1,"action":"build-many","runtime":str(runtime),
                         "contract_digest":"sha256:"+"e"*64,"workers":1,"max_seconds":30,"databases":databases}

    def test_build_window_shares_encoder_but_keeps_each_database_and_noop_separate(self):
        runtime, request = self.build_window()
        connect = lambda path: sqlite3.connect(path, factory=SimulatedVectorConnection)
        with patch.object(WORKER, "load_runtime", return_value=runtime), patch.object(WORKER, "initialize_encoder") as initialize, \
             patch.object(WORKER, "open_database", side_effect=connect), patch.object(WORKER.concurrent.futures, "ThreadPoolExecutor", InlinePool):
            result = WORKER.execute_request(request)
            self.assertEqual(initialize.call_count, 1)
            self.assertEqual(result["not_started"], [])
            for number, item in enumerate(result["results"]):
                self.assertEqual(item["index"], number)
                self.assertEqual(item["result"]["embedded"], 1)
                self.assertTrue(item["result"]["complete"])
                database = request["databases"][number]
                database["expected_database_digest"] = item["result"]["database_digest"]
                with closing(sqlite3.connect(database["database"])) as connection:
                    self.assertEqual(connection.execute("SELECT chunk_id FROM documents").fetchall(), [(f"scope-{number}",)])
            before = [Path(item["database"]).read_bytes() for item in request["databases"]]
            initialize.reset_mock()
            noop = WORKER.execute_request(request)
            self.assertEqual([item["result"]["embedded"] for item in noop["results"]], [0, 0])
            initialize.assert_not_called()
            self.assertEqual(before, [Path(item["database"]).read_bytes() for item in request["databases"]])

    def test_build_window_budget_is_shared_and_unstarted_databases_are_untouched(self):
        runtime, request = self.build_window()
        request["max_seconds"] = 2
        with patch.object(WORKER, "load_runtime", return_value=runtime), \
             patch.object(WORKER.time, "monotonic", side_effect=[10, 10, 12]), \
             patch.object(WORKER, "build", return_value={"complete":False}) as build:
            result = WORKER.execute_request(request)
        self.assertEqual(result["not_started"], [1])
        build.assert_called_once()
        self.assertEqual(build.call_args.kwargs["final_deadline"], 42)
        self.assertFalse(any(Path(item["database"]).exists() for item in request["databases"]))

    def test_build_window_rejects_duplicate_or_invalid_later_partition_before_writes(self):
        runtime, request = self.build_window()
        original = json.loads(json.dumps(request))
        request["databases"][1] = request["databases"][0]
        with patch.object(WORKER, "load_runtime", return_value=runtime), self.assertRaises(ValueError):
            WORKER.execute_request(request)
        request = json.loads(json.dumps(original))
        request["databases"][1]["chunks"][0]["digest"] = "invalid"
        with patch.object(WORKER, "load_runtime", side_effect=AssertionError("no runtime read")), self.assertRaises(ValueError):
            WORKER.execute_request(request)
        with patch.object(WORKER, "MAX_CHUNKS", 1), self.assertRaises(ValueError):
            WORKER.execute_request(original)
        self.assertFalse(any(Path(item["database"]).exists() for item in original["databases"]))

    def test_worker_parallelism_is_bounded_before_runtime_access(self):
        request = {"contract_digest":"sha256:"+"a"*64,"manifest_digest":"sha256:"+"b"*64,"chunks":[],"max_seconds":30}
        with patch.object(WORKER, "load_runtime", side_effect=AssertionError("must not read runtime")):
            for workers in (True,0,17):
                with self.subTest(workers=workers), self.assertRaises(ValueError):
                    WORKER.build({**request,"workers":workers})

    def test_combined_query_requires_receipt_code_and_query_only_authority(self):
        (self.root / "tmp").mkdir()
        code = b"SCHEMA=1\ndef execute_request(request):\n    return {'matches': []}\n"
        helper = self.root / "vector_helper.py"
        helper.write_bytes(code)
        approved = {"receipt_digest":"sha256:"+"a"*64,"identity":{"platform":"synthetic"},"verified":True}
        request = {"root":str(self.root),"lock":{},"helper_digest":RUNTIME.digest_bytes(code),
            "receipt_digest":approved["receipt_digest"],"identity":approved["identity"],
            "request":{"schema_version":1,"action":"query","runtime":str(self.root)}}
        with patch.object(RUNTIME,"verify",return_value=approved), patch.dict(os.environ,{}):
            self.assertEqual(RUNTIME.run_verified_query(request),{"schema_version":1,"matches":[]})
            for changed in (
                {**request,"receipt_digest":"sha256:"+"b"*64},
                {**request,"identity":{"platform":"different"}},
                {**request,"helper_digest":"sha256:"+"c"*64},
                {**request,"request":{**request["request"],"action":"build"}},
                {**request,"request":{**request["request"],"runtime":str(self.root/"other")}},
            ):
                with self.subTest(changed=changed), self.assertRaises(ValueError):
                    RUNTIME.run_verified_query(changed)
            helper.write_bytes(b"raise RuntimeError('must never execute changed code')")
            with self.assertRaises(ValueError):
                RUNTIME.run_verified_query(request)

    def test_trusted_main_file_does_not_authorize_sqlite_sidecars(self):
        database = self.root / "staging.sqlite3"
        database.write_bytes(b"trusted main database")
        expected = WORKER.database_digest(database)
        for suffix in ("-journal", "-wal", "-shm"):
            sibling = Path(str(database) + suffix)
            sibling.write_bytes(b"untrusted recovery state")
            with self.subTest(suffix=suffix), self.assertRaises(ValueError):
                WORKER.authenticate_database(database, expected, create=False)
            self.assertEqual(sibling.read_bytes(), b"untrusted recovery state")
            sibling.unlink()

    def test_database_writes_cannot_target_fts_or_published_generations(self):
        base = self.root / ".hive/index/vector"
        runtime = base / "runtimes" / ("a" * 64)
        scope = base / "scopes" / ("b" * 64)
        scope.mkdir(parents=True)
        self.assertEqual(WORKER.database_path(runtime, str(scope / "staging.sqlite3"), True), scope / "staging.sqlite3")
        for path in (base.parent / "hive.sqlite3", scope / "generations" / ("c" * 64) / "index.sqlite3", self.root / "outside"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                WORKER.database_path(runtime, str(path), True)

    def test_hardlink_cannot_modify_a_published_or_foreign_database(self):
        base = self.root / ".hive/index/vector"
        runtime = base / "runtimes" / ("a" * 64)
        scope = base / "scopes" / ("b" * 64)
        scope.mkdir(parents=True)
        original = self.root / "foreign.sqlite3"
        original.write_bytes(b"foreign database bytes")
        stage = scope / "staging.sqlite3"
        try:
            os.link(original, stage)
        except OSError as error:
            self.skipTest(f"hardlinks unavailable on this filesystem: {error}")
        with self.assertRaises(ValueError):
            WORKER.database_path(runtime, str(stage), True)
        source = self.root / "artifact"
        source.write_bytes(b"replacement")
        with self.assertRaises(FileExistsError):
            RUNTIME.copy_exclusive(source, stage)
        self.assertEqual(original.read_bytes(), b"foreign database bytes")

    def test_checkpoint_resume_reuses_vectors_and_rejects_corrupt_cache(self):
        base = self.root / ".hive/index/vector"
        runtime = base / "runtimes" / ("a" * 64)
        (runtime / "site").mkdir(parents=True)
        scope = base / "scopes" / ("b" * 64)
        scope.mkdir(parents=True)
        database = scope / "staging.sqlite3"
        chunks = [{"chunk_id": f"chunk-{index}", "digest": "sha256:" + "c" * 64, "title": "Example", "text": f"synthetic text {index}"} for index in range(129)]
        request = {"runtime": str(runtime), "database": str(database), "chunks": chunks, "contract_digest": "sha256:" + "d" * 64, "manifest_digest": "sha256:" + "e" * 64, "workers": 1, "max_seconds": 1, "expected_database_digest": None}
        connect = lambda path: sqlite3.connect(path, factory=SimulatedVectorConnection)
        with patch.object(WORKER, "load_runtime", return_value=runtime), patch.object(WORKER, "initialize_encoder") as initialize, patch.object(WORKER, "open_database", side_effect=connect), patch.object(WORKER.concurrent.futures, "ThreadPoolExecutor", InlinePool):
            with patch.object(WORKER.time, "monotonic", side_effect=[0.0, 0.0, 2.0, 2.0]):
                first = WORKER.build(request)
            self.assertFalse(first["complete"])
            self.assertEqual(first["embedded"], 64)
            self.assertEqual(initialize.call_count, 1)
            request["expected_database_digest"] = first["database_digest"]
            with patch.object(WORKER.time, "monotonic", return_value=0.0):
                second = WORKER.build(request)
            self.assertTrue(second["complete"])
            self.assertEqual(second["embedded"], 65)
            self.assertEqual(initialize.call_count, 2)
            request["expected_database_digest"] = second["database_digest"]
            unchanged = WORKER.build(request)
            self.assertEqual(unchanged["embedded"], 0)
            self.assertEqual(initialize.call_count, 2)
            self.assertEqual(unchanged["database_digest"], second["database_digest"])
            request["chunks"][0] = {**chunks[0], "text":"updated synthetic text", "digest":"sha256:"+"f"*64}
            updated = WORKER.build(request)
            self.assertTrue(updated["complete"])
            self.assertEqual(updated["embedded"], 1)
            request["expected_database_digest"] = updated["database_digest"]
            with sqlite3.connect(database) as connection:
                self.assertEqual(connection.execute("SELECT count(*) FROM documents").fetchone()[0], 129)
                self.assertEqual(connection.execute("SELECT count(*) FROM cache").fetchone()[0], 0)
                connection.execute("UPDATE vectors SET embedding=? WHERE rowid=1", (b"\x00" * (384 * 4),))
            connection.close()
            with patch.object(WORKER.time, "monotonic", return_value=0.0), self.assertRaises(ValueError):
                WORKER.build(request)

    def test_finalization_resumes_from_a_trusted_cursor(self):
        database = self.root / "finalize.sqlite3"
        connection = sqlite3.connect(database, factory=SimulatedVectorConnection)
        try:
            connection.execute("PRAGMA auto_vacuum=INCREMENTAL")
            connection.execute("CREATE TABLE cache(contract TEXT,digest TEXT,vector BLOB,checksum TEXT,PRIMARY KEY(contract,digest))")
            connection.execute("CREATE TABLE meta(key TEXT PRIMARY KEY,value TEXT)")
            chunks = [{"chunk_id": f"chunk-{index:04d}", "digest": "sha256:" + "a" * 64, "title": "Example", "text": f"row {index}"} for index in range(600)]
            contract = "sha256:" + "b" * 64
            vector = struct.pack("<384f", 1.0, *([0.0] * 383))
            connection.executemany("INSERT INTO cache VALUES (?,?,?,?)", [(contract, WORKER.record_digest(row), vector, hashlib.sha256(vector).hexdigest()) for row in chunks])
            request = {"contract_digest": contract, "manifest_digest": "sha256:" + "c" * 64}
            with patch.object(WORKER.time, "monotonic", side_effect=[0.0, 31.0]):
                self.assertFalse(WORKER.finalize(connection, request, chunks, 30.0))
            self.assertEqual(connection.execute("SELECT count(*) FROM documents").fetchone()[0], 512)
            with patch.object(WORKER.time, "monotonic", return_value=0.0):
                self.assertTrue(WORKER.finalize(connection, request, chunks, 30.0))
            self.assertEqual(connection.execute("SELECT count(*) FROM documents").fetchone()[0], 600)
            self.assertEqual(connection.execute("SELECT value FROM meta WHERE key='phase'").fetchone()[0], "ready")
            self.assertEqual(connection.execute("SELECT count(*) FROM cache").fetchone()[0], 0)
            with patch.object(WORKER.time, "monotonic", side_effect=[0.0, 31.0]):
                self.assertFalse(WORKER.restore_embedding_cache(connection, contract, 30.0))
            self.assertEqual(connection.execute("SELECT count(*) FROM cache").fetchone()[0], 512)
            with patch.object(WORKER.time, "monotonic", return_value=0.0):
                self.assertTrue(WORKER.restore_embedding_cache(connection, contract, 30.0))
            self.assertEqual(connection.execute("SELECT count(*) FROM cache").fetchone()[0], 600)
            with patch.object(WORKER.time, "monotonic", return_value=0.0):
                self.assertTrue(WORKER.finalize(connection, request, chunks, 30.0))
            connection.execute("CREATE TABLE garbage(data BLOB)")
            connection.executemany("INSERT INTO garbage VALUES (?)", [(b"x"*4096,)]*300)
            connection.execute("DROP TABLE garbage")
            connection.execute("UPDATE meta SET value='compacting' WHERE key='phase'")
            connection.commit()
            with patch.object(WORKER.time, "monotonic", return_value=31.0):
                self.assertFalse(WORKER.compact_generation(connection, 30.0))
            self.assertEqual(connection.execute("SELECT value FROM meta WHERE key='phase'").fetchone()[0], "compacting")
            with patch.object(WORKER.time, "monotonic", return_value=0.0):
                self.assertTrue(WORKER.compact_generation(connection, 30.0))
            self.assertEqual(connection.execute("SELECT count(*) FROM documents").fetchone()[0], 600)
        finally:
            connection.close()

    def test_multi_partition_query_embeds_once_and_rejects_corrupt_or_duplicate_inputs(self):
        base = self.root / ".hive/index/vector"
        runtime = base / "runtimes" / ("a"*64)
        contract = "sha256:" + "d"*64
        manifest = "sha256:" + "e"*64
        databases = []
        for name in ("b", "c"):
            path = base / "scopes" / (name*64) / "generations" / ("f"*64) / "index.sqlite3"
            path.parent.mkdir(parents=True)
            path.write_bytes(b"synthetic immutable database")
            databases.append({"database":str(path),"manifest_digest":manifest,"expected_database_digest":WORKER.database_digest(path)})
        class Rows:
            def __init__(self, rows): self.rows = rows
            def __iter__(self): return iter(self.rows)
            def fetchall(self): return self.rows
        class QueryConnection:
            def __init__(self, path): self.name = path.parts[-4][0]
            def execute(self, sql, _parameters=()):
                if sql.startswith("SELECT key,value"):
                    return Rows({"schema_version":"1","phase":"ready","contract_digest":contract,"manifest_digest":manifest}.items())
                return Rows([(self.name,"sha256:"+"1"*64,1.0 if self.name=="b" else .1)])
            def close(self): pass
        def connect(path, readonly=False):
            self.assertTrue(readonly)
            return QueryConnection(path)
        request = {"runtime":str(runtime),"query":"meaning search","limit":1,"contract_digest":contract,"databases":databases}
        with patch.object(WORKER,"load_runtime",return_value=runtime), patch.object(WORKER,"initialize_encoder") as initialize, patch.object(WORKER,"encode_batch",return_value=[b"vector"]) as encode, patch.object(WORKER,"open_database",side_effect=connect):
            result = WORKER.query_many(request)
            self.assertEqual([hit["chunk_id"] for hit in result["matches"]],["c"])
            initialize.assert_called_once()
            encode.assert_called_once()
            with self.assertRaises(ValueError):
                WORKER.query_many({**request,"databases":[databases[0],databases[0]]})
            Path(databases[1]["database"]).write_bytes(b"untrusted replacement")
            with self.assertRaises(ValueError):
                WORKER.query_many(request)


if __name__ == "__main__":
    unittest.main()
