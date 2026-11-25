"""Keep public acceptance jobs active across numbered test versions."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = ROOT / ".github/workflows/public-test-acceptance.yml"


class AcceptanceWorkflowVersionPinTests(unittest.TestCase):
    def test_version_pins_have_an_adjacent_reason(self):
        lines = WORKFLOW.read_text(encoding="utf-8").splitlines()
        for index, line in enumerate(lines):
            if "inputs.product_version ==" not in line and "startsWith(inputs.product_version" not in line:
                continue
            previous = lines[index - 1].strip() if index else ""
            self.assertTrue(
                previous.startswith("# version-pin-allowed:"),
                f"unexplained product version pin at {WORKFLOW}:{index + 1}",
            )


if __name__ == "__main__":
    unittest.main()
