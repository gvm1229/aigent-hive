"""Standard-library bootstrap for the consented, file-only vector runtime.

The Rust caller supplies its embedded lock and helper, and owns approval and promotion.
This is a dedicated wheel-module layout, not a dependency-complete pip installation.
"""

from __future__ import annotations

import base64
import csv
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import platform
import shutil
import stat
import sys
import urllib.parse
import urllib.request
import zipfile

MAX_DOWNLOAD = 512 * 1024 * 1024
PACKAGES = {"onnxruntime", "numpy", "flatbuffers", "packaging", "protobuf", "tokenizers", "sqlite-vec"}


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def digest_file(path: Path) -> str:
    with path.open("rb") as stream:
        return "sha256:" + hashlib.file_digest(stream, "sha256").hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def safe_path(value: str) -> Path:
    path = Path(os.path.abspath(value))
    for node in [path, *path.parents]:
        if node.is_symlink() or (hasattr(node, "is_junction") and node.is_junction()):
            raise ValueError("vector runtime paths cannot contain links")
    return path


def platform_key() -> str:
    if sys.implementation.name != "cpython" or sys.version_info[:2] not in ((3, 12), (3, 13)):
        raise ValueError("the vector runtime requires CPython 3.12 or 3.13")
    system, machine = platform.system(), platform.machine().lower()
    if system == "Windows" and machine in ("amd64", "x86_64"):
        name = "windows-x64"
    elif system == "Darwin" and machine in ("arm64", "aarch64"):
        name = "macos-arm64"
    elif system == "Linux" and machine == "x86_64" and platform.libc_ver()[0] == "glibc":
        name = "linux-x64"
    else:
        raise ValueError("unsupported vector Python platform")
    return f"{name}-cp{sys.version_info.major}{sys.version_info.minor}"


def validate_lock(lock: dict) -> list[dict]:
    if type(lock.get("schema_version")) is not int or lock.get("schema_version") != 1 or lock.get("execution_profile") != "file-only-offline-embedding":
        raise ValueError("unsupported vector runtime lock")
    packages = [lock["packages"][key] for key in lock["platforms"][platform_key()]]
    if len(packages) != 7 or {item["name"] for item in packages} != PACKAGES:
        raise ValueError("invalid file-only distribution set")
    for item in packages:
        if type(item.get("size")) is not int or not 1 <= item["size"] <= MAX_DOWNLOAD:
            raise ValueError("invalid wheel size")
        parsed = urllib.parse.urlparse(item["url"])
        if parsed.scheme != "https" or parsed.hostname != "files.pythonhosted.org" or parsed.username or parsed.password or parsed.port:
            raise ValueError("unapproved wheel URL")
        if len(item["sha256"]) != 64 or any(c not in "0123456789abcdef" for c in item["sha256"]):
            raise ValueError("invalid wheel digest")
    model = lock["model"]
    if model.get("dimension") != 384 or model.get("token_limit") != 128 or model.get("pooling") != "title-context-window-mean" or model.get("normalization") != "l2":
        raise ValueError("unsupported embedding contract")
    if len(model["revision"]) != 40 or any(c not in "0123456789abcdef" for c in model["revision"]):
        raise ValueError("invalid model revision")
    repository = model["repository"].split("/")
    if len(repository) != 2 or any(not part or part in (".", "..") or any(not (c.isascii() and (c.isalnum() or c in "._-")) for c in part) for part in repository):
        raise ValueError("invalid model repository")
    if len(model["files"]) != 5 or {item["filename"] for item in model["files"]} != {"model_optimized.onnx", "tokenizer.json", "tokenizer_config.json", "special_tokens_map.json", "config.json"}:
        raise ValueError("unsupported model file inventory")
    for item in model["files"]:
        if type(item["size"]) is not int or not 1 <= item["size"] <= MAX_DOWNLOAD or len(item["sha256"]) != 64 or any(c not in "0123456789abcdef" for c in item["sha256"]):
            raise ValueError("invalid model file contract")
    return packages


def describe(lock: dict) -> dict:
    packages = validate_lock(lock)
    executable = Path(sys.executable).resolve(strict=True)
    return {
        "platform": platform_key(),
        "python_version": platform.python_version(),
        "python_executable": str(executable),
        "python_digest": digest_file(executable),
        "lock_digest": digest_bytes(canonical(lock)),
        "packages": [{"name": item["name"], "version": item["version"], "sha256": item["sha256"]} for item in packages],
        "complete_pip_environment": False,
        "omitted_declared_dependencies": lock["omitted_declared_dependencies"],
        "model": lock["model"]["id"],
        "download_bytes": sum(item["size"] for item in packages) + sum(item["size"] for item in lock["model"]["files"]),
    }


class ArtifactRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, new_url):
        parsed = urllib.parse.urlparse(new_url)
        host = parsed.hostname or ""
        if parsed.scheme != "https" or parsed.username or parsed.password or parsed.port or not (
            host == "files.pythonhosted.org" or host == "huggingface.co" or host.endswith(".huggingface.co")
            or host.endswith(".hf.co") or host == "hf.co"
        ):
            raise ValueError("unapproved artifact redirect")
        return super().redirect_request(request, response, code, message, headers, new_url)


def fetch(url: str, sha256: str, cache: Path) -> Path:
    target = safe_path(str(cache / sha256))
    if target.exists():
        if digest_file(target) != "sha256:" + sha256:
            raise ValueError("corrupt vector download cache")
        return target
    temporary = cache / (sha256 + ".partial")
    if temporary.exists():
        raise ValueError("incomplete download needs a new staging cache")
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), ArtifactRedirect())
    try:
        with opener.open(url, timeout=30) as response, temporary.open("xb") as output:
            total = 0
            while block := response.read(1024 * 1024):
                total += len(block)
                if total > MAX_DOWNLOAD:
                    raise ValueError("vector artifact exceeds download limit")
                output.write(block)
        if digest_file(temporary) != "sha256:" + sha256:
            raise ValueError("vector artifact digest mismatch")
        os.replace(temporary, target)
    except BaseException:
        # Only this invocation's private partial download is removed.
        if temporary.exists() and not temporary.is_symlink():
            temporary.unlink()
        raise
    return target


def wheel_path(name: str) -> str | None:
    path = PurePosixPath(name)
    if path.is_absolute() or ".." in path.parts or "\\" in name or ":" in name or "\x00" in name:
        raise ValueError("unsafe wheel member")
    if path.suffix in (".pth", ".pyc"):
        raise ValueError("runtime activation files are not allowed")
    parts = path.parts
    if parts and parts[0].endswith(".data"):
        if len(parts) >= 3 and parts[1] in ("purelib", "platlib"):
            return "/".join(parts[2:])
        if len(parts) >= 3 and parts[1] == "scripts":
            return None  # Console scripts are outside the file-only execution profile.
        raise ValueError("unsupported wheel data layout")
    return str(path)


def wheel_files(archive: zipfile.ZipFile) -> dict[str, tuple[str, str]]:
    records = [name for name in archive.namelist() if name.endswith(".dist-info/RECORD")]
    if len(records) != 1:
        raise ValueError("wheel has no exact RECORD")
    indexed = {row[0]: row[1] for row in csv.reader(io.StringIO(archive.read(records[0]).decode("utf-8")))}
    files = {}
    folded = set()
    size = 0
    for item in archive.infolist():
        if item.is_dir():
            continue
        if stat.S_IFMT(item.external_attr >> 16) not in (0, stat.S_IFREG):
            raise ValueError("non-regular wheel entry")
        relative = wheel_path(item.filename)
        if relative is None:
            continue
        size += item.file_size
        if size > 1024 * 1024 * 1024 or len(files) >= 20_000 or relative.casefold() in folded:
            raise ValueError("wheel member limits or uniqueness failed")
        folded.add(relative.casefold())
        recorded = indexed.get(item.filename)
        if recorded is None:
            raise ValueError("wheel member is absent from RECORD")
        if recorded:
            algorithm, encoded = recorded.split("=", 1)
            if algorithm != "sha256":
                raise ValueError("unsupported wheel RECORD digest")
            expected = base64.urlsafe_b64decode(encoded + "=" * (-len(encoded) % 4)).hex()
        else:
            expected = hashlib.sha256(archive.read(item)).hexdigest()
        files[relative] = (item.filename, expected)
    return files


def copy_exclusive(source: Path, destination: Path) -> None:
    with source.open("rb") as input_file, safe_path(str(destination)).open("xb") as output:
        shutil.copyfileobj(input_file, output, 1024 * 1024)


