"""Non-generative offline embedding and sqlite-vec worker, invoked only by Hive.

The caller authenticates the runtime and owns scope, consent, locks, and publication.
This process never reads canonical knowledge, credentials, or host configuration.
"""

from __future__ import annotations

import concurrent.futures
import hashlib
import json
import math
import os
from pathlib import Path
import socket
import sqlite3
import struct
import sys
import threading
import time

DIMENSION = 384
SCHEMA = 1
MAX_REQUEST = 256 * 1024 * 1024
MAX_CHUNKS = 50_000
MODEL_FILE = "model_optimized.onnx"
TOKENIZER_FILE = "tokenizer.json"
TOKEN_LIMIT = 128


def deny_network(*_args, **_kwargs):
    raise RuntimeError("network is not part of the vector worker contract")


def offline_environment() -> None:
    # This blocks Python socket APIs, not arbitrary native OS syscalls.
    socket.create_connection = deny_network
    socket.socket.connect = deny_network
    socket.socket.connect_ex = deny_network
    os.environ["HF_HUB_OFFLINE"] = "1"
    os.environ["HF_HUB_DISABLE_TELEMETRY"] = "1"
    os.environ["TOKENIZERS_PARALLELISM"] = "false"
    sys.dont_write_bytecode = True


def regular_path(value: str, directory: bool = False) -> Path:
    path = Path(os.path.abspath(value))
    for ancestor in [path, *path.parents]:
        if ancestor.is_symlink() or (hasattr(ancestor, "is_junction") and ancestor.is_junction()):
            raise ValueError("vector worker paths cannot contain links")
    if directory and not path.is_dir():
        raise ValueError("vector worker directory is absent")
    if not directory and path.exists() and not path.is_file():
        raise ValueError("vector worker file is not regular")
    return path


def native_loader_path(path: Path) -> str:
    """Preserve long Windows DLL/model paths without changing SQLite URI names."""
    value = str(path)
    if sys.platform != "win32" or value.startswith("\\\\?\\"):
        return value
    if value.startswith("\\\\"):
        return "\\\\?\\UNC\\" + value[2:]
    return "\\\\?\\" + value


def windows_performance_mask(raw: bytes, process_mask: int, selected_ids=None) -> int:
    """Use one CPU class without expanding the caller's affinity/CPU Set authority."""
    rows, offset = [], 0
    while offset < len(raw):
        if len(raw) - offset < 8:
            raise ValueError("truncated CPU Set information")
        size, kind = struct.unpack_from("<II", raw, offset)
        if size < 8 or size > len(raw) - offset:
            raise ValueError("invalid CPU Set information size")
        if kind == 0:
            if size < 32:
                raise ValueError("truncated CPU Set record")
            identity, group = struct.unpack_from("<IH", raw, offset + 8)
            rows.append((identity, group, raw[offset + 14], raw[offset + 18], raw[offset + 19]))
        offset += size
    if not rows:
        raise ValueError("CPU Set information is unavailable")
    classes = {row[3] for row in rows}
    if len(classes) == 1:
        return process_mask  # Homogeneous processors need no scheduling change.
    if any(row[1] != 0 or row[2] >= 64 for row in rows):
        raise ValueError("heterogeneous multi-group CPU profile is unsupported")
    highest = max(classes)
    selected = 0
    for identity, _group, logical, efficiency, flags in rows:
        if efficiency != highest or flags & 2 and not flags & 4:
            continue
        if selected_ids is not None and identity not in selected_ids:
            continue
        selected |= (1 << logical) & process_mask
    if not selected:
        raise ValueError("no permitted performance CPU is available")
    return selected


