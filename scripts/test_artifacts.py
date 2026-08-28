"""Source-only test evidence and conservative artifact lifecycle management.

No age-based deletion: a completed, explicitly owned record and committed evidence
are required. Unknown processes, paths and stale leases fail closed.
"""
from __future__ import annotations

import contextlib
from datetime import datetime, timedelta, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
UTC = timezone.utc
MAX_REUSE = timedelta(hours=72)


class ArtifactError(RuntimeError):
    """An operation needs review; no unsafe fallback is permitted."""


def now():
    return datetime.now(UTC)


def stamp():
    return now().isoformat()


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def linked(path):
    info = path.lstat()
    return stat.S_ISLNK(info.st_mode) or bool(
        getattr(info, "st_file_attributes", 0) & 0x400
    )


def confined(root, relative, prefixes):
    """Reject aliases before resolution, including Windows junction ancestors."""
    relative = str(relative).replace("\\", "/")
    parts = relative.split("/")
    if not relative or any(p in ("", ".", "..") or ":" in p for p in parts):
        raise ArtifactError("invalid relative path")
    if not any(relative == p or relative.startswith(p + "/") for p in prefixes):
        raise ArtifactError("path outside declared ownership")
    root = Path(root).resolve(strict=True)
    path = root
    for part in parts:
        path = path / part
        if path.exists() or path.is_symlink():
            if linked(path):
                raise ArtifactError("linked path refused: " + relative)
    if not path.resolve().is_relative_to(root):
        raise ArtifactError("path escaped repository")
    return path


def atomic(path, content):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        stream.write(content.encode("utf-8"))
        stream.flush()
        os.fsync(stream.fileno())
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def clean_text(value, root=ROOT):
    """Do not export local identities or token-shaped values into tracked evidence."""
    value = str(value).replace(str(root).replace("\\", "\\\\"), "<repository>").replace(str(root), "<repository>").replace(str(root).replace("\\", "/"), "<repository>")
    value = value.replace(str(Path.home()), "<user>").replace(str(Path.home()).replace("\\", "/"), "<user>")
    value = re.sub(r"(?i)[A-Z]:[\\/]+(?:Users|Documents and Settings)[\\/]+[^\s\"'<>]+", "<private-path>", value)
    value = re.sub(r"/(?:Users|home)/[^\s\"<>]+", "<private-path>", value)
    value = re.sub(r'''(?i)((?:token|password|secret|authorization|api[-_]key)["']?\s*[=:]\s*["']?)[^\s,;"']+''', r"\1<redacted>", value)
    value = re.sub(r'''(?i)(Bearer\s+)[^\s"']+''', r"\1<redacted>", value)
    value = re.sub(r"(?:gh[pousr]_[A-Za-z0-9_]{20,}|github_pat_[A-Za-z0-9_]+|sk-[A-Za-z0-9_-]{20,})", "<redacted>", value)
    return value


def scrub(value, root=ROOT):
    if isinstance(value, dict):
        return {clean_text(k, root): ("<redacted>" if re.search(r"(?i)token|password|secret|authorization_token|session_id", str(k)) else scrub(v, root)) for k, v in value.items()}
    if isinstance(value, list):
        result, sensitive = [], False
        for item in value:
            result.append("<redacted>" if sensitive else scrub(item, root))
            sensitive = isinstance(item, str) and bool(re.fullmatch(r"--(?:authorization-)?token|--password|--secret|--api-key", item, re.I))
        return result
    return clean_text(value, root) if isinstance(value, str) else value


