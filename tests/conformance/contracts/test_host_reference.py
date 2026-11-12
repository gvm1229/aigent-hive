"""Product-generated SummonEnvelope files through a nonshipping fake host."""
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import unittest

from jsonschema import Draft202012Validator, FormatChecker
from tests.conformance.support.harness import Phase1CliTestCase, REPOSITORY_ROOT
from tests.conformance.contracts.test_roster_gods import GODS

SPEC = importlib.util.spec_from_file_location('host_reference', REPOSITORY_ROOT / 'tests/fixtures/host-handler/handler.py')
HANDLER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HANDLER)


class HostReferenceTests(Phase1CliTestCase):
    def test_public_cli_resolves_all_nine_gods_and_their_aliases(self):
        root = (self.work_root / 'god-consumer').resolve()
        root.mkdir()
        for god in GODS:
            digests = set()
            for name in (god.name, god.korean, *god.aliases):
                output = subprocess.run([
                    str(self.hive_binary), 'summon', name, '--task', 'bounded task',
                    '--target', str(root), '--user-root', str(self.setup_user_root),
                    '--run', 'run-1', '--action', 'action-1', '--expected-head', 'none',
                    '--control-epoch', '0', '--authority-id', 'prepare-1', '--output', 'json',
                ], capture_output=True, text=True, encoding='utf-8')
                self.assertEqual(output.returncode, 0, output.stdout + output.stderr)
                data = json.loads(output.stdout)['data']
                self.assertFalse(data['enabled'])
                self.assertEqual(data['envelope']['role'], god.role)
                self.assertEqual(data['mutation_count'], 0)
                digests.add(data['request_digest'])
            self.assertEqual(len(digests), 1, god.name)
        self.assertFalse((root / '.hive').exists())

    def test_roster_merge_reload_rejection_and_host_report(self):
        root, path, envelope, context = self.prepare()
        def call(*args):
            return subprocess.run([str(self.hive_binary), 'roster', *args,
                                   '--target', str(root), '--user-root', str(self.setup_user_root),
                                   '--output', 'json'], capture_output=True, text=True, encoding='utf-8')
        check = ('check', '--role', 'hive-routine-implementer', '--capabilities', str(root / 'capabilities.json'))
        self.assertEqual(call(*check).returncode, 0)
        global_roster = self.setup_user_root / '.hive/roster.toml'
        global_roster.write_text('[roles.hive-routine-implementer]\nmodel = "exact-global-model"\neffort = "high"\n')
        project_roster = root / '.hive/roster.toml'
        project_roster.write_text('[roles.hive-routine-implementer]\neffort = "low"\n')
        merged = json.loads(call().stdout)['roster']['roles']['hive-routine-implementer']
        self.assertEqual(merged['model'], 'exact-global-model')
        self.assertEqual(merged['effort'], 'low')
        self.assertNotEqual(call(*check).returncode, 0, 'unreported model must be refused')
        project_roster.write_text('[roles.hive-routine-implementer]\neffort = false\n')
        self.assertNotEqual(call().returncode, 0, 'wrong TOML type must be refused')

    def prepare(self, task='bounded task'):
        root = (self.work_root / 'reference-consumer').resolve()
        root.mkdir(exist_ok=True)
        report = dict(provider='codex', version='0.167.2', version_output='codex-cli 0.167.2',
                      help_output='--model -o, --output-last-message --sandbox --skip-git-repo-check --ephemeral -C, --cd',
                      login_status_output='Logged in using ChatGPT',
                      models={'gpt-5.6-luna': ['max']},
                      project_hook_events=['Stop', 'UserPromptSubmit'])
        capability = root / 'capabilities.json'
        capability.write_text(json.dumps(report), encoding='utf-8')
        head = '1:sha256:' + '1' * 64
        args = [str(self.hive_binary), 'summon', 'hive-routine-implementer', '--task', task,
                '--target', str(root), '--user-root', str(self.setup_user_root), '--run', 'run-1',
                '--action', 'action-1', '--expected-head', head, '--control-epoch', '0',
                '--authority-id', 'prepare-1', '--capabilities', str(capability), '--output', 'json']
        output = subprocess.run(args, capture_output=True, text=True, encoding='utf-8', cwd=REPOSITORY_ROOT)
        self.assertEqual(output.returncode, 0, output.stdout + output.stderr)
        data = json.loads(output.stdout)['data']
        envelope = data['envelope']
        self.assertFalse(envelope['enabled'])
        self.assertFalse((root / '.hive').exists(), 'product preview must stay inert')
        path = root / '.hive/runs/run-1/inbox/action-1.json'
        HANDLER.atomic(path, envelope)
        task_path = root / envelope['task_file']
        task_path.parent.mkdir(parents=True)
        task_path.write_bytes(task.encode())
        context = dict(request_digest=data['request_digest'], capability_bytes=capability.read_bytes(),
                       event_head=head, control_epoch=0, authority_id='prepare-1',
                       idempotency_key=envelope['idempotency_key'], fencing_token=1)
        return root, path, envelope, context

    def test_product_envelope_argv_task_receipt_and_duplicate(self):
        root, path, envelope, context = self.prepare()
        receipt = HANDLER.consume(path, root, context)
        schema = json.loads((REPOSITORY_ROOT / 'schemas/host-orchestration-receipt.schema.json').read_text())
        Draft202012Validator(schema, format_checker=FormatChecker()).validate(receipt)
        self.assertEqual(receipt['outcome'], 'succeeded')
        self.assertEqual(receipt['source_digest'], context['request_digest'])
        self.assertEqual(receipt['host_capability_digest'], envelope['capability_digest'])
        self.assertEqual((root / envelope['result_path']).read_text(), 'REFERENCE_CODEX_OK')
        launch = json.loads((root / 'fake-launches.jsonl').read_text())
        expected = [str(root) if arg == '{target}' else str(root / '.hive/runs/run-1/host-reference/run-1-action-1/candidate.txt') if arg == '{result_path}' else arg for arg in envelope['profile']['argv'][1:]]
        self.assertEqual(launch['argv'], expected)
        self.assertEqual(launch['task'], 'bounded task')
        self.assertEqual(HANDLER.consume(path, root, context), receipt)
        self.assertEqual(len((root / 'fake-launches.jsonl').read_text().splitlines()), 1)
        # A claim without its durable receipt must never launch a second worker.
        (root / '.hive/runs/run-1/host-reference/run-1-action-1/receipt.json').unlink()
        with self.assertRaisesRegex(ValueError, 'dispatch-uncertain'):
            HANDLER.consume(path, root, context)
        self.assertEqual(len((root / 'fake-launches.jsonl').read_text().splitlines()), 1)

    def test_host_bindings_and_task_tampering_rejected_before_launch(self):
        root, path, envelope, context = self.prepare()
        for key, value in [('target_digest', 'sha256:' + '0' * 64), ('authority_id', 'other'),
                           ('result_path', '../escape'), ('enabled', True)]:
            bad = copy.deepcopy(envelope)
            bad[key] = value
            HANDLER.atomic(path, bad)
            with self.assertRaises(ValueError):
                HANDLER.consume(path, root, context)
        HANDLER.atomic(path, envelope)
        for key, value in [('capability_bytes', b'other'), ('event_head', '2:sha256:' + '2' * 64),
                           ('control_epoch', 1), ('authority_id', 'other'), ('fencing_token', 0)]:
            with self.assertRaises(ValueError):
                HANDLER.consume(path, root, dict(context, **{key: value}))
        (root / envelope['task_file']).write_text('tampered')
        with self.assertRaisesRegex(ValueError, 'task digest'):
            HANDLER.consume(path, root, context)
        self.assertFalse((root / 'fake-launches.jsonl').exists())

    def test_cancel_before_launch(self):
        root, path, envelope, context = self.prepare()
        state = root / '.hive/runs/run-1/host-reference/run-1-action-1'
        state.mkdir(parents=True)
        (state / 'cancel').touch()
        self.assertEqual(HANDLER.consume(path, root, context)['outcome'], 'blocked')
        self.assertFalse((root / 'fake-launches.jsonl').exists())

    def test_cancelled_late_result_quarantined(self):
        root, path, envelope, context = self.prepare('cancel-late')
        self.assertEqual(HANDLER.consume(path, root, context)['outcome'], 'blocked')
        self.assertFalse((root / envelope['result_path']).exists())
        self.assertTrue((root / '.hive/runs/run-1/host-reference/run-1-action-1/quarantine.txt').exists())

    def test_timeout_quarantines_candidate(self):
        root, path, envelope, context = self.prepare('timeout')
        self.assertEqual(HANDLER.consume(path, root, context, timeout=0.2)['outcome'], 'blocked')
        self.assertFalse((root / envelope['result_path']).exists())
        self.assertEqual(len((root / 'fake-launches.jsonl').read_text().splitlines()), 1)

    def test_failed_worker_has_no_automatic_retry(self):
        root, path, envelope, context = self.prepare('fail')
        receipt = HANDLER.consume(path, root, context)
        self.assertEqual(receipt['outcome'], 'blocked')
        self.assertEqual(HANDLER.consume(path, root, context), receipt)
        self.assertEqual(len((root / 'fake-launches.jsonl').read_text().splitlines()), 1)


if __name__ == '__main__':
    unittest.main()
