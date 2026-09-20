"""One read-only interpretation of the registered Markdown implementation plan."""

from __future__ import annotations

from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tempfile

PLAN_ID = re.compile(r"[A-Z][A-Z0-9]*-[0-9]{3}")
VERSION = re.compile(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)")
STATES = ("complete", "agent-owned", "awaiting-user-authority", "awaiting-external-evidence", "blocked")
SCOPES = ("product", "source", "release")
START = "<!-- HIVE:PLAN-STATE:START -->"
END = "<!-- HIVE:PLAN-STATE:END -->"
MAX_PLAN_BYTES = 8192


class PlanError(ValueError):
    """Invalid or stale canonical plan state; no guessed fallback."""


@dataclass(frozen=True)
class Criterion:
    id: str
    title: str
    state: str
    dependencies: tuple[str, ...]
    evidence: str | None
    owner: str | None
    reason: str | None
    fragment: str
    scope: str


@dataclass
class Plan:
    root: Path
    version: str
    groups: list[tuple[str, str]]
    criteria: dict[str, Criterion]
    inputs: dict[Path, bytes]

    def completed_product_ids(self) -> set[str]:
        return {item.id for item in self.criteria.values()
                if item.state == "complete" and item.scope == "product"}


def confined(root: Path, relative: str) -> Path:
    lexical = PurePosixPath(relative)
    if not relative or lexical.is_absolute() or "\\" in relative or ":" in relative:
        raise PlanError(f"unsafe plan path: {relative}")
    if any(part in ("", ".", "..") for part in relative.split("/")):
        raise PlanError(f"unsafe plan path: {relative}")
    target = root.joinpath(*lexical.parts)
    for candidate in (root, *[root.joinpath(*lexical.parts[:n]) for n in range(1, len(lexical.parts) + 1)]):
        if not candidate.exists() and not candidate.is_symlink():
            raise PlanError(f"missing plan input: {relative}")
        attributes = getattr(candidate.lstat(), "st_file_attributes", 0)
        if candidate.is_symlink() or attributes & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0):
            raise PlanError(f"symlink in plan input: {relative}")
    if not target.is_file():
        raise PlanError(f"missing plan input: {relative}")
    return target


def read(root: Path, relative: str, inputs: dict[Path, bytes], *, limit=MAX_PLAN_BYTES) -> str:
    path = confined(root, relative)
    payload = path.read_bytes()
    if len(payload) >= limit:
        raise PlanError(f"oversized plan input: {relative}")
    if path in inputs and inputs[path] != payload:
        raise PlanError(f"plan input changed while reading: {relative}")
    inputs[path] = payload
    try:
        return payload.decode("utf-8")
    except UnicodeError as error:
        raise PlanError(f"invalid UTF-8: {relative}") from error


def field(text: str, name: str) -> str:
    values = re.findall(r"^> " + re.escape(name) + r":\s*`?([^`\r\n]+?)`?\s*$", text, re.M)
    if len(values) != 1:
        raise PlanError(f"expected exactly one {name} field")
    return values[0]


def evidence_input(root: Path, locator: str, inputs: dict[Path, bytes]) -> None:
    match = re.fullmatch(r"repo:([^#]+)#sha256:([0-9a-f]{64})", locator)
    if match is None or not match[1].startswith(("docs/", "tests/results/")):
        raise PlanError("completion evidence must be a bounded repository Markdown locator")
    if not match[1].endswith(".md"):
        raise PlanError("completion evidence must be Markdown")
    text = read(root, match[1], inputs, limit=1024 * 1024)
    if hashlib.sha256(inputs[root / match[1]]).hexdigest() != match[2]:
        raise PlanError(f"stale completion evidence: {match[1]}")
    if match[1].startswith("tests/results/runs/"):
        report = re.search(r"```json\s*\n(.*?)\n```", text, re.S)
        try:
            payload = json.loads(report[1]) if report else {}
        except json.JSONDecodeError as error:
            raise PlanError("malformed test evidence") from error
        if (not isinstance(payload, dict) or payload.get("status") != "passed"
                or type(payload.get("exit_code")) is not int or payload["exit_code"] != 0
                or payload.get("source_changed_during_run") is True):
            raise PlanError(f"test evidence did not pass: {match[1]}")