def archive_legacy(root, source_relative, *, purpose, limit):
    """Explicitly selected JSON evidence, not a recursive harvest of fixture content."""
    manager = Manager(root)
    source = confined(manager.root, source_relative, ("tests/work",))
    if not source.is_file() or source.suffix != ".json" or source.stat().st_size > 4 * 1024 * 1024:
        raise ArtifactError("select one small JSON result")
    original_digest = sha(source)
    value = json.loads(source.read_text(encoding="utf-8"))
    identity = hashlib.sha256((source_relative + original_digest).encode()).hexdigest()[:20]
    folder = f"tests/results/legacy/{identity}"
    attachment = confined(manager.root, folder + "/receipt.json", ("tests/results",))
    report = confined(manager.root, folder + ".md", ("tests/results",))
    with manager.lock():
        atomic(attachment, json.dumps(scrub(value, manager.root), ensure_ascii=False, indent=2) + "\n")
        metadata = {"purpose": purpose, "original_locator": source_relative, "original_sha256": original_digest,
                    "archived_at": stamp(), "archive_host": platform.platform(),
                    "result": value.get("status", "not specified in original") if isinstance(value, dict) else "not specified in original",
                    "source_commit": value.get("source_commit", "not specified in original") if isinstance(value, dict) else "not specified in original",
                    "attachments": [{"path": attachment.relative_to(manager.root).as_posix(), "sha256": sha(attachment)}]}
        text = "# 과거 시험 결과 보존\n\n```json\n" + json.dumps(scrub(metadata, manager.root), ensure_ascii=False, indent=2) + "\n```\n\n"
        text += "## 증명 범위와 한계\n\n- " + clean_text(limit, manager.root) + "\n- 기존 JSON의 값 보존, 현재 코드 재실행·재검증 근거에서 제외\n- 원본에 없는 실행 시각·명령·소스·통과 수치 추정 없음\n- 개인 경로·비밀 값 치환으로 원본 전체 바이트와 보존 파일 지문 구분\n"
        text += f"\n[보존 JSON]({identity}/receipt.json)\n"
        atomic(report, text)
    return report.relative_to(manager.root).as_posix()


def result_index(root=ROOT):
    manager = Manager(root)
    base = confined(manager.root, "tests/results", ("tests/results",))
    lines = ["# 시험 결과 색인", "", "| 결과 | 목적·원본 | 소스 | 실행 환경 |", "| --- | --- | --- | --- |"]
    for group in ("legacy", "runs"):
        for path in sorted((base / group).glob("*.md")):
            confined(manager.root, path.relative_to(manager.root).as_posix(), ("tests/results",))
            match = re.search(r"```json\n(.*?)\n```", path.read_text(encoding="utf-8"), re.S)
            if not match:
                continue
            record = json.loads(match.group(1))
            label = record.get("status", record.get("result", "unknown"))
            purpose = record.get("original_locator", record.get("purpose", ""))
            source = record.get("source_commit", "unknown")
            environment = record.get("platform", "원본 JSON 참조")
            cells = [label, purpose, source, environment]
            cells = [str(c).replace("|", "\\|").replace("\n", " ") for c in cells]
            lines.append(f"| [{cells[0]}]({path.relative_to(base).as_posix()}) | {cells[1]} | {cells[2]} | {cells[3]} |")
    with manager.lock():
        destination = confined(manager.root, "tests/results/INDEX.md", ("tests/results",))
        atomic(destination, "\n".join(lines) + "\n")


def processes():
    """Return process identity and command data only in memory; never archive it."""
    if os.name == "nt":
        command = ["pwsh", "-NoProfile", "-NonInteractive", "-Command",
                   "@(Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,Name,CreationDate,CommandLine,ExecutablePath) | ConvertTo-Json -Compress"]
        result = subprocess.run(command, capture_output=True, text=True, check=True, timeout=20)
        entries = json.loads(result.stdout)
        return [{"pid": p["ProcessId"], "parent": p["ParentProcessId"], "name": p["Name"],
                 "start": str(p["CreationDate"]), "command": p.get("CommandLine") or "",
                 "image": p.get("ExecutablePath") or ""} for p in entries]
    result = subprocess.run(["ps", "-eo", "pid=,ppid=,lstart=,args="], capture_output=True, text=True, check=True, timeout=20)
    entries = []
    for line in result.stdout.splitlines():
        fields = line.split(None, 7)
        if len(fields) == 8:
            entries.append({"pid": int(fields[0]), "parent": int(fields[1]), "start": " ".join(fields[2:7]),
                            "name": Path(fields[7].split()[0]).name, "command": fields[7], "image": ""})
    return entries


