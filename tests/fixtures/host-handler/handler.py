"""Nonshipping host reference. Only the adjacent fake Codex can run here.

SummonEnvelope and WorkerProfile are produced by Hive, never translated to the
old lab schema. This conformance adapter grants no product execution authority.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import uuid
from datetime import datetime, timezone


def digest(data):
    return 'sha256:' + hashlib.sha256(data).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()


def read(path):
    return json.loads(path.read_text(encoding='utf-8'))


def atomic(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + '.' + uuid.uuid4().hex)
    temporary.write_bytes(canonical(value))
    os.replace(temporary, path)


def safe(root, relative):
    path = root / relative
    if Path(relative).is_absolute() or not path.resolve().is_relative_to(root.resolve()):
        raise ValueError('path escape')
    for item in (path, *path.parents):
        if item == root.parent:
            break
        if item.is_symlink() or (hasattr(item, 'is_junction') and item.is_junction()):
            raise ValueError('linked path')
    return path


def validate(root, envelope, context):
    if digest(canonical(envelope)) != context['request_digest']:
        raise ValueError('request digest mismatch')
    if envelope['schema_version'] != 1 or envelope['enabled'] is not False or envelope['state'] != 'prepared':
        raise ValueError('inert product envelope required')
    if envelope['target_digest'] != digest(str(root).encode()):
        raise ValueError('target mismatch')
    if envelope['capability_digest'] != digest(context['capability_bytes']):
        raise ValueError('capability mismatch')
    for key in ('event_head', 'control_epoch', 'authority_id', 'idempotency_key'):
        if envelope[key] != context[key]:
            raise ValueError('host binding mismatch: ' + key)
    if context['fencing_token'] < 1 or context['event_head'] == 'none':
        raise ValueError('committed host binding required')
    profile = envelope['profile']
    if profile['provider'] != 'codex' or not profile['supported'] or profile['input_mode'] != 'stdin-task' or profile['result_format'] != 'text':
        raise ValueError('unsupported worker')
    if not profile['argv'] or profile['argv'][0] != 'codex':
        raise ValueError('Codex profile required')
    for key in ('task_file', 'result_path'):
        if not envelope[key].startswith('.hive/runs/' + envelope['run_id'] + '/'):
            raise ValueError('path outside exact run')
        safe(root, envelope[key])
    if envelope['task_file'] == envelope['result_path']:
        raise ValueError('task/result collision')
    if digest(safe(root, envelope['task_file']).read_bytes()) != envelope['task_digest']:
        raise ValueError('task digest mismatch')


def consume(envelope_path, root, context, *, timeout=5):
    """Host-authenticated context is test data, not a Hive Prepare authority.

    shortcut: fake-only executable, replace only after future real-host lifecycle
    qualification; the 0.15.0 product has no activation path.
    """
    envelope = read(envelope_path)
    validate(root, envelope, context)
    run = safe(root, '.hive/runs/' + envelope['run_id'])
    state = run / 'host-reference' / envelope['idempotency_key']
    state.mkdir(parents=True, exist_ok=True)
    receipt_path = state / 'receipt.json'
    try:
        with (state / 'claim.json').open('x', encoding='utf-8') as claim:
            json.dump({'request_digest': context['request_digest']}, claim)
    except FileExistsError:
        if read(state / 'claim.json')['request_digest'] != context['request_digest']:
            raise ValueError('idempotency key belongs to a different request')
        if receipt_path.exists():
            receipt = read(receipt_path)
            if receipt['source_digest'] != context['request_digest']:
                raise ValueError('receipt request mismatch')
            return receipt
        raise ValueError('dispatch-uncertain: claim without receipt') from None
    result = safe(root, envelope['result_path'])
    result.parent.mkdir(parents=True, exist_ok=True)
    candidate = state / 'candidate.txt'
    cancel = state / 'cancel'
    issued = datetime.now(timezone.utc).isoformat()
    outcome = 'blocked'
    native_id = 'not-launched'
    reason = 'cancel-before-launch'
    if not cancel.exists():
        replacements = {'{target}': str(root), '{result_path}': str(candidate)}
        argv = [replacements.get(arg, arg) for arg in envelope['profile']['argv'][1:]]
        # The executable is fixed: a profile cannot select a real model process.
        command = [sys.executable, str(Path(__file__).with_name('fake_codex.py')), *argv]
        atomic(state / 'intent.json', {'argv': command, 'request_digest': context['request_digest']})
        try:
            process = subprocess.Popen(command, cwd=root, stdin=subprocess.PIPE,
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            native_id = str(process.pid)
            process.stdin.write(safe(root, envelope['task_file']).read_bytes())
            process.stdin.close()
            deadline = time.monotonic() + timeout
            reason = None
            while process.poll() is None:
                if cancel.exists() or time.monotonic() >= deadline:
                    reason = 'cancelled' if cancel.exists() else 'timeout'
                    process.kill()
                    process.wait(timeout=5)
                    break
                time.sleep(0.01)
            outcome = 'succeeded' if process.returncode == 0 and candidate.is_file() and not reason else 'blocked'
        except (OSError, subprocess.SubprocessError) as error:
            reason = type(error).__name__
    if cancel.exists():
        outcome, reason = 'blocked', 'cancelled'
    if candidate.exists():
        os.replace(candidate, result if outcome == 'succeeded' else state / 'quarantine.txt')
    now = datetime.now(timezone.utc).isoformat()
    receipt = dict(schema_version=1, receipt_id='receipt-' + envelope['action_id'],
                   kind='final-result', run_id=envelope['run_id'], action_id=envelope['action_id'],
                   event_head=envelope['event_head'], control_epoch=envelope['control_epoch'],
                   idempotency_key=envelope['idempotency_key'], fencing_token=context['fencing_token'],
                   host='codex', native_task_id=native_id, outcome=outcome,
                   evidence_locator=envelope['result_path'] if outcome == 'succeeded' else envelope_path.relative_to(root).as_posix(),
                   host_capability_digest=envelope['capability_digest'],
                   source_locator=envelope_path.relative_to(root).as_posix(),
                   source_digest=context['request_digest'], issued_at=issued, received_at=now,
                   provenance=json.dumps({'fixture_only': True, 'reason': reason,
                                          'result_digest': digest(result.read_bytes()) if outcome == 'succeeded' else None}))
    atomic(receipt_path, receipt)
    return receipt
