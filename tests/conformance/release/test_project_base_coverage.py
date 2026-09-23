"""Release-range coverage checks for frozen full project bases."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
CHECKER = ROOT / "scripts/check-project-base-coverage.py"


class ProjectBaseCoverageContract(unittest.TestCase):
    def test_candidate_and_public_acceptance_require_all_predecessors(self) -> None:
        import yaml
        workflow = yaml.safe_load((ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8"))
        jobs = workflow["jobs"]
        for platform in ("unix", "windows"):
            self.assertEqual(jobs[platform]["needs"], "predecessor-recovery")
            self.assertIn("scripts/qualify-project-predecessors.py", str(jobs[platform]["steps"]))
        self.assertIn("--failure-tests", str(jobs["predecessor-recovery"]["steps"]))
        public = (ROOT / ".github/workflows/public-test-acceptance.yml").read_text(encoding="utf-8")
        self.assertIn("test_public_all_stable_predecessors_preserve_and_upgrade", public)

    def test_missing_duplicate_future_and_cross_major_sources_fail_closed(self) -> None:
        spec = importlib.util.spec_from_file_location("coverage_checker", CHECKER)
        checker = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(checker)
        sources = ["0.9.1", "0.9.2", "0.10.3", "0.12.0"]
        route = {"route_id": "all", "kind": "cross-major", "from_min": "0.9.1",
                 "from_max": "0.12.0", "to_version": "1.0.0"}
        table = {"target_version": "1.0.0", "routes": [route]}
        checker.validate_complete_routes(table, sources)
        with self.assertRaises(ValueError):
            checker.validate_complete_routes({**table, "routes": [{k: v for k, v in route.items() if k != "to_version"}]}, sources)
        for change in ({"from_min": "0.9.2"}, {"from_max": "0.10.3"},
                       {"to_version": "0.12.0"}, {"kind": "same-major"},
                       {"from_max": "1.0.0"}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                checker.validate_complete_routes({**table, "routes": [{**route, **change}]}, sources)
        with self.assertRaises(ValueError):
            checker.validate_complete_routes({**table, "routes": [route, route]}, sources)
        with self.assertRaises(ValueError):
            checker.validate_complete_routes({**table, "routes": [
                {**route, "from_max": "0.9.1"}, {**route, "from_min": "0.10.3"}]}, sources)

    def run_checker(self, table: dict[str, object]) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as temporary:
            work = Path(temporary)
            migration = work / "migration-table.json"
            report = work / "coverage.json"
            migration.write_text(json.dumps(table), encoding="utf-8")
            result = subprocess.run(
                ["python", str(CHECKER), "--migration-table", str(migration), "--output", str(report)],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            if result.returncode == 0:
                result.coverage = json.loads(report.read_text(encoding="utf-8"))  # type: ignore[attr-defined]
            return result

    def test_full_range_report_is_digest_bound_and_includes_each_frozen_source(self) -> None:
        result = self.run_checker(
            {
                "schema_version": 1,
                "target_version": "0.9.5",
                "routes": [
                    {
                        "route_id": "same-major-0-9",
                        "kind": "same-major",
                        "from_min": "0.9.1",
                        "from_max": "0.9.4",
                        "to_version": "0.9.5",
                    }
                ],
            }
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        coverage = result.coverage["coverage"][0]["sources"]  # type: ignore[attr-defined,index]
        self.assertEqual(
            [item["version"] for item in coverage],
            ["0.9.1", "0.9.2", "0.9.3", "0.9.4"],
        )
        self.assertTrue(result.coverage["coverage_digest"].startswith("sha256:"))  # type: ignore[attr-defined,index]

    def test_declared_legacy_source_without_a_full_base_fails_closed(self) -> None:
        result = self.run_checker(
            {
                "schema_version": 1,
                "target_version": "0.9.5",
                "routes": [
                    {
                        "route_id": "unsafe-legacy-range",
                        "kind": "same-major",
                        "from_min": "0.1.0",
                        "from_max": "0.9.4",
                        "to_version": "0.9.5",
                    }
                ],
            }
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("without a full project base", result.stderr)


if __name__ == "__main__":
    unittest.main()
