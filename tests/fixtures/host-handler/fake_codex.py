"""Fake executable: records product argv/stdin and writes a deterministic result."""
import json
from pathlib import Path
import sys
import time

args = sys.argv[1:]
task = sys.stdin.buffer.read()
target = Path(args[args.index('-C') + 1])
result = Path(args[args.index('-o') + 1])
state = result.parent
with (target / 'fake-launches.jsonl').open('a', encoding='utf-8') as log:
    log.write(json.dumps({'argv': args, 'task': task.decode()}) + '\n')
if task == b'fail':
    sys.exit(1)
result.write_text('REFERENCE_CODEX_OK', encoding='utf-8')
if task == b'cancel-late':
    (state / 'cancel').touch()
if task == b'timeout':
    time.sleep(10)
