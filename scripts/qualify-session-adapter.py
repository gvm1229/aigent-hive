"""Explicit, scoped Windows acceptance of the production desktop adapter. No model launch."""
from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import queue
import re
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "crates/hive-cli/src/usage_monitor/desktop.ps1"


class AppPin:
    """Read-only handle; never obtains termination or suspension rights."""

    def __init__(self, pid: int, ticks: int):
        self.pid = pid
        self.kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        self.kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        self.kernel.OpenProcess.restype = wintypes.HANDLE
        self.kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        self.kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
        self.kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        self.handle = self.kernel.OpenProcess(0x101000, False, pid)
        if not self.handle:
            raise RuntimeError("Cannot pin original app")
        times = [wintypes.FILETIME() for _ in range(4)]
        if not self.kernel.GetProcessTimes(self.handle, *(ctypes.byref(t) for t in times)):
            self.close()
            raise RuntimeError("Cannot verify original app creation time")
        created = (times[0].dwHighDateTime << 32) | times[0].dwLowDateTime
        if created + 504911232000000000 != ticks:
            self.close()
            raise RuntimeError("Original app creation time differs")

    def alive_and_visible(self) -> bool:
        if self.kernel.WaitForSingleObject(self.handle, 0) != 258:
            return False
        user = ctypes.WinDLL("user32", use_last_error=True)
        callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
        user.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
        user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
        user.IsWindowVisible.argtypes = [wintypes.HWND]
        found = []

        def inspect(window, _):
            owner = wintypes.DWORD()
            user.GetWindowThreadProcessId(window, ctypes.byref(owner))
            if owner.value == self.pid and user.IsWindowVisible(window):
                found.append(True)
            return True

        user.EnumWindows(callback_type(inspect), 0)
        return bool(found)

    def close(self):
        if self.handle:
            self.kernel.CloseHandle(self.handle)
            self.handle = None


def read_packets(stream, packets):
    try:
        while line := stream.readline(16385):
            if len(line) > 16384:
                break
            packets.put(json.loads(line), timeout=1)
    except (ValueError, queue.Full):
        pass
    finally:
        try:
            packets.put(None, timeout=1)
        except queue.Full:
            pass


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--thread", required=True)
    parser.add_argument("--app-pid", type=int, required=True)
    parser.add_argument("--app-created-ticks", type=int, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--interrupt", action="store_true")
    parser.add_argument("--hold-seconds", type=int, default=1, choices=range(0, 61))
    args = parser.parse_args()
    if os.name != "nt" or not re.fullmatch(r"[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}", args.thread):
        parser.error("Exact Windows app and thread identity required")
    output = args.output.resolve()
    if not output.is_relative_to(ROOT / "tests/work/usage-monitor-acceptance"):
        parser.error("Receipt outside owned acceptance fixtures")
    output.parent.mkdir(parents=True, exist_ok=True)
    record = {
        "schema_version": 1, "phase": "started", "scope": "single-test-thread",
        "target_digest": "sha256:" + hashlib.sha256(args.thread.encode()).hexdigest(),
        "adapter_sha256": hashlib.sha256(SCRIPT.read_bytes()).hexdigest(),
        "app_pid": args.app_pid, "package_version": "26.1013.4894.0",
        "model_started": False, "interrupt_requested": args.interrupt,
    }
    app = None
    helper = None
    try:
        app = AppPin(args.app_pid, args.app_created_ticks)
        if not app.alive_and_visible():
            raise RuntimeError("Original app is not visible")
        executable = Path(os.environ["SystemRoot"]) / "System32/WindowsPowerShell/v1.0/powershell.exe"
        helper = subprocess.Popen(
            [str(executable), "-NoLogo", "-NoProfile", "-NonInteractive", "-File", str(SCRIPT),
             "-QualificationThread", args.thread],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            creationflags=subprocess.CREATE_NO_WINDOW,
        )
        packets = queue.Queue(maxsize=32)
        threading.Thread(target=read_packets, args=(helper.stdout, packets), daemon=True).start()

        def send(value):
            helper.stdin.write(json.dumps(value, separators=(",", ":")).encode() + b"\n")
            helper.stdin.flush()

        def receive(deadline):
            while time.monotonic() < deadline:
                try:
                    packet = packets.get(timeout=0.2)
                except queue.Empty:
                    if helper.poll() is not None:
                        raise RuntimeError("Adapter exited")
                    continue
                if packet is None:
                    raise RuntimeError("Adapter output ended")
                record["last_packet"] = packet
                if packet.get("connected"):
                    if packet.get("server_pid") != args.app_pid:
                        raise RuntimeError("Adapter connected to a different app")
                    return packet
                if packet.get("error"):
                    raise RuntimeError(packet["error"])
            raise RuntimeError("Adapter deadline exceeded")

        send({"operation": "discover", "valid": True, "ids": [args.thread]})
        deadline = time.monotonic() + 12
        while True:
            packet = receive(deadline)
            if packet.get("known_sessions", 0) > 1:
                raise RuntimeError("Qualification escaped its exact scope")
            if packet.get("known_sessions") == 1 and packet.get("unknown_sessions") == 0:
                break
        record["metadata_only_ready"] = True
        if args.interrupt:
            incident = "sha256:" + hashlib.sha256((args.thread + str(time.monotonic_ns())).encode()).hexdigest()
            send({"operation": "stop", "incident": incident, "detected_at_ms": time.time_ns() // 1000000})
            deadline = time.monotonic() + 6
            while True:
                packet = receive(deadline)
                stop = packet.get("stop") or {}
                if stop.get("verified"):
                    if stop.get("targeted", 0) < 1 or stop.get("elapsed_ms", 99999) > 5000 or stop.get("request_ms", 99999) > 2000:
                        raise RuntimeError("Actual interruption missed acceptance deadline")
                    record["confirmed_stop"] = stop
                    break
            record["phase"] = "holding-protection"
            output.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
            until = time.monotonic() + args.hold_seconds
            seen = {record["confirmed_stop"]["scope_digest"]}
            record["subsequent_stops"] = []
            while time.monotonic() < until:
                stop = receive(time.monotonic() + 2).get("stop") or {}
                scope = stop.get("scope_digest")
                if stop.get("verified") and scope not in seen:
                    if stop.get("elapsed_ms", 99999) > 5000 or stop.get("request_ms", 99999) > 2000:
                        raise RuntimeError("Latched restart interruption missed deadline")
                    seen.add(scope)
                    record["subsequent_stops"].append(stop)
            send({"operation": "clear"})
            deadline = time.monotonic() + 2
            while receive(deadline).get("control_stopping"):
                pass
            record["subscriptions_released"] = True
        record["app_preserved"] = app.alive_and_visible()
        if not record["app_preserved"]:
            raise RuntimeError("Original app did not survive")
        record["phase"] = "passed-scoped-interruption" if args.interrupt else "passed-metadata-only"
        return 0
    except Exception as error:
        record["phase"] = "failed"
        record["error"] = str(error)
        return 1
    finally:
        if helper is not None:
            # Only this explicitly spawned PowerShell helper; never a Codex process.
            if helper.poll() is None:
                helper.terminate()
            helper.wait(timeout=5)
        if app is not None:
            app.close()
        output.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(record, ensure_ascii=False, separators=(",", ":")))


if __name__ == "__main__":
    raise SystemExit(main())