def stabilize_cpu_profile():
    """Constrain this short-lived Windows worker only; never alter another process."""
    if sys.platform != "win32":
        return None
    import ctypes as c
    from ctypes import wintypes as w
    api = c.WinDLL("kernel32", use_last_error=True)
    api.GetCurrentProcess.restype = w.HANDLE
    api.GetSystemCpuSetInformation.argtypes = [c.c_void_p, w.ULONG, c.POINTER(w.ULONG), w.HANDLE, w.ULONG]
    api.GetSystemCpuSetInformation.restype = w.BOOL
    api.GetProcessDefaultCpuSets.argtypes = [w.HANDLE, c.POINTER(w.ULONG), w.ULONG, c.POINTER(w.ULONG)]
    api.GetProcessDefaultCpuSets.restype = w.BOOL
    api.GetProcessAffinityMask.argtypes = [w.HANDLE, c.POINTER(c.c_size_t), c.POINTER(c.c_size_t)]
    api.GetProcessAffinityMask.restype = w.BOOL
    api.SetProcessAffinityMask.argtypes = [w.HANDLE, c.c_size_t]
    api.SetProcessAffinityMask.restype = w.BOOL
    handle, length = api.GetCurrentProcess(), w.ULONG()
    if not api.GetSystemCpuSetInformation(None, 0, c.byref(length), handle, 0) and c.get_last_error() != 122:
        raise OSError("cannot inspect CPU Sets")
    if not 1 <= length.value <= 1024 * 1024:
        raise ValueError("invalid CPU Set inventory size")
    buffer = c.create_string_buffer(length.value)
    if not api.GetSystemCpuSetInformation(buffer, length.value, c.byref(length), handle, 0):
        raise OSError("CPU Set inventory changed")
    count = w.ULONG()
    if not api.GetProcessDefaultCpuSets(handle, None, 0, c.byref(count)) and c.get_last_error() != 122:
        raise OSError("cannot inspect process CPU Sets")
    if count.value > 65536:
        raise ValueError("invalid process CPU Set count")
    selected_ids = None
    if count.value:
        ids = (w.ULONG * count.value)()
        if not api.GetProcessDefaultCpuSets(handle, ids, count.value, c.byref(count)):
            raise OSError("process CPU Sets changed")
        selected_ids = set(ids[:count.value])
    original, system = c.c_size_t(), c.c_size_t()
    if not api.GetProcessAffinityMask(handle, c.byref(original), c.byref(system)):
        raise OSError("cannot inspect worker affinity")
    selected = windows_performance_mask(buffer.raw[:length.value], original.value, selected_ids)
    if selected != original.value:
        if not api.SetProcessAffinityMask(handle, selected):
            raise OSError("cannot bind worker CPU profile")
        applied = c.c_size_t()
        if not api.GetProcessAffinityMask(handle, c.byref(applied), c.byref(system)) or applied.value != selected:
            raise OSError("worker CPU profile changed")
    return selected


def load_runtime(runtime: str) -> Path:
    root = regular_path(runtime, directory=True)
    site = regular_path(str(root / "site"), directory=True)
    # -I -S and this explicit path prevent user-site and .pth activation.
    sys.path.insert(0, native_loader_path(site))
    offline_environment()
    return root


def database_path(runtime: Path, value: str, writable: bool) -> Path:
    path = regular_path(value)
    base = runtime.parent.parent
    if runtime.parent.name != "runtimes" or base.name != "vector":
        raise ValueError("vector database requires an activated runtime")
    if tuple(base.parts[-3:]) not in ((".hive", "index", "vector"), (".agents", "work", "vector")):
        raise ValueError("vector database root is not Hive-owned")
    parts = path.relative_to(base).parts
    expected_length = 3 if writable else 5
    if len(parts) != expected_length or parts[0] != "scopes" or len(parts[1]) != 64 or any(c not in "0123456789abcdef" for c in parts[1]):
        raise ValueError("invalid vector scope path")
    if writable and parts[2] != "staging.sqlite3":
        raise ValueError("vector writes are restricted to staging")
    if not writable and (parts[2] != "generations" or len(parts[3]) != 64 or any(c not in "0123456789abcdef" for c in parts[3]) or parts[4] != "index.sqlite3"):
        raise ValueError("vector queries require a published generation")
    if path.exists():
        metadata = path.stat()
        if metadata.st_nlink != 1:
            raise ValueError("hardlinked vector databases are forbidden")
        if metadata.st_size > 512 * 1024 * 1024:
            raise ValueError("vector database exceeds limits")
    return path


def validate_digest(value: object) -> str:
    if not isinstance(value, str) or len(value) != 71 or not value.startswith("sha256:") or any(c not in "0123456789abcdef" for c in value[7:]):
        raise ValueError("invalid vector digest")
    return value


def database_digest(path: Path) -> str:
    with path.open("rb") as stream:
        return "sha256:" + hashlib.file_digest(stream, "sha256").hexdigest()