def metadata(line: str) -> dict[str, str]:
    if not line.startswith("  - state: "):
        raise PlanError("criterion requires an adjacent state metadata line")
    result = {}
    for part in line[4:].split("; "):
        key, separator, value = part.partition(": ")
        if not separator or not value or key in result:
            raise PlanError("malformed or duplicate criterion metadata")
        if key not in ("state", "depends", "evidence", "owner", "reason"):
            raise PlanError(f"unknown criterion metadata: {key}")
        result[key] = value
    return result


def load_plan(root: Path) -> Plan:
    root = root.absolute()
    inputs = {}
    text = read(root, "docs/plans/PLAN.md", inputs)
    version = field(text, "Product version")
    if not VERSION.fullmatch(version):
        raise PlanError("invalid product version in plan")
    if re.search(r"^- \[[ x]\]", text, re.M):
        raise PlanError("PLAN.md must not own checkboxes")
    sections = re.findall(r"^## Active fragments\s*\n(.*?)(?=^## |\Z)", text, re.M | re.S)
    if len(sections) != 1:
        raise PlanError("missing or duplicate Active fragments section")
    groups = []
    for line in sections[0].splitlines():
        if not line.startswith("|"):
            continue
        columns = [column.strip() for column in line.strip("|").split("|")]
        links = re.findall(r"\]\(([^)]+)\)", columns[0])
        if not links:
            if columns[0] not in ("Fragment", "---"):
                raise PlanError("unlinked fragment row")
            continue
        if len(columns) != 3 or len(links) != 1 or not re.fullmatch(r"active/[a-z0-9][a-z0-9.-]*\.md", links[0]):
            raise PlanError("invalid active fragment registration")
        relative = "docs/plans/" + links[0]
        if relative in [path for path, _ in groups]:
            raise PlanError("duplicate active fragment registration")
        groups.append((relative, columns[2]))
    if not groups:
        raise PlanError("no active fragments registered")
    criteria = {}
    for relative, _label in groups:
        fragment = read(root, relative, inputs)
        if field(fragment, "Plan version") != version:
            raise PlanError(f"fragment version mismatch: {relative}")
        scope = field(fragment, "Scope")
        if scope not in SCOPES:
            raise PlanError(f"unsupported fragment scope: {scope}")
        lines = fragment.splitlines()
        count = 0
        metadata_lines = set()
        for index, line in enumerate(lines):
            if not re.match(r"^- \[[^]]?\]\s", line):
                continue
            match = re.fullmatch(r"- \[([ x])\] \[([^]]+)\] (.+)", line)
            if not match or not PLAN_ID.fullmatch(match[2]):
                raise PlanError(f"invalid criterion ID or checkbox: {relative}")
            ident = match[2]
            if ident in criteria:
                raise PlanError(f"duplicate criterion: {ident}")
            data = metadata(lines[index + 1] if index + 1 < len(lines) else "")
            metadata_lines.add(index + 1)
            state = data["state"]
            if state not in STATES or ((match[1] == "x") != (state == "complete")):
                raise PlanError(f"checkbox/state mismatch: {ident}")
            depends = tuple(data.get("depends", "").split(",")) if data.get("depends") else ()
            if len(depends) != len(set(depends)) or any(not PLAN_ID.fullmatch(item) for item in depends):
                raise PlanError(f"invalid dependencies: {ident}")
            if state in STATES[2:] and not (data.get("owner") and data.get("reason")):
                raise PlanError(f"waiting criterion requires owner and reason: {ident}")
            if state == "complete":
                if not data.get("evidence"):
                    raise PlanError(f"complete criterion lacks evidence: {ident}")
                evidence_input(root, data["evidence"], inputs)
            criteria[ident] = Criterion(ident, match[3], state, depends, data.get("evidence"),
                                       data.get("owner"), data.get("reason"), relative, scope)
            count += 1
        if not count:
            raise PlanError(f"registered fragment owns no criteria: {relative}")
        if any(line.startswith("  - state:") and index not in metadata_lines for index, line in enumerate(lines)):
            raise PlanError(f"orphan criterion metadata: {relative}")
    visited, visiting = set(), set()

    def visit(ident):
        if ident in visiting:
            raise PlanError(f"dependency cycle: {ident}")
        if ident in visited:
            return
        visiting.add(ident)
        for dependency in criteria[ident].dependencies:
            if dependency not in criteria:
                raise PlanError(f"unknown dependency: {ident} -> {dependency}")
            if criteria[ident].state == "complete" and criteria[dependency].state != "complete":
                raise PlanError(f"completed criterion has incomplete dependency: {ident}")
            visit(dependency)
        visiting.remove(ident)
        visited.add(ident)

    for ident in criteria:
        visit(ident)
    return Plan(root, version, groups, criteria, inputs)


