#!/usr/bin/env python3
"""Build resumable research embeddings and evaluate the 0.10 vector hard gate."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import struct
import time
from pathlib import Path
from typing import Iterable


SCHEMA_VERSION = 1
DEFAULT_MODEL = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def digest_text(value: str) -> str:
    return digest_bytes(value.encode("utf-8"))


def atomic_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    os.replace(temporary, path)


def load_jsonl(path: Path) -> list[dict[str, str]]:
    rows = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        value = json.loads(line)
        if not isinstance(value, dict) or not all(
            isinstance(value.get(key), str) for key in ("id", "text", "scope")
        ):
            raise ValueError(f"invalid corpus row at line {line_number}")
        rows.append({key: value[key] for key in ("id", "text", "scope")})
    if not rows or len({row["id"] for row in rows}) != len(rows):
        raise ValueError("corpus must contain non-empty unique ids")
    return rows


def write_jsonl(path: Path, rows: Iterable[dict[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".tmp")
    with temporary.open("w", encoding="utf-8", newline="\n") as stream:
        for row in rows:
            stream.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    os.replace(temporary, path)


def source_documents(path: Path, max_chars: int) -> list[str]:
    value = json.loads(path.read_text(encoding="utf-8"))
    documents = value.get("documents") if isinstance(value, dict) else None
    if not isinstance(documents, list) or not documents:
        raise ValueError("source corpus does not contain documents")
    result = []
    for item in documents:
        text = item.get("text") if isinstance(item, dict) else None
        if isinstance(text, str) and text.strip():
            result.append(" ".join(text.split())[:max_chars])
    if not result:
        raise ValueError("source corpus documents are empty")
    return result


def generate(args: argparse.Namespace) -> dict[str, object]:
    base = source_documents(args.source_corpus, args.max_chars)
    scopes = ("source", "project", "user-root", "shared", "private", "confidential")
    rows = []
    digests = set()
    for index in range(args.count):
        text = base[index % len(base)]
        if args.kind == "unique":
            text = f"자료 {index:05d}. {text} 관련 기록 {index // len(base):04d}."
        digests.add(digest_text(text))
        rows.append({"id": f"doc-{index:05d}", "scope": scopes[index % len(scopes)], "text": text})
    write_jsonl(args.output, rows)
    result = {
        "schema_version": SCHEMA_VERSION,
        "action": "generate",
        "kind": args.kind,
        "count": len(rows),
        "unique_text_digests": len(digests),
        "deduplicated_embeddings": len(digests),
        "corpus_digest": digest_bytes(args.output.read_bytes()),
        "average_chars": sum(len(row["text"]) for row in rows) / len(rows),
    }
    print(json.dumps(result, ensure_ascii=False, sort_keys=True))
    return result


def hash_vectors(texts: list[str], dimension: int) -> list[bytes]:
    vectors = []
    for text in texts:
        seed = hashlib.sha256(text.encode("utf-8")).digest()
        values = [((seed[index % len(seed)] / 255.0) * 2.0) - 1.0 for index in range(dimension)]
        vectors.append(struct.pack(f"<{dimension}f", *values))
    return vectors


def fastembed_vectors(
    texts: list[str], model_name: str, model_cache: Path, threads: int, batch_size: int
) -> tuple[list[bytes], int]:
    try:
        from fastembed import TextEmbedding
    except ImportError as error:
        raise RuntimeError("fastembed backend requires the isolated research environment") from error
    model = TextEmbedding(
        model_name=model_name,
        cache_dir=str(model_cache),
        threads=threads,
    )
    values = list(model.embed(texts, batch_size=batch_size))
    if not values:
        return [], 0
    dimension = len(values[0])
    return [struct.pack(f"<{dimension}f", *map(float, value)) for value in values], dimension


def generation_id(rows: list[dict[str, str]], model_name: str, backend: str) -> str:
    digest = hashlib.sha256()
    digest.update(model_name.encode())
    digest.update(b"\0")
    digest.update(backend.encode())
    for row in rows:
        digest.update(row["id"].encode())
        digest.update(b"\0")
        digest.update(digest_text(row["text"]).encode())
        digest.update(b"\0")
        digest.update(row["scope"].encode())
    return digest.hexdigest()


def read_previous_entries(state_dir: Path) -> dict[str, str]:
    pointer = state_dir / "current.json"
    if not pointer.exists():
        return {}
    current = json.loads(pointer.read_text(encoding="utf-8"))
    entries_path = state_dir / "generations" / current["generation_id"] / "entries.jsonl"
    return {row["id"]: row["text_digest"] for row in load_generation_entries(entries_path)}


def load_generation_entries(path: Path) -> list[dict[str, str]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]


def build(args: argparse.Namespace) -> dict[str, object]:
    rows = load_jsonl(args.corpus)
    scope_root = args.state_dir
    if args.scope:
        rows = [row for row in rows if row["scope"] == args.scope]
        if not rows:
            raise ValueError("selected scope has no corpus rows")
        args.state_dir = scope_root / "scopes" / hashlib.sha256(args.scope.encode()).hexdigest()
    args.state_dir.mkdir(parents=True, exist_ok=True)
    generation = generation_id(rows, args.model, args.backend)
    staging = args.state_dir / "staging" / generation
    cache = args.state_dir / "cache" / args.model.replace("/", "--")
    staging.mkdir(parents=True, exist_ok=True)
    cache.mkdir(parents=True, exist_ok=True)
    previous = read_previous_entries(args.state_dir)
    current = {row["id"]: digest_text(row["text"]) for row in rows}
    added = sum(identifier not in previous for identifier in current)
    changed = sum(identifier in previous and previous[identifier] != digest for identifier, digest in current.items())
    deleted = sum(identifier not in current for identifier in previous)
    unique: dict[str, str] = {}
    for row in rows:
        unique.setdefault(current[row["id"]], row["text"])
    missing = [(digest, text) for digest, text in unique.items() if not (cache / f"{digest[7:]}.f32").exists()]
    started = time.perf_counter()
    embedded = 0
    completed_batches = 0
    dimension = args.dimension if args.backend == "hash" else 0
    for offset in range(0, len(missing), args.batch_size):
        batch = missing[offset : offset + args.batch_size]
        texts = [text for _, text in batch]
        if args.backend == "hash":
            vectors = hash_vectors(texts, args.dimension)
        else:
            vectors, dimension = fastembed_vectors(
                texts, args.model, args.model_cache, args.threads, args.batch_size
            )
        for (digest, _), vector in zip(batch, vectors, strict=True):
            output = cache / f"{digest[7:]}.f32"
            temporary = output.with_suffix(".tmp")
            temporary.write_bytes(vector)
            os.replace(temporary, output)
        embedded += len(batch)
        completed_batches += 1
        atomic_json(
            staging / "checkpoint.json",
            {
                "schema_version": SCHEMA_VERSION,
                "generation_id": generation,
                "embedded": embedded,
                "remaining": len(missing) - embedded,
                "completed_batches": completed_batches,
            },
        )
        if args.max_batches and completed_batches >= args.max_batches and embedded < len(missing):
            result = {
                "schema_version": SCHEMA_VERSION,
                "action": "build",
                "status": "interrupted",
                "generation_id": generation,
                "embedded": embedded,
                "remaining": len(missing) - embedded,
                "activated": False,
                "scope": args.scope,
                "elapsed_seconds": time.perf_counter() - started,
            }
            if args.output:
                atomic_json(args.output, result)
            print(json.dumps(result, sort_keys=True))
            return result
    entries = [
        {
            "id": row["id"],
            "scope": row["scope"],
            "text_digest": current[row["id"]],
            "vector_digest": digest_bytes((cache / f"{current[row['id']][7:]}.f32").read_bytes()),
        }
        for row in rows
    ]
    completed = args.state_dir / "generations" / generation
    completed.parent.mkdir(parents=True, exist_ok=True)
    if not completed.exists():
        temporary = completed.with_name(completed.name + ".tmp")
        temporary.mkdir(parents=True, exist_ok=False)
        write_jsonl(temporary / "entries.jsonl", entries)
        atomic_json(
            temporary / "manifest.json",
            {
                "schema_version": SCHEMA_VERSION,
                "generation_id": generation,
                "model": args.model,
                "backend": args.backend,
                "dimension": dimension,
                "entry_count": len(entries),
                "unique_vector_count": len(unique),
            },
        )
        os.replace(temporary, completed)
    pointer = {
        "schema_version": SCHEMA_VERSION,
        "generation_id": generation,
        "generation_digest": digest_bytes((completed / "entries.jsonl").read_bytes()),
    }
    atomic_json(args.state_dir / "current.json", pointer)
    elapsed = time.perf_counter() - started
    result = {
        "schema_version": SCHEMA_VERSION,
        "action": "build",
        "status": "complete",
        "generation_id": generation,
        "entry_count": len(rows),
        "unique_text_digests": len(unique),
        "embedded": embedded,
        "reused": len(unique) - embedded,
        "added": added,
        "changed": changed,
        "deleted": deleted,
        "dimension": dimension,
        "elapsed_seconds": elapsed,
        "activated": True,
        "scope": args.scope,
        "scope_dir": args.state_dir.relative_to(scope_root).as_posix()
        if args.scope
        else ".",
        "pointer_digest": digest_bytes((args.state_dir / "current.json").read_bytes()),
    }
    if args.output:
        atomic_json(args.output, result)
    print(json.dumps(result, sort_keys=True))
    return result


def decide(args: argparse.Namespace) -> dict[str, object]:
    evidence = json.loads(args.evidence.read_text(encoding="utf-8"))
    checks = {
        "semantic_quality": evidence["semantic_improvement_points"] >= 15
        and evidence["semantic_recall_at_10"] >= 0.90,
        "exact_regression": evidence["hybrid_exact_recall_at_10"] == 1.0,
        "warm_query": evidence["warm_query_p95_ms"] <= 500,
        "cold_query": evidence["cold_query_p95_ms"] <= 2_000,
        "full_build": evidence.get("full_build_50000_seconds") is not None
        and evidence["full_build_50000_seconds"] <= 600,
        "incremental": evidence["incremental_100_seconds"] <= 30,
        "index_size": evidence["index_bytes"] <= 512 * 1024 * 1024,
        "scope_leaks": evidence["scope_leaks"] == 0,
        "offline": evidence["network_calls"] == 0 and evidence["provider_api_calls"] == 0,
        "fallback": evidence["fts_fallback_failures"] == 0,
        "resume": evidence["resume_equivalent"] is True,
        "platforms": set(evidence["accepted_platforms"])
        == {"windows-x64", "macos-arm64", "linux-musl-x64"},
    }
    decision = "adopt" if all(checks.values()) else "defer"
    result = {
        "schema_version": SCHEMA_VERSION,
        "action": "decide",
        "decision": decision,
        "checks": checks,
        "failed_checks": sorted(name for name, passed in checks.items() if not passed),
        "product_dependencies_added": decision == "adopt",
    }
    atomic_json(args.output, result)
    print(json.dumps(result, sort_keys=True))
    return result


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser()
    commands = root.add_subparsers(dest="command", required=True)
    generate_parser = commands.add_parser("generate")
    generate_parser.add_argument("--source-corpus", type=Path, required=True)
    generate_parser.add_argument("--output", type=Path, required=True)
    generate_parser.add_argument("--count", type=int, default=50_000)
    generate_parser.add_argument("--kind", choices=("repeated", "unique"), required=True)
    generate_parser.add_argument("--max-chars", type=int, default=160)
    build_parser = commands.add_parser("build")
    build_parser.add_argument("--corpus", type=Path, required=True)
    build_parser.add_argument("--state-dir", type=Path, required=True)
    build_parser.add_argument("--output", type=Path)
    build_parser.add_argument("--backend", choices=("hash", "fastembed"), default="hash")
    build_parser.add_argument("--model", default=DEFAULT_MODEL)
    build_parser.add_argument("--model-cache", type=Path, default=Path(".cache/fastembed"))
    build_parser.add_argument("--dimension", type=int, default=16)
    build_parser.add_argument("--batch-size", type=int, default=100)
    build_parser.add_argument("--threads", type=int, default=4)
    build_parser.add_argument("--max-batches", type=int, default=0)
    build_parser.add_argument(
        "--scope",
        choices=("source", "project", "user-root", "shared", "private", "confidential"),
    )
    decide_parser = commands.add_parser("decide")
    decide_parser.add_argument("--evidence", type=Path, required=True)
    decide_parser.add_argument("--output", type=Path, required=True)
    return root


def main() -> int:
    args = parser().parse_args()
    if args.command == "generate":
        generate(args)
    elif args.command == "build":
        build(args)
    else:
        decide(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