def install(request: dict) -> dict:
    lock = request["lock"]
    identity = describe(lock)
    root = safe_path(request["root"])
    cache = safe_path(request["cache"])
    if not root.is_dir() or any(root.iterdir()):
        raise ValueError("runtime staging root must be empty")
    cache.mkdir(parents=True, exist_ok=True, mode=0o700)
    site, wheels, model = (root / name for name in ("site", "wheels", "model"))
    for directory in (site, wheels, model, root / "tmp"):
        directory.mkdir(mode=0o700)
    for item in validate_lock(lock):
        artifact = fetch(item["url"], item["sha256"], cache)
        destination = wheels / (item["sha256"] + ".whl")
        copy_exclusive(artifact, destination)
        with zipfile.ZipFile(destination) as archive:
            for relative, (member, expected) in wheel_files(archive).items():
                target = safe_path(str(site / relative))
                target.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
                data = archive.read(member)
                if hashlib.sha256(data).hexdigest() != expected:
                    raise ValueError("wheel file digest mismatch")
                if target.exists():
                    if digest_file(target) != "sha256:" + expected:
                        raise ValueError("conflicting runtime distribution files")
                else:
                    with target.open("xb") as output:
                        output.write(data)
    model_lock = lock["model"]
    for item in model_lock["files"]:
        url = f"https://huggingface.co/{model_lock['repository']}/resolve/{model_lock['revision']}/{item['filename']}"
        artifact = fetch(url, item["sha256"], cache)
        copy_exclusive(artifact, model / item["filename"])
    helper = request["helper"].encode("utf-8")
    with (root / "vector_helper.py").open("xb") as output:
        output.write(helper)
    receipt = {"schema_version": 1, **identity, "helper_digest": digest_bytes(helper)}
    with (root / "receipt.json").open("xb") as output:
        output.write(canonical(receipt))
    return verify({**request, "helper_digest": digest_bytes(helper)})


def verify(request: dict) -> dict:
    root = safe_path(request["root"])
    identity = describe(request["lock"])
    receipt = json.loads(safe_path(str(root / "receipt.json")).read_text("utf-8"))
    if any(receipt.get(key) != value for key, value in identity.items()) or receipt.get("helper_digest") != request["helper_digest"]:
        raise ValueError("vector runtime identity changed")
    if digest_file(safe_path(str(root / "vector_helper.py"))) != request["helper_digest"]:
        raise ValueError("vector helper changed")
    expected_files = {}
    for item in validate_lock(request["lock"]):
        wheel = safe_path(str(root / "wheels" / (item["sha256"] + ".whl")))
        if wheel.stat().st_size != item["size"] or digest_file(wheel) != "sha256:" + item["sha256"]:
            raise ValueError("vector wheel changed")
        with zipfile.ZipFile(wheel) as archive:
            for relative, (_, expected) in wheel_files(archive).items():
                if relative in expected_files and expected_files[relative] != expected:
                    raise ValueError("conflicting runtime inventory")
                expected_files[relative] = expected
    actual_files = set()
    site = safe_path(str(root / "site"))
    for directory, folders, files in os.walk(site):
        # Walk top-down and reject reparse points before descending; do not re-stat
        # every already checked ancestor for every distribution file.
        for name in folders:
            node = Path(directory) / name
            if node.is_symlink() or (hasattr(node, "is_junction") and node.is_junction()):
                raise ValueError("linked runtime directory")
        for name in files:
            path = Path(directory) / name
            metadata = path.lstat()
            if not stat.S_ISREG(metadata.st_mode) or getattr(metadata, "st_file_attributes", 0) & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0):
                raise ValueError("non-regular runtime file")
            relative = path.relative_to(site).as_posix()
            if expected_files.get(relative) != digest_file(path)[7:]:
                raise ValueError("vector runtime file changed")
            actual_files.add(relative)
    if actual_files != set(expected_files):
        raise ValueError("vector runtime inventory changed")
    for item in request["lock"]["model"]["files"]:
        path = safe_path(str(root / "model" / item["filename"]))
        if path.stat().st_size != item["size"] or digest_file(path) != "sha256:" + item["sha256"]:
            raise ValueError("vector model changed")
    return {"receipt_digest": digest_bytes(canonical(receipt)), "identity": identity, "verified": True}


def main() -> int:
    try:
        raw = sys.stdin.buffer.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            raise ValueError("bootstrap request exceeds limits")
        request = json.loads(raw)
        action = request.get("action")
        if action == "describe":
            result = describe(request["lock"])
        elif action == "stage":
            result = install(request)
        elif action == "verify":
            result = verify(request)
        else:
            raise ValueError("unsupported bootstrap action")
        print(json.dumps({"status": "success", **result}, allow_nan=False))
        return 0
    except Exception as error:
        print(json.dumps({"status": "error", "error_type": type(error).__name__}))
        return 10


if __name__ == "__main__":
    raise SystemExit(main())
