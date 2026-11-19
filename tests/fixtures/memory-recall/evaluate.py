"""Re-measure the frozen OVX-002 retrieval set with a development Hive binary.

Usage: evaluate.py <hive-binary> <result.json>
Uses disposable user roots under tests/work; no user installation is touched and no model runs.
Modes: baseline (OVX-002 commands), subjects (facts carry frozen subjects), subjects+expansions
(retrieve also passes the frozen host expansions). Inputs: docs/research/overhaul/OVX-002-inputs.json
and tests/fixtures/memory-recall/host-inputs.json, both fixed before the first run.
"""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))
from tests.conformance.support.harness import write_operational_user_setup  # noqa: E402

binary, output = Path(sys.argv[1]).resolve(), Path(sys.argv[2])
inputs_path = ROOT / "docs/research/overhaul/OVX-002-inputs.json"
host_path = ROOT / "tests/fixtures/memory-recall/host-inputs.json"
inputs = json.loads(inputs_path.read_text(encoding="utf-8"))
host = json.loads(host_path.read_text(encoding="utf-8"))
facts = {fact_id: text for fact_id, text, _ in inputs["facts"]}
facts.update({fact_id: text for fact_id, text in inputs["distractors"]})
queries = [(query, fact_id, "held-out" if index == 3 else "development")
           for fact_id, _, fact_queries in inputs["facts"] for index, query in enumerate(fact_queries)]
assert len(queries) == 40 and set(host["expansions"]) == {query for query, _, _ in queries}
# Precision guard added 2026-11-16 after the stage review: no stored fact answers these, so
# every returned hit is a false positive. Fixed before its first run; never tuned on results.
NEGATIVE_QUERIES = ["what is the plan to do an update", "how do I set up the printer",
                    "weekend hiking trail", "database migration rollback", "is it going to rain",
                    "favorite movie genre", "주말 등산 코스", "데이터베이스 이전 되돌리기"]


def hive(*arguments):
    result = subprocess.run([str(binary), *arguments, "--output", "json"], capture_output=True,
                            text=True, timeout=120, stdin=subprocess.DEVNULL)
    assert result.returncode == 0, (arguments, result.stdout, result.stderr)
    return json.loads(result.stdout)


def run_mode(work, mode):
    user = work / mode.replace("+", "-")
    write_operational_user_setup(user)
    if mode == "baseline":
        for fact_id, text in facts.items():
            hive("knowledge", "remember", "--user-root", str(user), "--user-statement", text,
                 "--claim-key", f"ovx-{fact_id}", "--kind", "preference")
    else:
        ordered = list(facts.items())
        for start in range(0, len(ordered), 8):
            batch = work / f"{mode}-{start}.json"
            batch.write_text(json.dumps({"schema_version": 1, "facts": [
                {"normalized_fact": text, "claim_key": f"ovx-{fact_id}", "kind": "preference",
                 "subject": host["subjects"][fact_id]} for fact_id, text in ordered[start:start + 8]]},
                ensure_ascii=False), encoding="utf-8")
            hive("knowledge", "remember", "--user-root", str(user), "--batch", str(batch))
    rows = []
    for query, fact_id, split in queries:
        arguments = ["knowledge", "retrieve", "--user-root", str(user), "--target", str(user),
                     "--scope", "auto", "--query", query, "--top-k", "5", "--byte-budget", "16384"]
        if mode == "subjects+expansions":
            arguments += ["--expansions", ",".join(host["expansions"][query])]
        hits = hive(*arguments)["data"]["hits"]
        texts = [hit["text"] for hit in hits]
        rank = next((index + 1 for index, text in enumerate(texts) if facts[fact_id] in text), None)
        groups, seen, group_rank = 0, set(), None
        for hit in hits:
            key = hit.get("subject") or hit["chunk_id"]
            if key not in seen:
                seen.add(key)
                groups += 1
            if group_rank is None and facts[fact_id] in hit["text"]:
                group_rank = groups
        rows.append({"query": query, "answer": fact_id, "split": split, "returned": len(hits),
                     "hit_rank": rank, "group_rank": group_rank,
                     "hit_at_5": rank is not None and rank <= 5,
                     "group_at_5": group_rank is not None and group_rank <= 5})
    negatives = []
    for query in NEGATIVE_QUERIES:
        arguments = ["knowledge", "retrieve", "--user-root", str(user), "--target", str(user),
                     "--scope", "auto", "--query", query, "--top-k", "5", "--byte-budget", "16384"]
        negatives.append({"query": query, "returned": len(hive(*arguments)["data"]["hits"])})
    summary = {"negatives": {"queries": len(negatives),
                             "with_hits": sum(1 for row in negatives if row["returned"]),
                             "total_hits": sum(row["returned"] for row in negatives)}}
    for split in ("all", "development", "held-out"):
        selected = [row for row in rows if split == "all" or row["split"] == split]
        summary[split] = {"queries": len(selected),
                          "hit_at_5": sum(row["hit_at_5"] for row in selected),
                          "group_at_5": sum(row["group_at_5"] for row in selected)}
    return {"mode": mode, "summary": summary, "rows": rows, "negatives": negatives}


(ROOT / "tests/work").mkdir(exist_ok=True)
work = Path(tempfile.mkdtemp(prefix="memory-recall-", dir=ROOT / "tests/work"))
try:
    version = subprocess.run([str(binary), "--version"], capture_output=True, text=True).stdout.strip()
    report = {
        "binary_version": version,
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
        "inputs_sha256": hashlib.sha256(inputs_path.read_bytes()).hexdigest(),
        "host_inputs_sha256": hashlib.sha256(host_path.read_bytes()).hexdigest(),
        "modes": [run_mode(work, mode) for mode in ("baseline", "subjects", "subjects+expansions")],
    }
    output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({mode["mode"]: mode["summary"] for mode in report["modes"]}, ensure_ascii=False))
finally:
    shutil.rmtree(work)
