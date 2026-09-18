"""Plan state is scoped, evidence-bound, deterministic, and read-only by default."""

import hashlib
from pathlib import Path
import tempfile
import unittest
from unittest import mock

from scripts import plan_state as policy


class PlanStateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.write("docs/plans/PLAN.md", "> Product version: `0.11.0`\n\n## Active fragments\n\n"
                   "| Fragment | Checklist | 범위 |\n| --- | --- | --- |\n"
                   "| [one](active/one.md) | A-* | 예제 |\n\n## Summary\n"
                   + policy.START + "\nold\n" + policy.END + "\nTAIL\n")
        self.write("docs/state/CURRENT.md", "USER PREFIX\n" + policy.START + "\nold\n" + policy.END + "\nUSER SUFFIX\n")
        self.fragment("- [ ] [A-001] first\n  - state: agent-owned\n")

    def write(self, path, value):
        p = self.root / path
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(value, encoding="utf-8", newline="\n")
        return p

    def fragment(self, items, *, version="0.11.0", scope="product"):
        return self.write("docs/plans/active/one.md", f"> Plan version: {version}\n> Scope: {scope}\n\n{items}")

    def evidence(self, text="Reviewed evidence\n"):
        p = self.write("docs/research/evidence.md", text)
        return "repo:docs/research/evidence.md#sha256:" + hashlib.sha256(p.read_bytes()).hexdigest()

    def test_ignores_unregistered_historical_and_backlog_items(self):
        self.write("docs/plans/active/history.md", "- [x] [OLD-001] historical\n")
        self.write("docs/plans/backlog/idea.md", "- [x] [OLD-002] candidate\n")
        plan = policy.load_plan(self.root)
        self.assertEqual(set(plan.criteria), {"A-001"})
        self.assertEqual(plan.completed_product_ids(), set())

    def test_versioned_fragment_filename_is_supported(self):
        p = self.root / "docs/plans/PLAN.md"
        p.write_text(p.read_text(encoding="utf-8").replace("active/one.md", "active/work-0.11.0.md"), encoding="utf-8")
        (self.root / "docs/plans/active/one.md").rename(self.root / "docs/plans/active/work-0.11.0.md")
        self.assertEqual(set(policy.load_plan(self.root).criteria), {"A-001"})

    def test_navigation_links_are_not_checkboxes(self):
        self.fragment("- [ ] [A-001] first\n  - state: agent-owned\n\n- [Related plan](other.md)\n")
        self.assertEqual(set(policy.load_plan(self.root).criteria), {"A-001"})

    def test_completion_requires_state_and_fresh_evidence(self):
        for item in ("- [x] [A-001] done\n  - state: agent-owned\n",
                     "- [x] [A-001] done\n  - state: complete\n",
                     "- [ ] [A-001] pending\n  - state: complete\n"):
            self.fragment(item)
            with self.assertRaises(policy.PlanError): policy.load_plan(self.root)
        evidence = self.evidence()
        self.fragment(f"- [x] [A-001] done\n  - state: complete; evidence: {evidence}\n")
        self.assertEqual(policy.load_plan(self.root).completed_product_ids(), {"A-001"})
        self.write("docs/research/evidence.md", "modified\n")
        with self.assertRaisesRegex(policy.PlanError, "stale"): policy.load_plan(self.root)

    def test_source_completion_never_authorizes_product_release(self):
        evidence = self.evidence()
        self.fragment(f"- [x] [A-001] done\n  - state: complete; evidence: {evidence}\n", scope="source")
        self.assertEqual(policy.load_plan(self.root).completed_product_ids(), set())

    def test_failed_test_receipt_cannot_complete_criterion(self):
        p = self.write("tests/results/runs/fail.md", '```json\n{"status":"failed","exit_code":1}\n```\n')
        evidence = "repo:tests/results/runs/fail.md#sha256:" + hashlib.sha256(p.read_bytes()).hexdigest()
        self.fragment(f"- [x] [A-001] done\n  - state: complete; evidence: {evidence}\n")
        with self.assertRaisesRegex(policy.PlanError, "did not pass"): policy.load_plan(self.root)

    def test_invalid_or_concurrently_changed_test_receipt_is_rejected(self):
        for payload in ('[]', '{"status":"passed","exit_code":false}',
                        '{"status":"passed","exit_code":0,"source_changed_during_run":true}'):
            p = self.write("tests/results/runs/receipt.md", '```json\n' + payload + '\n```\n')
            evidence = 'repo:tests/results/runs/receipt.md#sha256:' + hashlib.sha256(p.read_bytes()).hexdigest()
            self.fragment(f'- [x] [A-001] done\n  - state: complete; evidence: {evidence}\n')
            with self.assertRaises(policy.PlanError): policy.load_plan(self.root)

    def test_rejects_bad_ids_versions_metadata_and_duplicates(self):
        for item in ("- [ ] [RF-A01] first\n  - state: agent-owned\n",
                     "- [ ] [A-001] first\n",
                     "- [ ] [A-001] first\n  - state: unknown\n",
                     "- [ ] [A-001] first\n  - state: agent-owned; state: complete\n",
                     "- [ ] [A-001] first\n  - state: agent-owned; extra: field\n",
                     "- [ ] [A-001] first\n  - state: agent-owned\n  - state: complete\n",
                     "- [ ] [A-001] first\n  - state: agent-owned\n" * 2):
            with self.subTest(item=item):
                self.fragment(item)
                with self.assertRaises(policy.PlanError): policy.load_plan(self.root)
        self.fragment("- [ ] [A-001] first\n  - state: agent-owned\n", version="0.10.3")
        with self.assertRaisesRegex(policy.PlanError, "version"): policy.load_plan(self.root)

    def test_waiting_requires_an_owner_and_concrete_reason(self):
        self.fragment("- [ ] [A-001] host acceptance\n  - state: awaiting-external-evidence\n")
        with self.assertRaises(policy.PlanError): policy.load_plan(self.root)
        self.fragment("- [ ] [A-001] host acceptance\n  - state: awaiting-external-evidence; owner: host; reason: trust not approved\n")
        self.assertEqual(policy.load_plan(self.root).criteria["A-001"].owner, "host")

    def test_dependency_cycles_missing_nodes_and_unmet_completion(self):
        for items in (
            "- [ ] [A-001] first\n  - state: agent-owned; depends: A-002\n",
            "- [ ] [A-001] first\n  - state: agent-owned; depends: A-001\n",
            "- [ ] [A-001] first\n  - state: agent-owned; depends: A-002\n- [ ] [A-002] second\n  - state: agent-owned; depends: A-001\n",
            f"- [x] [A-001] first\n  - state: complete; depends: A-002; evidence: {self.evidence()}\n- [ ] [A-002] second\n  - state: agent-owned\n",
        ):
            self.fragment(items)
            with self.assertRaises(policy.PlanError): policy.load_plan(self.root)

    def test_check_is_read_only_and_generation_preserves_outside_bytes(self):
        before = {p: p.read_bytes() for p in self.root.rglob("*.md")}
        self.assertEqual(len(policy.render(policy.load_plan(self.root))), 2)
        self.assertEqual(before, {p: p.read_bytes() for p in self.root.rglob("*.md")})
        self.assertEqual(len(policy.render(policy.load_plan(self.root), write=True)), 2)
        for path in (self.root / "docs/plans/PLAN.md", self.root / "docs/state/CURRENT.md"):
            prior, after = before[path], path.read_bytes()
            self.assertEqual(prior.split(policy.START.encode())[0], after.split(policy.START.encode())[0])
            self.assertEqual(prior.split(policy.END.encode())[1], after.split(policy.END.encode())[1])
        self.assertEqual(policy.render(policy.load_plan(self.root), write=True), [])

    def test_crlf_and_generation_preflight(self):
        p = self.root / "docs/state/CURRENT.md"
        p.write_bytes(p.read_bytes().replace(b"\n", b"\r\n"))
        policy.render(policy.load_plan(self.root), write=True)
        self.assertNotIn(b"\n", p.read_bytes().replace(b"\r\n", b""))
        before = (self.root / "docs/plans/PLAN.md").read_bytes()
        p.write_bytes(b"missing markers")
        with self.assertRaises(policy.PlanError): policy.render(policy.load_plan(self.root), write=True)
        self.assertEqual(before, (self.root / "docs/plans/PLAN.md").read_bytes())

    def test_changed_inputs_are_not_overwritten(self):
        plan = policy.load_plan(self.root)
        p = self.root / "docs/plans/PLAN.md"
        p.write_bytes(p.read_bytes() + b"concurrent edit\n")
        with self.assertRaisesRegex(policy.PlanError, "changed"): policy.render(plan, write=True)
        self.assertTrue(p.read_bytes().endswith(b"concurrent edit\n"))

    def test_partial_write_failure_is_reported_and_next_generation_converges(self):
        replace = policy.os.replace
        calls = 0

        def fail_second(source, destination):
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("injected second publication failure")
            return replace(source, destination)

        with mock.patch.object(policy.os, "replace", side_effect=fail_second):
            with self.assertRaises(OSError): policy.render(policy.load_plan(self.root), write=True)
        self.assertEqual(policy.render(policy.load_plan(self.root), write=True), ['docs/state/CURRENT.md'])
        self.assertEqual(policy.render(policy.load_plan(self.root)), [])
        self.assertFalse(list(self.root.rglob('.hive-plan-*')))

    def test_rejects_escaping_or_unlinked_registration(self):
        p = self.root / "docs/plans/PLAN.md"
        original = p.read_text(encoding="utf-8")
        for link in ("../secret.md", "active/../secret.md", "active/missing.md"):
            p.write_text(original.replace("active/one.md", link), encoding="utf-8")
            with self.assertRaises(policy.PlanError): policy.load_plan(self.root)


class RepositoryPlanTests(unittest.TestCase):
    def test_registered_plan_and_generated_views_are_current(self):
        root = Path(__file__).resolve().parents[3]
        plan = policy.load_plan(root)
        self.assertFalse(policy.render(plan), "run scripts/check-plan-state.py --write")


if __name__ == "__main__":
    unittest.main()