def summaries(plan: Plan) -> tuple[str, str]:
    lines = ["| 범위 | 완료 | 미완료 | 진행률 |", "| --- | ---: | ---: | ---: |"]
    for path, label in plan.groups:
        items = [item for item in plan.criteria.values() if item.fragment == path]
        done = sum(item.state == "complete" for item in items)
        lines.append(f"| {label} | {done} | {len(items)-done} | {done/len(items)*100:.1f}% |")
    count = len(plan.criteria)
    done = sum(item.state == "complete" for item in plan.criteria.values())
    lines.append(f"| **현재 범위 합계** | **{done}** | **{count-done}** | **{done/count*100:.1f}%** |")
    current = [f"- 구현 목표: `{plan.version}`", f"- 현재 등록 항목: {done}/{count} 완료", ""]
    for state in STATES[1:]:
        items = [item.id for item in plan.criteria.values() if item.state == state]
        current.append(f"- `{state}`: " + (", ".join(f"`{item}`" for item in items) or "없음"))
    return "\n".join(lines) + "\n", "\n".join(current) + "\n"


def replace_section(original: bytes, body: str) -> bytes:
    start, end = START.encode(), END.encode()
    if original.count(start) != 1 or original.count(end) != 1:
        raise PlanError("expected exactly one generated-section marker pair")
    match = re.search(re.escape(start) + rb"(\r?\n)", original)
    end_at = original.index(end)
    if not match or match.end() > end_at:
        raise PlanError("invalid generated-section marker order")
    replacement = body.encode().replace(b"\n", match[1])
    result = original[:match.end()] + replacement + original[end_at:]
    if len(result) >= MAX_PLAN_BYTES:
        raise PlanError("generated document exceeds 8KiB")
    return result


def render(plan: Plan, *, write: bool = False) -> list[str]:
    bodies = summaries(plan)
    paths = ("docs/plans/PLAN.md", "docs/state/CURRENT.md")
    changes = []
    for relative, body in zip(paths, bodies):
        read(plan.root, relative, plan.inputs)
        path = plan.root / relative
        desired = replace_section(plan.inputs[path], body)
        if desired != plan.inputs[path]:
            changes.append((relative, path, desired))
    if write:
        for relative, path, desired in changes:
            for observed, expected in plan.inputs.items():
                if confined(plan.root, observed.relative_to(plan.root).as_posix()).read_bytes() != expected:
                    raise PlanError("plan input changed during generation")
            temporary = None
            try:
                with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".hive-plan-", delete=False) as stream:
                    temporary = Path(stream.name)
                    stream.write(desired)
                    stream.flush()
                    os.fsync(stream.fileno())
                os.chmod(temporary, stat.S_IMODE(path.stat().st_mode))
                if path.read_bytes() != plan.inputs[path]:
                    raise PlanError("generated target changed before replacement")
                os.replace(temporary, path)
                plan.inputs[path] = desired
            finally:
                if temporary is not None and temporary.exists():
                    temporary.unlink()
    return [relative for relative, _, _ in changes]