def overlap(left, right):
    a, b = left.casefold().rstrip("/"), right.casefold().rstrip("/")
    return a == b or a.startswith(b + "/") or b.startswith(a + "/")


class Manager:
    def __init__(self, root=ROOT):
        self.root = Path(root).resolve(strict=True)
        if not (self.root / "hive-source.json").is_file():
            raise ArtifactError("source marker required")
        self.state = confined(self.root, ".agents/work/test-artifacts", (".agents/work/test-artifacts",))

    @contextlib.contextmanager
    def lock(self):
        # OS lock is released on process death. Never unlink a held lock file.
        self.state.mkdir(parents=True, exist_ok=True)
        path = confined(self.root, ".agents/work/test-artifacts/lock", (".agents/work/test-artifacts",))
        with path.open("a+b") as stream:
            if stream.tell() == 0:
                stream.write(b"0")
                stream.flush()
            stream.seek(0)
            try:
                if os.name == "nt":
                    import msvcrt
                    msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
                else:
                    import fcntl
                    fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError as error:
                raise ArtifactError("artifact operation already in progress") from error
            try:
                yield
            finally:
                stream.seek(0)
                if os.name == "nt":
                    msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)
                else:
                    fcntl.flock(stream, fcntl.LOCK_UN)

    def records(self):
        if not self.state.exists():
            return []
        result = []
        for path in sorted(self.state.glob("*.json")):
            if linked(path):
                raise ArtifactError("linked lifecycle record")
            record = json.loads(path.read_text(encoding="utf-8"))
            if record.get("id") != path.stem:
                raise ArtifactError("record identity mismatch")
            result.append(record)
        return result

    def save(self, record):
        if not re.fullmatch(r"[a-zA-Z0-9_-]+", record["id"]):
            raise ArtifactError("invalid record ID")
        path = confined(self.root, f".agents/work/test-artifacts/{record['id']}.json", (".agents/work/test-artifacts",))
        atomic(path, json.dumps(record, ensure_ascii=False, indent=2) + "\n")

    def target(self, relative, *, reservation=False):
        if str(relative).replace("\\", "/") == "tests/work" and not reservation:
            raise ArtifactError("work root deletion refused")
        return confined(self.root, relative, ("tests/work", "target/debug"))

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.root, capture_output=True, check=True).stdout

    def untracked_source_digest(self):
        digest = hashlib.sha256()
        for name in sorted(self.git("ls-files", "--others", "--exclude-standard", "-z").decode().split("\0")):
            if not name or name.startswith(("tests/results/", "tests/work/", "target/", ".agents/work/", ".hive/", ".omx/", ".omc/", ".codex/", ".claude/")):
                continue
            path = confined(self.root, name, (name,))
            if not path.is_file():
                raise ArtifactError("untracked source is not a regular file")
            digest.update(name.encode() + b"\0" + sha(path).encode() + b"\0")
        return digest.hexdigest()

    def evidence(self, relative, expected=None, committed=True):
        path = confined(self.root, relative, ("tests/results",))
        if path.suffix != ".md" or not path.is_file():
            raise ArtifactError("Markdown evidence required")
        digest = sha(path)
        if expected and digest != expected:
            raise ArtifactError("evidence changed after review")
        if committed:
            try:
                # Compare Git's canonical bytes (including configured EOL conversion).
                if self.git("rev-parse", "HEAD:" + relative).strip() != self.git("hash-object", "--path=" + relative, str(path)).strip():
                    raise ArtifactError("evidence differs from committed bytes")
            except subprocess.CalledProcessError as error:
                raise ArtifactError("commit evidence before deletion") from error
        metadata = re.search(r"```json\n(.*?)\n```", path.read_text(encoding="utf-8"), re.S)
        if metadata:
            value = json.loads(metadata.group(1))
            for item in value.get("attachments", []) if isinstance(value, dict) else []:
                attachment = confined(self.root, item["path"], ("tests/results",))
                if sha(attachment) != item["sha256"]:
                    raise ArtifactError("attachment changed after report")
                if committed and self.git("rev-parse", "HEAD:" + item["path"]).strip() != self.git("hash-object", "--path=" + item["path"], str(attachment)).strip():
                    raise ArtifactError("attachment is not committed")
        return digest

    def review(self, request):
        """Explicit operator review; never classify unknown legacy content by age."""
        with self.lock():
            for item in request:
                path = item["path"].replace("\\", "/")
                self.target(path, reservation=item["state"] != "completed")
                if item["state"] not in ("completed", "retained", "review", "released"):
                    raise ArtifactError("invalid review state")
                if not item.get("owner") or not item.get("reason"):
                    raise ArtifactError("owner and reviewed reason required")
                if item["state"] not in ("completed", "released"):
                    due = datetime.fromisoformat(item["review_at"])
                    if not now() < due <= now() + MAX_REUSE:
                        raise ArtifactError("review/reuse deadline must be within 72 hours")
                    if item["state"] == "retained" and not item.get("task"):
                        raise ArtifactError("concrete reuse task required")
                record = {**item, "path": path, "id": "review-" + hashlib.sha256(path.encode()).hexdigest()[:24], "reviewed_at": stamp()}
                if item["state"] == "completed":
                    record["report_sha256"] = self.evidence(item["report"])
                    record["inventory"] = self.inventory(path)
                for previous in self.records():
                    if previous.get("path") != path or previous["id"] == record["id"]:
                        continue
                    if previous.get("state") == "active":
                        if not item.get("resolve_stale"):
                            continue
                        identity = next((p for p in processes() if p["pid"] == previous.get("pid")), None)
                        if identity and identity["start"] == previous.get("process_start"):
                            raise ArtifactError("cannot resolve a live run")
                    previous.update(state="superseded", superseded_by=record["id"])
                    self.save(previous)
                self.save(record)

    def inventory(self, relative):
        path = self.target(relative)
        if not path.exists():
            return {"bytes": 0, "files": 0, "fingerprint": "missing"}
        entries = [path]
        digest, size, count = hashlib.sha256(), 0, 0
        while entries:
            item = entries.pop()
            if linked(item):
                raise ArtifactError("linked descendant refused")
            info = item.stat()
            digest.update(f"{item.relative_to(path.parent).as_posix()}:{info.st_size}:{info.st_mtime_ns}:{info.st_ino}\n".encode())
            if item.is_dir():
                entries.extend(sorted(item.iterdir(), reverse=True))
            elif item.is_file():
                size += info.st_size
                count += 1
            else:
                raise ArtifactError("special file refused")
        return {"bytes": size, "files": count, "fingerprint": digest.hexdigest()}

    def process_block(self, relative, snapshot):
        absolute = str(self.root / relative).replace("\\", "/").casefold()
        admin_ancestors = {os.getpid()}
        cursor = next((p for p in snapshot if p["pid"] == os.getpid()), None)
        while cursor:
            cursor = next((p for p in snapshot if p["pid"] == cursor["parent"]), None)
            if not cursor or cursor["name"].casefold().removesuffix(".exe") not in ("pwsh", "powershell", "cmd", "bash", "sh", "codex", "node"):
                break
            admin_ancestors.add(cursor["pid"])
        for p in snapshot:
            if p["pid"] in admin_ancestors:
                continue
            command = (p["command"] + " " + p["image"]).replace("\\", "/").casefold()
            name = p["name"].casefold().removesuffix(".exe")
            # Null command lines of relevant programs are not proof of absence.
            relevant = name in ("cargo", "rustc", "hive", "python", "python3", "python3.12", "python3.13", "pytest", "cl", "link")
            if relevant and not p["command"]:
                return "process-identity-unavailable"
            if absolute in command or relative.casefold() in command:
                return "live-path-reference"
            if relative.startswith("target/debug") and relevant and (
                name in ("cargo", "rustc", "hive") or "test-lanes.py" in command or "unittest" in command or "dev-check.py" in command
            ):
                return "live-shared-build-consumer"
        return None

    def scan(self, *, sizes=False, selected=None):
        records = self.records()
        snapshot = processes()  # A failed snapshot aborts instead of permitting cleanup.
        paths = set(selected or ())
        if selected is None:
            work = confined(self.root, "tests/work", ("tests/work",))
            if work.exists():
                paths.update(p.relative_to(self.root).as_posix() for p in work.iterdir())
            if (self.root / "target/debug").exists():
                paths.add("target/debug")
            paths.update(r["path"] for r in records if "path" in r and r.get("state") not in ("superseded", "released"))
        result = []
        for path in sorted(paths):
            row = {"path": path, "status": "review", "reason": "unowned-artifact", "bytes": None}
            try:
                target = self.target(path, reservation=path == "tests/work")
                if not target.exists():
                    continue
                owners = [r for r in records if "path" in r and r.get("state") not in ("superseded", "released") and overlap(r["path"], path)]
                blockers = [r for r in owners if r.get("state") != "completed"]
                if blockers:
                    owner = blockers[0]
                    row.update(status=owner["state"], reason=owner.get("reason", "registered-run"), owner=owner.get("owner"), review_at=owner.get("review_at"))
                    if row["status"] == "active":
                        identity = next((p for p in snapshot if p["pid"] == owner.get("pid")), None)
                        if not identity or identity["start"] != owner.get("process_start"):
                            row.update(status="review", reason="stale-or-reused-process-identity")
                    elif owner.get("review_at") and datetime.fromisoformat(owner["review_at"]) <= now():
                        row.update(status="review", reason="expired-review-required")
                else:
                    completed = [r for r in owners if r["path"] == path and r["state"] == "completed"]
                    if completed and path != "tests/work":
                        for record in completed:
                            self.evidence(record["report"], record["report_sha256"])
                        row.update(status="eligible", reason=completed[-1]["reason"])
                block = self.process_block(path, snapshot)
                if block:
                    row.update(status="active", reason=block)
                if (sizes or row["status"] == "eligible") and path != "tests/work":
                    row.update(self.inventory(path))
                if row["status"] == "eligible":
                    if any(r.get("inventory", {}).get("fingerprint") != row["fingerprint"] for r in completed):
                        raise ArtifactError("artifact changed since completion review")
                if row["status"] == "eligible" and self.git("ls-files", "--", path).strip():
                    raise ArtifactError("tracked content refused")
            except (ArtifactError, OSError, ValueError, subprocess.SubprocessError) as error:
                row.update(status="review", reason=str(error))
            result.append(row)
        return result

    def cleanup(self, *, apply=False, selected=None):
        with self.lock():
            rows = self.scan(selected=selected)
            free_before = shutil.disk_usage(self.root).free
            audit = None
            if apply:
                audit_relative = "tests/results/cleanup/" + now().strftime("%Y%m%dT%H%M%S") + "-" + uuid.uuid4().hex[:12] + ".md"
                audit = confined(self.root, audit_relative, ("tests/results",))
                self.last_cleanup_report = audit_relative
                atomic(audit, "# 산출물 정리 시작 기록\n\n```json\n" + json.dumps(scrub({"started_at": stamp(), "status": "in-progress", "rows": rows}, self.root), ensure_ascii=False, indent=2) + "\n```\n\n- 완료 기록 전 삭제 성공 추론 금지\n")
            for row in rows:
                if row["status"] != "eligible" or not apply:
                    continue
                try:
                    # Fresh process/state/evidence checks and inventory immediately before removal.
                    fresh = self.scan(selected=[row["path"]])[0]
                    if fresh["status"] != "eligible" or fresh["fingerprint"] != row["fingerprint"]:
                        raise ArtifactError("artifact changed after preview")
                    self.remove(row["path"])
                    row["status"] = "removed"
                except (ArtifactError, OSError, subprocess.SubprocessError) as error:
                    row.update(status="cleanup-failed", reason=str(error))
            if apply:
                journal = confined(self.root, ".agents/work/test-artifacts/last-cleanup.log", (".agents/work/test-artifacts",))
                result = {"finished_at": stamp(), "platform": platform.platform(), "rows": rows,
                          "removed_logical_bytes": sum(r["bytes"] or 0 for r in rows if r["status"] == "removed"),
                          "drive_free_before": free_before, "drive_free_after": shutil.disk_usage(self.root).free}
                atomic(journal, json.dumps(result, ensure_ascii=False, indent=2) + "\n")
                atomic(audit, "# 산출물 정리 결과\n\n```json\n" + json.dumps(scrub(result, self.root), ensure_ascii=False, indent=2) + "\n```\n\n- 논리적 파일 크기와 드라이브 여유 공간 차이 구분\n- 다른 프로세스의 쓰기·하드 링크·파일시스템 할당에 따른 실제 회수량 차이 가능\n- 삭제 환경의 재생성은 원래 소스·입력·도구 필요, 원시 산출물의 직접 복구 보장 없음\n- 보존된 결과 Markdown은 이전 시험 근거이며 현재 코드 재검증과 구분\n")
            return rows

    def remove(self, relative):
        path = self.target(relative)
        # No symlink/reparse children; no forced process termination or alternate-shell retry.
        self.inventory(relative)
        if os.name == "nt":
            script = r'''$ErrorActionPreference='Stop'
$root=[IO.Path]::GetFullPath($env:HIVE_CLEAN_ROOT)
$target=[IO.Path]::GetFullPath($env:HIVE_CLEAN_TARGET)
$work=[IO.Path]::Combine($root,'tests','work')+[IO.Path]::DirectorySeparatorChar
$debug=[IO.Path]::Combine($root,'target','debug')
if (-not ($target.StartsWith($work,[StringComparison]::OrdinalIgnoreCase) -or $target.Equals($debug,[StringComparison]::OrdinalIgnoreCase) -or $target.StartsWith($debug+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase))) { throw 'unsafe cleanup target' }
$item=Get-Item -LiteralPath $target -Force
$cursor=$item
while ($cursor -and $cursor.FullName -ne $root) {
 if ($cursor.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'linked ancestor' }
 $cursor=Get-Item -LiteralPath ([IO.Path]::GetDirectoryName($cursor.FullName)) -Force
}
if ($item.PSIsContainer) {
 $queue=[Collections.Generic.Stack[string]]::new(); $queue.Push($target)
 while ($queue.Count) { foreach ($child in Get-ChildItem -LiteralPath $queue.Pop() -Force) {
  if ($child.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'linked descendant' }
  if ($child.PSIsContainer) { $queue.Push($child.FullName) }
 } }
}
Remove-Item -LiteralPath $target -Recurse -Force
'''
            subprocess.run(["pwsh", "-NoProfile", "-NonInteractive", "-Command", script],
                           env={**os.environ, "HIVE_CLEAN_ROOT": str(self.root), "HIVE_CLEAN_TARGET": str(path)}, check=True)
        elif path.is_dir():
            if not shutil.rmtree.avoids_symlink_attacks:
                raise ArtifactError("platform lacks safe directory removal")
            shutil.rmtree(path)
        else:
            path.unlink()


