"""Bridge the Rust signed-Prepare test to the fake-only host reference."""
import json
from pathlib import Path
import sys

from handler import consume

root = Path(sys.argv[1])
context = json.loads(Path(sys.argv[2]).read_text(encoding='utf-8'))
context['capability_bytes'] = Path(context.pop('capability_file')).read_bytes()
receipt = consume(root / '.hive/runs/run-1/inbox/action-1.json', root, context)
assert receipt['outcome'] == 'succeeded', receipt
assert consume(root / '.hive/runs/run-1/inbox/action-1.json', root, context) == receipt
assert len((root / 'fake-launches.jsonl').read_text().splitlines()) == 1
print(json.dumps(receipt))
