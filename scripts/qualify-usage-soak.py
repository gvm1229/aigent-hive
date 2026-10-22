"""Read-only Windows native-reader qualification; never controls desktop conversations."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import queue
import subprocess
import threading
import time


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--executable", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seconds", type=int, default=1800)
    args = parser.parse_args()
    if args.seconds < 1800:
        parser.error("qualification requires at least 1800 seconds")
    executable = args.executable.resolve(strict=True)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    summary = {
        "schema_version": 1,
        "scope": "Windows native reader, validation and monotonic cadence; no desktop stop",
        "executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
        "started_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "records": [],
    }
    process = subprocess.Popen(
        [str(executable), "__usage-observation-supervisor"],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        text=True, encoding="utf-8", creationflags=subprocess.CREATE_NO_WINDOW,
    )
    replies: queue.Queue = queue.Queue()

    def consume():
        for line in process.stdout:
            try:
                replies.put(json.loads(line))
            except ValueError:
                replies.put({"protocol_error": True})
        replies.put({"supervisor_exited": True})

    threading.Thread(target=consume, daemon=True).start()
    started = time.monotonic()
    due = started
    fault_injected = False
    identities = set()
    try:
        while True:
            time.sleep(max(0, due - time.monotonic()))
            request = {"id": len(summary["records"]) + 1}
            if not fault_injected and time.monotonic() - started >= 600:
                # The source-only supervisor holds the exact query helper it created.
                # This cannot address an app or user process by PID or name.
                request["exit_owned_reader"] = True
                fault_injected = True
            process.stdin.write(json.dumps(request) + "\n")
            process.stdin.flush()
            record = replies.get(timeout=12)
            if record.get("id") != request["id"]:
                raise RuntimeError("supervisor protocol failed")
            record["fault_injected"] = request.get("exit_owned_reader", False)
            worker = record.get("primary_worker_pid")
            if worker:
                # Inventory only the children of our own reader. No command lines/data.
                script = (
                    f"Get-CimInstance Win32_Process -Filter 'ParentProcessId={int(worker)}' | "
                    "Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}} | ConvertTo-Json -Compress"
                )
                inventory = subprocess.run(
                    ["powershell.exe", "-NoProfile", "-NonInteractive", "-Command", script],
                    capture_output=True, text=True, timeout=5,
                    creationflags=subprocess.CREATE_NO_WINDOW,
                )
                if inventory.returncode:
                    raise RuntimeError("owned reader inventory failed")
                children = json.loads(inventory.stdout) if inventory.stdout.strip() else []
                if isinstance(children, dict):
                    children = [children]
                identities.update((child["ProcessId"], child["created"]) for child in children)
            record["query_child_identities_seen"] = len(identities)
            summary["records"].append(record)
            summary["elapsed_seconds"] = round(time.monotonic() - started, 3)
            args.output.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
            if time.monotonic() - started >= args.seconds:
                break
            due = time.monotonic() + record["next_in_ms"] / 1000
        records = summary["records"]
        summary["passed"] = (
            summary["elapsed_seconds"] >= args.seconds
            and len(identities) >= 6
            and any(r["fault_injected"] and r["valid"] and r["source"] == "fresh-process" for r in records)
            and all(not r["blind_before"] for r in records)
            and records[-1]["valid"]
        )
    except (OSError, RuntimeError, ValueError, queue.Empty, subprocess.TimeoutExpired) as error:
        summary["passed"] = False
        summary["failure"] = type(error).__name__ + ": " + str(error)
    finally:
        process.stdin.close()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()  # Only the exact diagnostic supervisor spawned above.
            process.wait(timeout=3)
        args.output.write_text(json.dumps(summary, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps({key: value for key, value in summary.items() if key != "records"}, ensure_ascii=False))
    return 0 if summary.get("passed") else 1


if __name__ == "__main__":
    raise SystemExit(main())
