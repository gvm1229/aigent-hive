"""Contract-test view of a directive and its reachable owned references, not an agent load order."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def directive_text(relative):
    boundary = (ROOT / ".agents/directives").resolve()
    seen = set()

    def visit(path):
        path = path.resolve()
        if path in seen or not path.is_relative_to(boundary):
            return ""
        seen.add(path)
        text = path.read_text(encoding="utf-8")
        related = []
        for target in re.findall(r"\]\(([^)]+)\)", text):
            file = target.split("#", 1)[0]
            if file and ":" not in file and file.endswith(".md"):
                related.append(visit(path.parent / file))
        return text + "\n" + "\n".join(related)

    return visit(ROOT / relative)
