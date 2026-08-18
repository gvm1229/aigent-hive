"""Release-range coverage checks for frozen full project bases."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CHECKER = ROOT / "scripts/check-project-base-coverage.py"


class ProjectBaseCoverageContract(unittest.TestCase):
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
                    }
                ],
            }
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("without a full project base", result.stderr)


if __name__ == "__main__":
    unittest.main()
