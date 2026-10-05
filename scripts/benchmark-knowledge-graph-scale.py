#!/usr/bin/env python3
"""Measure a unique canonical corpus; never substitute synthetic model output."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--hive', type=Path, required=True)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    work = args.work.resolve()
    owned = (ROOT / 'tests/work').resolve()
    if work == owned or not work.is_relative_to(owned) or work.exists():
        raise SystemExit('use a fresh exact child of tests/work')
    work.mkdir(parents=True)
    binary = args.hive.resolve()
    user = work / 'user'
    wiki = user / '.hive/knowledge/Wiki'
    wiki.mkdir(parents=True)
    config = user / '.hive/config'
    config.mkdir(parents=True)
    (config / 'user-setup.yml').write_text('''schema_version: 1
interface_language: en
wiki:
  enabled: true
  language: both
profile:
  id: web-developer
persona:
  id: balanced
selected_hosts: [codex]
skills:
  mode: individual
  selected: [knowledge-capture]
usage_guard:
  enabled: false
  stop_remaining_percent: 20
  codexbar_fallback_enabled: false
''', encoding='utf-8', newline='\n')
    report = {'schema_version': 1, 'host': sys.platform, 'chunks_requested': 50000, 'canonical_documents': 500,
              'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
              'model_calls': 0, 'steps': {}, 'semantic_full_generation': 'not-run: host analysis of 5000 bounded batches required'}
    started = time.monotonic()
    for number in range(500):
        page = f'node-{number:05}'
        target = f'node-{(number + 1) % 500:05}'
        paragraphs = []
        for ordinal in range(100):
            identity = number * 100 + ordinal
            statement = f'Public synthetic passage {identity:05} of {page} depends on [[{target}]]. '
            # Each unique paragraph stays below the existing 1200-byte chunk
            # boundary; two paragraphs cannot collapse into a single chunk.
            paragraphs.append((statement + 'This public example describes a documented relationship. ' * 14)[:900])
        body = '\n\n'.join(paragraphs) + '\n'
        metadata = f'''---
schema_version: 1
id: {page}
kind: concept
summary: Unique record {number:05}
tags: [scale]
aliases: []
sources: []
links: [{target}]
contradictions: []
status: active
created_at: 2026-10-05T00:00:00Z
updated_at: 2026-10-05T00:00:00Z
---

'''
        (wiki / f'{page}.md').write_text(metadata + body, encoding='utf-8', newline='\n')
    report['fixture_seconds'] = round(time.monotonic() - started, 3)

    def invoke(label, *arguments):
        begin = time.monotonic()
        peak = None
        with tempfile.TemporaryFile(dir=work) as output, tempfile.TemporaryFile(dir=work) as errors:
            process = subprocess.Popen([str(binary), *map(str, arguments), '--output', 'json'],
                                       stdout=output, stderr=errors, cwd=work)
            if os.name == 'nt':
                class Counters(ctypes.Structure):
                    _fields_ = [('cb', ctypes.c_ulong), ('faults', ctypes.c_ulong)] + [
                        (name, ctypes.c_size_t) for name in ('peak_ws', 'ws', 'peak_paged', 'paged',
                        'peak_nonpaged', 'nonpaged', 'pagefile', 'peak_pagefile')]
                probe = ctypes.WinDLL('psapi').GetProcessMemoryInfo
                probe.argtypes = [ctypes.c_void_p, ctypes.POINTER(Counters), ctypes.c_ulong]
                probe.restype = ctypes.c_int
            while process.poll() is None:
                if os.name == 'nt':
                    counters = Counters()
                    counters.cb = ctypes.sizeof(counters)
                    if probe(int(process._handle), ctypes.byref(counters), counters.cb):
                        peak = max(peak or 0, counters.peak_ws)
                if time.monotonic()-begin > 300:
                    process.kill()
                    process.wait()
                    report['steps'][label] = {'seconds': round(time.monotonic()-begin, 3),
                        'timeout': True, 'sampled_peak_working_set_bytes': peak}
                    raise RuntimeError(label + ': child timeout')
                time.sleep(.02)
            output.seek(0)
            value = json.load(output)
        report['steps'][label] = {'seconds': round(time.monotonic()-begin, 3),
                                  'exit_code': process.returncode, 'code': value.get('code'),
                                  'sampled_peak_working_set_bytes': peak}
        if process.returncode:
            raise RuntimeError(label + ': ' + str(value.get('message')))
        return value['data']

    try:
        invoke('index', 'index', 'rebuild', '--user-root', user)
        with sqlite3.connect(f'file:{(user / ".hive/index/hive.sqlite3").as_posix()}?mode=ro', uri=True) as connection:
            count, unique = connection.execute('select count(*), count(distinct text) from chunks').fetchone()
        report.update(chunks_indexed=count, unique_chunk_texts=unique)
        if count != 50000 or unique != 50000:
            raise RuntimeError('corpus uniqueness/count mismatch')
        invoke('native_full', 'knowledge', 'graph', 'rebuild', '--target', user)
        common = ['--engine', 'host-semantic', '--target', user, '--user-root', user,
                  '--collection', 'user-root', '--visibility', 'shared', '--host', 'codex']
        preview = invoke('semantic_preview', 'knowledge', 'graph', 'preview', *common)
        invoke('semantic_enable', 'knowledge', 'graph', 'enable', *common, '--consent-digest', preview['consent_digest'])
        prepared = invoke('semantic_prepare', 'knowledge', 'graph', 'prepare', *common)['request']
        report['semantic_batch'] = {k: prepared[k] for k in ('pending_count', 'blocked_oversized_count')}
        report['semantic_batch']['documents'] = len(prepared['documents'])
        report['semantic_batch']['bytes'] = sum(len(d['text'].encode()) for d in prepared['documents'] + prepared['related_documents'])
        for number in (0, 250, 499):
            invoke(f'fts_{number}', 'knowledge', 'query', '--target', user, '--user-root', user,
                   '--text', f'node-{number:05}', '--limit', '10')
        report['status'] = 'measured'
    except Exception as error:
        report.update(status='failed', reason=str(error))
    report['disk_bytes'] = sum(p.stat().st_size for p in user.rglob('*') if p.is_file())
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8', newline='\n')
    print(json.dumps(report))
    return int(report['status'] != 'measured')

if __name__ == '__main__':
    raise SystemExit(main())
