#!/usr/bin/env python3
"""Replay the fixed stage-two detection and retrieval inputs against a local Hive binary."""
from __future__ import annotations

import argparse
import json
import subprocess
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--hive', type=Path, required=True)
    parser.add_argument('--work', type=Path, required=True)
    args = parser.parse_args()
    binary = args.hive.resolve(strict=True)
    work = args.work.resolve()
    if not work.is_relative_to(ROOT / 'tests/work') or work.exists():
        parser.error('--work must be a new directory below tests/work')
    work.mkdir(parents=True)

    def run(*arguments: str) -> dict:
        result = subprocess.run([str(binary), *map(str, arguments), '--output', 'json'], cwd=ROOT, capture_output=True, text=True, encoding='utf-8', check=False)
        if result.returncode:
            raise RuntimeError(f'Hive command failed: {arguments[0]}: {result.stderr}')
        return json.loads(result.stdout)

    corpus = ROOT / 'tests/fixtures/project-detection'
    cases = json.loads((corpus / 'expected.json').read_text(encoding='utf-8'))
    assert len(cases) == 30
    inputs = {corpus / case['id'] / name: content.encode() for case in cases for name, content in case['files'].items()}
    for path, expected in inputs.items():
        assert path.read_bytes() == expected, path
    user = work / 'detection-user'
    user.mkdir()
    empty = work / 'empty'
    empty.mkdir()
    detection = []
    for case in cases:
        target = corpus / case['id'] if case['files'] else empty
        value = run('setup', '--target', target, '--user-root', user, '--describe', '--question', 'project-kind')['data']
        profile = value['project_profile']
        question = value['questions'][0]
        detection.append({'id': case['id'], 'expected_category': case['expected_kind'], 'category': profile['category'], 'default': question['question']['default'], 'effective': question['effective']})
        assert profile['category'] == case['expected_kind'], case['id']
    fixed = json.loads((ROOT / 'docs/research/overhaul/OVX-002-inputs.json').read_text(encoding='utf-8'))
    user = work / 'retrieval-user'
    (user / '.hive/config').mkdir(parents=True)
    config = {'schema_version': 1, 'interface_language': 'en', 'wiki': {'enabled': True, 'language': 'both'}, 'profile': {'contexts': ['web-developer']}, 'persona': {'id': 'balanced'}, 'selected_hosts': ['codex'], 'skills': {'mode': 'all'}, 'usage_guard': {'enabled': False, 'stop_remaining_percent': 20, 'codexbar_fallback_enabled': False, 'discord': {'enabled': False}}}
    (user / '.hive/config/user-setup.yml').write_text(json.dumps(config), encoding='utf-8')
    for entry in fixed['facts'] + fixed['distractors']:
        run('knowledge', 'remember', '--user-root', user, '--user-statement', entry[1], '--claim-key', 'ovx-' + entry[0], '--kind', 'preference')
    retrieval = []
    for identity, fact, queries in fixed['facts']:
        for index, query in enumerate(queries):
            value = run('knowledge', 'retrieve', '--user-root', user, '--target', user, '--scope', 'auto', '--query', query, '--top-k', '5', '--byte-budget', '16384')
            retrieval.append({'id': identity, 'query': query, 'holdout': index == 3, 'hit': any(fact in hit['text'] for hit in value['data']['hits'])})
    assert len(retrieval) == 40 and sum(row['holdout'] for row in retrieval) == 10
    for path, expected in inputs.items():
        assert path.read_bytes() == expected, path
    summary = {'detection': {'matched': sum(row['category'] == row['expected_category'] for row in detection), 'total': 30, 'defaults': dict(Counter(row['default'] for row in detection)), 'unaccepted_effective': sum(row['effective'] is None for row in detection)}, 'retrieval': {'hits': sum(row['hit'] for row in retrieval), 'total': 40, 'holdout_hits': sum(row['hit'] and row['holdout'] for row in retrieval), 'holdout_total': 10}, 'fixed_inputs_preserved': True}
    (work / 'results.json').write_text(json.dumps({'summary': summary, 'detection': detection, 'retrieval': retrieval}, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(summary, ensure_ascii=False))


if __name__ == '__main__':
    main()
