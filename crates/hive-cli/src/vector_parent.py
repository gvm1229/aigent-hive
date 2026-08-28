"""Lifetime guard for a foreground helper, not a persistent/background service."""
import os as _hive_parent_os
import threading as _hive_parent_threading
import time as _hive_parent_time
import hashlib as _hive_parent_hashlib


def _hive_watch_parent(expected):
    while _hive_parent_os.getppid() == expected:
        _hive_parent_time.sleep(0.1)
    # No cleanup or publication is permitted after the owning CLI disappears. The CLI
    # will recover the previous authenticated checkpoint, never this mutable staging file.
    _hive_parent_os._exit(130)


def _hive_bind_parent(expected):
    if type(expected) is not int or expected <= 1:
        raise ValueError("invalid vector parent identity")
    # Windows uses the existing kill-on-close Job Object. Its getppid() intentionally
    # keeps the old PID after death, so the Unix reparenting test cannot be used there.
    if _hive_parent_os.name == "posix":
        if _hive_parent_os.getppid() != expected:
            _hive_parent_os._exit(130)
        _hive_parent_threading.Thread(target=_hive_watch_parent, args=(expected,), daemon=True).start()


def _hive_run_verified_file(path, expected_digest):
    # Keep the launcher below Windows' command-line limit; execute the bounded bytes
    # just checked, never reopen the mutable file between hashing and compilation.
    with open(path, "rb") as stream:
        code = stream.read(1024 * 1024 + 1)
    if len(code) > 1024 * 1024 or _hive_parent_hashlib.sha256(code).hexdigest() != expected_digest:
        raise ValueError("vector worker changed before execution")
    exec(compile(code, path, "exec"), {"__name__": "__main__", "__file__": path})