def authenticate_database(path: Path, expected: object, create: bool) -> tuple[int, int]:
    reject_database_sidecars(path)
    if path.exists():
        validate_digest(expected)
        before = path.stat()
        if before.st_nlink != 1 or database_digest(path) != expected:
            raise ValueError("vector database differs from the trusted checkpoint")
        after = path.stat()
        if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns):
            raise ValueError("vector database identity changed")
        return after.st_dev, after.st_ino
    if not create or expected is not None:
        raise ValueError("trusted vector database is absent")
    descriptor = os.open(path, os.O_RDWR | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600)
    try:
        metadata = os.fstat(descriptor)
        return metadata.st_dev, metadata.st_ino
    finally:
        os.close(descriptor)


def check_database_identity(path: Path, identity: tuple[int, int]) -> None:
    metadata = path.lstat()
    if path.is_symlink() or metadata.st_nlink != 1 or (metadata.st_dev, metadata.st_ino) != identity:
        raise ValueError("vector database changed before use")


def reject_database_sidecars(path: Path) -> None:
    for suffix in ("-journal", "-wal", "-shm"):
        sibling = Path(str(path) + suffix)
        if sibling.exists() or sibling.is_symlink():
            raise ValueError("vector database has an untrusted SQLite sidecar")


def initialize_encoder(runtime: str) -> None:
    global _session, _tokenizer, _numpy, _pad_id, _cls_id, _sep_id
    stabilize_cpu_profile()
    root = load_runtime(runtime)
    import numpy as np
    import onnxruntime as ort
    from tokenizers import AddedToken, Tokenizer

    _numpy = np
    ort.disable_telemetry_events()
    options = ort.SessionOptions()
    options.intra_op_num_threads = 1
    options.inter_op_num_threads = 1
    options.log_severity_level = 3
    _session = ort.InferenceSession(
        native_loader_path(regular_path(str(root / "model" / MODEL_FILE))),
        options,
        providers=["CPUExecutionProvider"],
    )
    _tokenizer = Tokenizer.from_file(native_loader_path(regular_path(str(root / "model" / TOKENIZER_FILE))))
    config = json.loads(regular_path(str(root / "model/config.json")).read_text("utf-8"))
    tokenizer_config = json.loads(regular_path(str(root / "model/tokenizer_config.json")).read_text("utf-8"))
    if min(tokenizer_config["model_max_length"], tokenizer_config["max_length"]) != TOKEN_LIMIT:
        raise ValueError("unsupported tokenizer context")
    if not _tokenizer.padding:
        _tokenizer.enable_padding(pad_id=config["pad_token_id"], pad_token=tokenizer_config["pad_token"])
    _pad_id = _tokenizer.padding["pad_id"]
    _tokenizer.no_truncation()
    _tokenizer.no_padding()
    special = json.loads(regular_path(str(root / "model/special_tokens_map.json")).read_text("utf-8"))
    for token in special.values():
        _tokenizer.add_special_tokens([AddedToken(**token) if isinstance(token, dict) else token])
    _cls_id = _tokenizer.token_to_id(tokenizer_config["cls_token"])
    _sep_id = _tokenizer.token_to_id(tokenizer_config["sep_token"])
    if _cls_id is None or _sep_id is None:
        raise ValueError("missing model boundary tokens")


def encode_batch(records: list[dict[str, str]]) -> list[bytes]:
    # Do not rely on truncation overflow metadata: explicitly partition all body
    # token IDs and repeat the authenticated title context for each window.
    bodies = _tokenizer.encode_batch([record["text"] for record in records], add_special_tokens=False)
    prefixes = [_tokenizer.encode(record["title"], add_special_tokens=False).ids[:32] for record in records]
    windows, owners, weights = [], [], []
    for index, body in enumerate(bodies):
        prefix = prefixes[index]
        width = TOKEN_LIMIT-2-len(prefix)
        for offset in range(0, max(1, len(body.ids)), width):
            content = prefix+body.ids[offset:offset+width]
            windows.append([_cls_id, *content, _sep_id])
            owners.append(index)
            weights.append(max(1, len(content)))
    pooled = _numpy.zeros((len(records), DIMENSION), dtype=_numpy.float64)
    totals = _numpy.zeros((len(records), 1), dtype=_numpy.float64)
    for offset in range(0, len(windows), 64):
        batch = windows[offset:offset+64]
        width = max(len(item) for item in batch)
        values = {
            "input_ids": _numpy.asarray([item+[_pad_id]*(width-len(item)) for item in batch], dtype=_numpy.int64),
            "attention_mask": _numpy.asarray([[1]*len(item)+[0]*(width-len(item)) for item in batch], dtype=_numpy.int64),
            "token_type_ids": _numpy.zeros((len(batch), width), dtype=_numpy.int64),
        }
        output = _session.run(None, {item.name: values[item.name] for item in _session.get_inputs()})[0]
        mask = values["attention_mask"][..., None]
        means = (output*mask).sum(axis=1)/_numpy.maximum(mask.sum(axis=1), 1)
        for index, mean in enumerate(means, offset):
            pooled[owners[index]] += mean*weights[index]
            totals[owners[index]] += weights[index]
    pooled /= _numpy.maximum(totals, 1)
    pooled /= _numpy.maximum(_numpy.linalg.norm(pooled, axis=1, keepdims=True), 1e-12)
    if pooled.shape != (len(records), DIMENSION) or not _numpy.isfinite(pooled).all():
        raise ValueError("invalid embedding output")
    return [row.astype("<f4").tobytes() for row in pooled]