class Run:
    """One durable report plus local ownership records; nested runners are supported."""
    def __init__(self, purpose, command, *, root=ROOT, paths=(), binary=None):
        self.manager = Manager(root)
        self.id = now().strftime("%Y%m%dT%H%M%S") + "-" + uuid.uuid4().hex[:12]
        self.relative = f"tests/results/runs/{self.id}.md"
        self.data = {"purpose": purpose, "command": list(command), "started_at": stamp(),
                     "platform": platform.platform(), "python": platform.python_version(), "status": "running",
                     "source_commit": self.manager.git("rev-parse", "HEAD").decode().strip(),
                     "tracked_diff_sha256": hashlib.sha256(self.manager.git("diff", "HEAD", "--binary")).hexdigest(),
                     "untracked_source_sha256": self.manager.untracked_source_digest(),
                     "fixture_tree_sha256": hashlib.sha256(self.manager.git("ls-tree", "-r", "HEAD", "--", "tests/fixtures")).hexdigest(),
                     "binary_sha256": sha(binary) if binary else None,
                     "input_limit": "fixture tree and tracked diff only; external inputs not fingerprinted"}
        self.output = []
        with self.manager.lock():
            identity = next(p for p in processes() if p["pid"] == os.getpid())
            self.identity = identity
            self._add_paths(paths)
            self.write()

    def _add_paths(self, paths):
        for relative in paths:
            self.manager.target(relative)
            self.manager.save({"id": self.id + "-" + hashlib.sha256(relative.encode()).hexdigest()[:12],
                               "run": self.id, "path": relative, "state": "active", "owner": self.data["purpose"],
                               "pid": os.getpid(), "process_start": self.identity["start"], "reason": "running test",
                               "started_at": self.data["started_at"]})

    def add_path(self, path):
        relative = Path(path).absolute().relative_to(self.manager.root).as_posix()
        with self.manager.lock():
            self._add_paths([relative])

    def write(self):
        report = confined(self.manager.root, self.relative, ("tests/results",))
        data = scrub(self.data, self.manager.root)
        body = "# 시험 실행 기록\n\n```json\n" + json.dumps(data, ensure_ascii=False, indent=2) + "\n```\n\n"
        body += "## 증명 범위와 한계\n\n- 위 명령·소스·실행 환경에 한정한 결과\n- 다른 소스·운영체제·실행하지 않은 시험의 통과 근거에서 제외\n- 변경 지문은 식별용이며 미커밋 소스 복구 수단에서 제외\n"
        if self.output:
            body += "\n## 실행 요약\n\n```text\n" + clean_text("\n".join(self.output), self.manager.root).replace("```", "~~~") + "\n```\n"
        atomic(report, body)

    def execute(self, command, *, env=None, cwd=None):
        started = time.monotonic()
        executable = Path(command[0]).name.casefold().removesuffix(".exe")
        if executable in ("cargo", "rustc", "python", "python3", "uv"):
            version = subprocess.run([command[0], "--version"], env=env, capture_output=True, text=True, check=False, timeout=20)
            self.data.setdefault("tools", {})[executable] = version.stdout.strip() or version.stderr.strip()
        lines = []
        with subprocess.Popen(command, cwd=cwd or self.manager.root, env=env, stdout=subprocess.PIPE,
                              stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace") as process:
            for line in process.stdout:
                print(line, end="", flush=True)
                # Summaries only, not raw host sessions or unlimited command output.
                if re.search(r"^(Ran \d+ tests?|OK(?:\s|$)|FAILED\b|test result:|error:|FAIL:|ERROR:|.*\.\.\. skipped )", line):
                    lines.append(line.rstrip()[:1000])
            code = process.wait()
        self.output.extend(lines[-1000:])
        self.data.setdefault("steps", []).append({"command": list(command), "exit_code": code, "elapsed_seconds": round(time.monotonic() - started, 3)})
        self.write()
        return code

    def finish(self, code, *, status=None, details=None):
        if status not in (None, "passed", "failed", "cancelled", "interrupted") or (status == "passed" and code != 0):
            raise ArtifactError("invalid terminal result")
        with self.manager.lock():
            self.data.update(status=status or ("passed" if code == 0 else "failed"), exit_code=code, finished_at=stamp())
            self.data["source_end"] = {
                "commit": self.manager.git("rev-parse", "HEAD").decode().strip(),
                "tracked_diff_sha256": hashlib.sha256(self.manager.git("diff", "HEAD", "--binary")).hexdigest(),
                "untracked_source_sha256": self.manager.untracked_source_digest(),
            }
            self.data["source_changed_during_run"] = any((
                self.data["source_commit"] != self.data["source_end"]["commit"],
                self.data["tracked_diff_sha256"] != self.data["source_end"]["tracked_diff_sha256"],
                self.data["untracked_source_sha256"] != self.data["source_end"]["untracked_source_sha256"],
            ))
            if details is not None:
                self.data["details"] = scrub(details, self.manager.root)
            self.write()  # Must succeed before changing lifecycle authority.
            digest = self.manager.evidence(self.relative, committed=False)
            for record in self.manager.records():
                if record.get("run") == self.id:
                    # A shared build tree needs a separate all-consumers review.
                    complete = code == 0 and record["path"] != "target/debug"
                    record.update(state="completed" if complete else "review", report=self.relative,
                                  report_sha256=digest, reason="completed test; report awaits commit" if complete else "shared build or failure reproduction review",
                                  review_at=(now() + MAX_REUSE).isoformat())
                    if complete:
                        record["inventory"] = self.manager.inventory(record["path"])
                    self.manager.save(record)
        return self.relative

    def archive_json(self, source, *, name="receipt.json"):
        """Preserve the existing JSON shape, redact private values, and bind its hash."""
        if not re.fullmatch(r"[a-zA-Z0-9_-]+\.json", name):
            raise ArtifactError("invalid attachment name")
        relative_source = Path(source).absolute().relative_to(self.manager.root).as_posix()
        source = confined(self.manager.root, relative_source, ("tests/work", "tests/results"))
        if linked(source) or source.stat().st_size > 4 * 1024 * 1024:
            raise ArtifactError("receipt must be a small regular JSON file")
        value = scrub(json.loads(source.read_text(encoding="utf-8")), self.manager.root)
        relative = f"tests/results/runs/{self.id}/{name}"
        destination = confined(self.manager.root, relative, ("tests/results",))
        atomic(destination, json.dumps(value, ensure_ascii=False, indent=2) + "\n")
        self.data.setdefault("attachments", []).append({"path": relative, "sha256": sha(destination),
                                                       "source_sha256": sha(source), "redacted": True})
        self.write()
        return destination


def cli(arguments=None):
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("scan", "check", "cleanup", "review", "archive", "index", "run"))
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--path", action="append")
    parser.add_argument("--sizes", action="store_true")
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--request", type=Path)
    parser.add_argument("--purpose")
    parser.add_argument("--command", nargs=argparse.REMAINDER)
    args = parser.parse_args(arguments)
    if args.apply and args.action != "cleanup":
        parser.error("--apply is only valid with cleanup")
    try:
        manager = Manager(args.root)
        if args.action == "run":
            if not args.purpose or not args.command:
                parser.error("run requires --purpose and --command")
            run = Run(args.purpose, args.command, root=args.root, paths=args.path or [])
            code, terminal = 1, None
            try:
                code = run.execute(args.command)
            except KeyboardInterrupt:
                code, terminal = 130, "cancelled"
            finally:
                print(run.finish(code, status=terminal))
            result_index(args.root)
            return code
        if args.action == "index":
            result_index(args.root)
            return 0
        if args.action == "archive":
            if not args.request:
                parser.error("archive requires --request with explicitly selected JSON evidence")
            for item in json.loads(args.request.read_text(encoding="utf-8")):
                print(archive_legacy(args.root, item["source"], purpose=item["purpose"], limit=item["limit"]))
            return 0
        if args.action == "review":
            if not args.request:
                parser.error("review requires --request")
            manager.review(json.loads(args.request.read_text(encoding="utf-8")))
            return 0
        rows = manager.cleanup(apply=args.apply, selected=args.path) if args.action == "cleanup" else manager.scan(sizes=args.sizes, selected=args.path)
        print(json.dumps(rows, ensure_ascii=False, indent=2))
        if args.action == "check":
            return int(any(r["status"] in ("eligible", "cleanup-failed") or (r["status"] == "review" and (not r.get("review_at") or datetime.fromisoformat(r["review_at"]) <= now())) for r in rows))
        return int(any(r["status"] == "cleanup-failed" for r in rows))
    except (ArtifactError, OSError, ValueError, subprocess.SubprocessError) as error:
        print("test-artifacts: " + str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(cli())
