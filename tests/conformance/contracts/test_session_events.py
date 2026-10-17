"""Native acceptance marker classification; no app or real conversations are read."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("session_events", ROOT / "scripts/qualify-session-events.py")
EVENTS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EVENTS)


class SessionEvents(unittest.TestCase):
    @unittest.skipUnless(os.name == "nt", "Windows native path spellings")
    def test_native_verbatim_prefix_keeps_the_same_owned_path(self):
        path=Path(__file__).resolve()
        self.assertEqual(EVENTS.native_path("\\\\?\\"+str(path)),path)

    def state(self, rows):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "rollout.jsonl"
            path.write_text("".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8")
            return EVENTS.scan_turn(path)

    def marker(self, kind, turn="current"):
        return {"type": "event_msg", "payload": {"type": kind, "turn_id": turn}}

    def test_native_abort_confirms_only_the_expected_turn(self):
        state = self.state([self.marker("task_started"), self.marker("turn_aborted")])
        self.assertTrue(EVENTS.evaluate({"thread": "fixture", "turn_id": "current"}, state, False)["verified"])
        self.assertFalse(EVENTS.evaluate({"thread": "fixture", "turn_id": "old"}, state, False)["verified"])

    def test_natural_completion_is_not_an_interruption(self):
        state = self.state([self.marker("task_started"), self.marker("task_complete")])
        self.assertFalse(EVENTS.evaluate({"thread": "fixture", "turn_id": "current"}, state, False)["verified"])

    def test_new_turn_after_abort_cannot_reuse_a_stop(self):
        state = self.state([self.marker("task_started"), self.marker("turn_aborted"), self.marker("task_started", "new")])
        self.assertEqual(state["status"], "running")
        self.assertFalse(EVENTS.evaluate({"thread": "fixture", "turn_id": "current"}, state, False)["verified"])

    def test_tool_free_reasoning_requires_no_new_tool_calls(self):
        for kind in ("function_call", "custom_tool_call"):
            state = self.state([self.marker("task_started"), {"type": "response_item", "payload": {"type": kind, "name": "fixture"}}, self.marker("turn_aborted")])
            subject = {"thread": "fixture", "turn_id": "current", "tool_free": True, "tool_calls": 0}
            self.assertFalse(EVENTS.evaluate(subject, state, False)["verified"])
            subject["tool_calls"] = 1
            self.assertTrue(EVENTS.evaluate(subject, state, False)["verified"])

    def test_old_or_quoted_abort_does_not_stop_the_current_turn(self):
        state = self.state([self.marker("task_started"), self.marker("turn_aborted", "old"), {"type": "response_item", "payload": {"type": "message", "text": json.dumps(self.marker("turn_aborted"))}}])
        self.assertEqual(state["status"], "running")