def open_database(path: Path, readonly: bool = False) -> sqlite3.Connection:
    import sqlite_vec

    connection = sqlite3.connect(path.as_uri() + "?mode=ro&immutable=1", uri=True) if readonly else sqlite3.connect(path)
    try:
        connection.enable_load_extension(True)
        sqlite_vec.load(connection)
    finally:
        connection.enable_load_extension(False)
    if connection.execute("SELECT vec_version()").fetchone()[0] != "v0.1.9":
        connection.close()
        raise ValueError("unsupported vector engine")
    connection.execute("PRAGMA mmap_size=268435456")
    connection.execute("PRAGMA cache_size=-100000")
    connection.execute("PRAGMA trusted_schema=OFF")
    if readonly:
        connection.execute("PRAGMA query_only=ON")
    return connection


def text_digest(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def record_digest(row: dict[str, str]) -> str:
    return text_digest(json.dumps([row["title"], row["text"]], ensure_ascii=False, separators=(",", ":")))


def validate_chunks(chunks: object) -> list[dict[str, str]]:
    if not isinstance(chunks, list) or len(chunks) > MAX_CHUNKS:
        raise ValueError("invalid vector corpus size")
    seen = set()
    for item in chunks:
        if not isinstance(item, dict) or set(item) != {"chunk_id", "digest", "title", "text"}:
            raise ValueError("invalid vector corpus row")
        if not all(isinstance(value, str) and value.strip() for value in item.values()):
            raise ValueError("invalid vector corpus field")
        if len(item["chunk_id"]) > 256 or len(item["title"].encode("utf-8")) > 4096 or len(item["text"].encode("utf-8")) > 128 * 1024:
            raise ValueError("vector corpus row exceeds limits")
        if item["chunk_id"] in seen:
            raise ValueError("duplicate vector corpus identity")
        seen.add(item["chunk_id"])
        digest = item["digest"]
        if len(digest) != 71 or not digest.startswith("sha256:") or any(c not in "0123456789abcdef" for c in digest[7:]):
            raise ValueError("invalid canonical digest")
    return chunks


def input_binding(request: dict, chunks: list[dict]) -> str:
    return hashlib.sha256(json.dumps(
        [request["contract_digest"], request["manifest_digest"], [(row["chunk_id"], row["digest"], record_digest(row)) for row in chunks]],
        separators=(",", ":"), ensure_ascii=False,
    ).encode("utf-8")).hexdigest()


def compact_generation(connection: sqlite3.Connection, deadline: float) -> bool:
    if connection.execute("PRAGMA auto_vacuum").fetchone()[0] != 2:
        raise ValueError("unsupported vector compaction policy; rebuild fresh")
    while (before := connection.execute("PRAGMA freelist_count").fetchone()[0]) > 0:
        if time.monotonic() >= deadline:
            return False
        connection.execute("PRAGMA incremental_vacuum(256)").fetchall()
        connection.commit()
        if connection.execute("PRAGMA freelist_count").fetchone()[0] >= before:
            raise ValueError("vector compaction cannot advance")
    connection.execute("INSERT OR REPLACE INTO meta VALUES ('phase','ready')")
    connection.commit()
    return True


def restore_embedding_cache(connection: sqlite3.Connection, contract: str, deadline: float) -> bool:
    # Completed generations retain a single copy in vec0. Reconstruct only a mutable
    # working cache from this externally authenticated generation before changing it.
    meta = dict(connection.execute("SELECT key,value FROM meta"))
    cursor = int(meta.get("restore_cursor", "0")) if meta.get("phase") == "restoring" else 0
    if not 0 <= cursor <= MAX_CHUNKS:
        raise ValueError("invalid vector restore cursor")
    connection.execute("INSERT OR REPLACE INTO meta VALUES ('restore_cursor',?)", (str(cursor),))
    connection.execute("INSERT OR REPLACE INTO meta VALUES ('phase','restoring')")
    connection.commit()
    while True:
        if time.monotonic() >= deadline:
            return False
        rows = connection.execute("SELECT d.id,d.text_digest,v.embedding FROM documents d JOIN vectors v ON v.rowid=d.id WHERE d.id>? ORDER BY d.id LIMIT 512", (cursor,)).fetchall()
        if not rows:
            break
        for cursor, digest, vector in rows:
            connection.execute("INSERT OR IGNORE INTO cache VALUES (?,?,?,?)", (contract, digest, vector, hashlib.sha256(vector).hexdigest()))
        connection.execute("INSERT OR REPLACE INTO meta VALUES ('restore_cursor',?)", (str(cursor),))
        connection.commit()
    connection.execute("INSERT OR REPLACE INTO meta VALUES ('phase','embedding')")
    connection.commit()
    return True


def finalize(connection: sqlite3.Connection, request: dict, chunks: list[dict], deadline: float) -> bool:
    contract = request["contract_digest"]
    binding = input_binding(request, chunks)
    meta = dict(connection.execute("SELECT key,value FROM meta"))
    if meta.get("finalize_binding") != binding:
        connection.execute("DROP TABLE IF EXISTS vectors")
        connection.execute("DROP TABLE IF EXISTS documents")
        connection.execute(f"CREATE VIRTUAL TABLE vectors USING vec0(embedding float[{DIMENSION}])")
        connection.execute("CREATE TABLE documents(id INTEGER PRIMARY KEY, chunk_id TEXT UNIQUE NOT NULL, digest TEXT NOT NULL, text_digest TEXT NOT NULL)")
        connection.execute("CREATE INDEX documents_text_digest ON documents(text_digest)")
        for key, value in {"finalize_binding": binding, "finalize_cursor": "0", "phase": "finalizing"}.items():
            connection.execute("INSERT OR REPLACE INTO meta VALUES (?,?)", (key, value))
        connection.commit()
        cursor = 0
    else:
        cursor = int(meta["finalize_cursor"])
        if not 0 <= cursor <= len(chunks) or connection.execute("SELECT count(*) FROM documents").fetchone()[0] != cursor:
            raise ValueError("invalid finalization checkpoint")
    for offset in range(cursor, len(chunks), 512):
        if time.monotonic() >= deadline:
            return False
        batch = chunks[offset:offset+512]
        for index, row in enumerate(batch, offset+1):
            digest = record_digest(row)
            vector, checksum = connection.execute("SELECT vector,checksum FROM cache WHERE contract=? AND digest=?", (contract, digest)).fetchone()
            if len(vector) != DIMENSION * 4 or hashlib.sha256(vector).hexdigest() != checksum:
                raise ValueError("cached embedding integrity mismatch")
            values = struct.unpack(f"<{DIMENSION}f", vector)
            if not all(math.isfinite(value) for value in values) or not 0.999 <= sum(value*value for value in values) <= 1.001:
                raise ValueError("cached embedding is not normalized and finite")
            connection.execute("INSERT INTO vectors(rowid,embedding) VALUES (?,?)", (index, vector))
            connection.execute("INSERT INTO documents VALUES (?,?,?,?)", (index, row["chunk_id"], row["digest"], digest))
        connection.execute("INSERT OR REPLACE INTO meta VALUES ('finalize_cursor',?)", (str(offset+len(batch)),))
        connection.commit()
    connection.set_progress_handler(lambda: int(time.monotonic() >= deadline), 1000)
    try:
        connection.execute("DELETE FROM cache")
        for key, value in {"schema_version": str(SCHEMA), "contract_digest": contract, "manifest_digest": request["manifest_digest"], "phase": "compacting"}.items():
            connection.execute("INSERT OR REPLACE INTO meta VALUES (?,?)", (key, value))
        connection.commit()
    except sqlite3.OperationalError:
        connection.rollback()
        if time.monotonic() >= deadline:
            return False
        raise
    finally:
        connection.set_progress_handler(None, 0)
    return compact_generation(connection, deadline)


def build(request: dict, *, encoder=None, final_deadline=None, soft_deadline=None) -> dict:
    contract = validate_digest(request["contract_digest"])
    validate_digest(request["manifest_digest"])
    chunks = sorted(validate_chunks(request["chunks"]), key=lambda row: row["chunk_id"])
    workers = request["workers"]
    seconds = request["max_seconds"]
    if type(workers) is not int or not 1 <= workers <= 16 or type(seconds) is not int or not 1 <= seconds <= 60:
        raise ValueError("invalid vector execution budget")
    root = load_runtime(request["runtime"])
    path = database_path(root, request["database"], writable=True)
    identity = authenticate_database(path, request["expected_database_digest"], create=True)
    started = time.monotonic()
    deadline = min(started+seconds, soft_deadline or math.inf)
    connection = open_database(path)
    try:
        check_database_identity(path, identity)
        connection.execute("PRAGMA journal_mode=DELETE")
        if connection.execute("PRAGMA auto_vacuum").fetchone()[0] != 2:
            if connection.execute("SELECT 1 FROM sqlite_schema LIMIT 1").fetchone():
                raise ValueError("unsupported vector compaction policy; rebuild fresh")
            connection.execute("PRAGMA auto_vacuum=INCREMENTAL")
        connection.execute("CREATE TABLE IF NOT EXISTS cache(contract TEXT, digest TEXT, vector BLOB NOT NULL, checksum TEXT NOT NULL, PRIMARY KEY(contract,digest))")
        connection.execute("CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL)")
        meta = dict(connection.execute("SELECT key,value FROM meta"))
        same_input = meta.get("finalize_binding") == input_binding(request, chunks)
        if same_input and meta.get("phase") in ("ready", "compacting"):
            embedded, missing, embeddings_complete = 0, [], True
            complete = meta["phase"] == "ready" or compact_generation(connection, min(time.monotonic()+30.0, final_deadline or math.inf))
        else:
            restored = True
            if meta.get("phase") in ("ready", "compacting", "restoring"):
                restored = restore_embedding_cache(connection, meta["contract_digest"], deadline)
            unique = {record_digest(row): {"title": row["title"], "text": row["text"]} for row in chunks}
            cached = {row[0] for row in connection.execute("SELECT digest FROM cache WHERE contract=?", (contract,))}
            missing = [(digest, text) for digest, text in sorted(unique.items()) if digest not in cached]
            embedded = 0
            # The caller imposes a hard process-tree deadline. A forced termination has
            # no trusted receipt: restore the last authenticated checkpoint, never
            # authenticate a killed worker's mutable cache from its self-reported checksum.
            if restored and missing:
                parallelism = min(workers, (len(missing)+63)//64)
                (encoder or initialize_encoder)(str(root))
                with concurrent.futures.ThreadPoolExecutor(max_workers=parallelism) as pool:
                    window_size = 64 * parallelism
                    for offset in range(0, len(missing), window_size):
                        # Preserve one bounded progress window even when model startup uses
                        # the soft budget. The parent's hard deadline still bounds this process.
                        if time.monotonic() >= deadline and offset > 0:
                            break
                        window = missing[offset:offset + window_size]
                        batches = [window[index:index + 64] for index in range(0, len(window), 64)]
                        for batch, vectors in zip(batches, pool.map(encode_batch, [[text for _, text in batch] for batch in batches]), strict=True):
                            check_database_identity(path, identity)
                            for (digest, _), vector in zip(batch, vectors, strict=True):
                                connection.execute("INSERT OR REPLACE INTO cache VALUES (?,?,?,?)", (contract, digest, vector, hashlib.sha256(vector).hexdigest()))
                            connection.commit()
                            embedded += len(batch)
            embeddings_complete = restored and embedded == len(missing)
            complete = finalize(connection, request, chunks, min(time.monotonic()+30.0, final_deadline or math.inf)) if embeddings_complete else False
        result = {
            "complete": complete, "phase": "ready" if complete else dict(connection.execute("SELECT key,value FROM meta")).get("phase", "embedding"),
            "embedded": embedded, "remaining": len(missing)-embedded, "chunks": len(chunks),
            "elapsed_seconds": time.monotonic()-started,
        }
    finally:
        connection.close()
    check_database_identity(path, identity)
    reject_database_sidecars(path)
    result["database_digest"] = database_digest(path)
    return result


def build_many(request: dict) -> dict:
    """Share only the encoder across a bounded window; each database stays independent."""
    validate_digest(request["contract_digest"])
    workers, seconds, databases = request["workers"], request["max_seconds"], request["databases"]
    if type(workers) is not int or not 1 <= workers <= 16 or type(seconds) is not int or not 1 <= seconds <= 60:
        raise ValueError("invalid vector execution budget")
    if not isinstance(databases, list) or not 1 <= len(databases) <= 16:
        raise ValueError("invalid vector build window")
    count = size = 0
    for item in databases:
        if not isinstance(item, dict) or set(item) != {"database", "chunks", "manifest_digest", "expected_database_digest"}:
            raise ValueError("invalid vector build partition")
        validate_digest(item["manifest_digest"])
        if item["expected_database_digest"] is not None:
            validate_digest(item["expected_database_digest"])
        chunks = validate_chunks(item["chunks"])
        count += len(chunks)
        size += sum(len(row["title"].encode("utf-8")) + len(row["text"].encode("utf-8")) for row in chunks)
    if count > MAX_CHUNKS or size > MAX_REQUEST:
        raise ValueError("vector build window exceeds limits")
    root = load_runtime(request["runtime"])
    paths = [database_path(root, item["database"], writable=True) for item in databases]
    if len(set(paths)) != len(paths):
        raise ValueError("duplicate vector build partition")
    started = time.monotonic()
    deadline = started + seconds
    initialized = False
    initialization_failed = False
    initialization_lock = threading.Lock()

    def encoder(runtime):
        nonlocal initialized, initialization_failed
        with initialization_lock:
            if initialization_failed:
                raise ValueError("shared vector encoder initialization failed")
            if not initialized:
                try:
                    initialize_encoder(runtime)
                except BaseException:
                    initialization_failed = True
                    raise
                initialized = True

    results = []
    while len(results) < len(databases):
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            break
        width = min(4, workers, len(databases)-len(results))
        indices = range(len(results), len(results)+width)

        def run(index):
            result = build({**databases[index], "runtime": request["runtime"], "contract_digest": request["contract_digest"],
                            "workers": workers//width, "max_seconds": min(seconds, max(1, math.ceil(remaining)))},
                           encoder=encoder, soft_deadline=deadline, final_deadline=deadline+30.0)
            return {"index": index, "result": {"schema_version": SCHEMA, **result}}

        if width == 1:
            results.append(run(indices.start))
        else:
            # Each coordinator owns its SQLite connection. Await every admitted task,
            # including on a submit/result error, before the caller can quarantine.
            with concurrent.futures.ThreadPoolExecutor(max_workers=width) as pool:
                futures = [pool.submit(run, index) for index in indices]
                completed = [future.result() for future in futures]
            results.extend(completed)
    return {"results": results, "not_started": list(range(len(results), len(databases)))}


def query(request: dict) -> dict:
    return query_many({"runtime": request["runtime"], "contract_digest": request["contract_digest"],
        "query": request["query"], "limit": request["limit"], "databases": [{
            "database": request["database"], "manifest_digest": request["manifest_digest"],
            "expected_database_digest": request["expected_database_digest"]}]})


def query_many(request: dict) -> dict:
    validate_digest(request["contract_digest"])
    root = load_runtime(request["runtime"])
    text = request["query"]
    limit = request["limit"]
    if not isinstance(text, str) or not text.strip() or len(text.encode("utf-8")) > 8192:
        raise ValueError("invalid semantic query")
    if type(limit) is not int or not 1 <= limit <= 1000:
        raise ValueError("invalid semantic query limit")
    databases = request["databases"]
    if not isinstance(databases, list) or not 1 <= len(databases) <= 256:
        raise ValueError("invalid vector partition count")
    initialize_encoder(request["runtime"])
    vector = encode_batch([{"title": "", "text": text}])[0]
    matches, seen_paths, seen_ids = [], set(), set()
    for item in databases:
        if not isinstance(item, dict) or set(item) != {"database", "manifest_digest", "expected_database_digest"}:
            raise ValueError("invalid vector partition contract")
        validate_digest(item["manifest_digest"])
        path = database_path(root, item["database"], writable=False)
        if path in seen_paths:
            raise ValueError("duplicate vector partition")
        seen_paths.add(path)
        identity = authenticate_database(path, item["expected_database_digest"], create=False)
        connection = open_database(path, readonly=True)
        try:
            check_database_identity(path, identity)
            meta = dict(connection.execute("SELECT key,value FROM meta"))
            if meta.get("schema_version") != str(SCHEMA) or meta.get("phase") != "ready" or meta.get("contract_digest") != request["contract_digest"] or meta.get("manifest_digest") != item["manifest_digest"]:
                raise ValueError("stale or mismatched vector generation")
            rows = connection.execute("SELECT d.chunk_id,d.digest,v.distance FROM vectors v JOIN documents d ON d.id=v.rowid WHERE v.embedding MATCH ? AND k=? ORDER BY v.distance,d.chunk_id", (vector,limit)).fetchall()
            for row in rows:
                if not math.isfinite(row[2]) or row[0] in seen_ids:
                    raise ValueError("invalid or duplicate vector identity")
                seen_ids.add(row[0])
                matches.append({"chunk_id": row[0], "digest": row[1], "score": 1.0-float(row[2])**2/2.0})
        finally:
            connection.close()
    matches.sort(key=lambda row: (-row["score"],row["chunk_id"]))
    return {"matches": matches[:limit]}


def self_test(request: dict) -> dict:
    initialize_encoder(request["runtime"])
    vector = encode_batch([{"title": "", "text": "A local knowledge retrieval check. 로컬 지식 검색 검사."}])[0]
    prefix = "Common reference context for this document. " * 60
    tails = encode_batch([{"title": "", "text": prefix+"Recover deleted knowledge from backups."}, {"title": "", "text": prefix+"Install a different software release."}])
    if tails[0] == tails[1]:
        raise ValueError("long document tails were discarded")
    import sqlite_vec
    connection = sqlite3.connect(":memory:")
    try:
        connection.enable_load_extension(True)
        sqlite_vec.load(connection)
        connection.enable_load_extension(False)
        version = connection.execute("SELECT vec_version()").fetchone()[0]
        if version != "v0.1.9":
            raise ValueError("unsupported vector engine")
        connection.execute(f"CREATE VIRTUAL TABLE v USING vec0(embedding float[{DIMENSION}])")
        connection.execute("INSERT INTO v(rowid,embedding) VALUES (1,?)", (vector,))
        found = connection.execute("SELECT rowid FROM v WHERE embedding MATCH ? AND k=1", (vector,)).fetchone()
        if found != (1,):
            raise ValueError("vector engine self-test failed")
        return {"dimension": DIMENSION, "engine": version, "offline_file_profile": True, "long_tail_included": True}
    finally:
        connection.close()


def execute_request(request: dict) -> dict:
    offline_environment()
    if not isinstance(request, dict) or type(request.get("schema_version")) is not int or request.get("schema_version") != SCHEMA:
        raise ValueError("unsupported vector worker schema")
    fields = {
        "build": {"database", "chunks", "contract_digest", "workers", "max_seconds", "manifest_digest", "expected_database_digest"},
        "build-many": {"databases", "contract_digest", "workers", "max_seconds"},
        "query": {"database", "query", "limit", "contract_digest", "manifest_digest", "expected_database_digest"},
        "query-many": {"databases", "query", "limit", "contract_digest"},
        "self-test": set(),
    }
    action = request.get("action")
    if action not in fields or set(request) != {"schema_version", "action", "runtime"} | fields[action]:
        raise ValueError("invalid vector worker fields")
    if request.get("action") == "build":
        result = build(request)
    elif request.get("action") == "build-many":
        result = build_many(request)
    elif request.get("action") == "query":
        result = query(request)
    elif request.get("action") == "query-many":
        result = query_many(request)
    elif request.get("action") == "self-test":
        result = self_test(request)
    else:
        raise ValueError("unsupported vector worker action")
    return result


def main() -> int:
    offline_environment()
    try:
        raw = sys.stdin.buffer.read(MAX_REQUEST + 1)
        if len(raw) > MAX_REQUEST:
            raise ValueError("vector request exceeds limits")
        request = json.loads(raw)
        result = execute_request(request)
        print(json.dumps({"schema_version": SCHEMA, "status": "success", **result}, allow_nan=False))
        return 0
    except Exception as error:
        # Never echo source text, query text, arbitrary paths, or dependency diagnostics.
        print(json.dumps({"schema_version": SCHEMA, "status": "error", "error_type": type(error).__name__}))
        return 10


if __name__ == "__main__":
    raise SystemExit(main())
