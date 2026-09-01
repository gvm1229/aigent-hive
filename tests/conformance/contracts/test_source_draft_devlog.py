#!/usr/bin/env python3
"""Source-only draft-devlog transport, authority, and content contracts."""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import sys
import tempfile
import threading
import unittest
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any, Iterator
from unittest import mock


ROOT = Path(__file__).resolve().parents[3]
SKILL = ROOT / ".agents/skills/draft-devlog"
SCRIPT = SKILL / "scripts/portfolio_mcp.py"
_SPEC = importlib.util.spec_from_file_location("draft_devlog_mcp", SCRIPT)
assert _SPEC and _SPEC.loader
MCP = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = MCP
_SPEC.loader.exec_module(MCP)


TOOLS = [
    {"name": "get_schema", "inputSchema": {"type": "object"}},
    {"name": "get_post", "inputSchema": {"type": "object"}},
    {"name": "create_post", "inputSchema": {"type": "object"}},
    {"name": "update_post", "inputSchema": {"type": "object"}},
]
SCHEMA = {
    "posts": {"published": "boolean, default false"},
    "content_components": {"YouTube": "allowed"},
}


class MockState:
    def __init__(self) -> None:
        self.auth_mode = "valid"
        self.calls: list[tuple[str, str | None]] = []
        self.posts: dict[str, dict[str, Any]] = {
            MCP.DEFAULT_REFERENCE: {
                "id": "reference-id",
                "slug": MCP.DEFAULT_REFERENCE,
                "title": "Reference",
                "content": "## Reference\n",
                "published": True,
                "job_field": ["web"],
            }
        }
        self.create_count = 0
        self.close_after_create = False
        self.expire_after_mutation = False
        self.omit_update_tool = False
        self.corrupt_readback = False


class Handler(BaseHTTPRequestHandler):
    server: ThreadingHTTPServer

    def log_message(self, _format: str, *_args: object) -> None:
        return

    def respond(self, status: int, value: dict[str, Any], headers: dict[str, str] | None = None) -> None:
        payload = json.dumps(value).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        for key, item in (headers or {}).items():
            self.send_header(key, item)
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def tool_result(self, request_id: object, value: Any) -> None:
        self.respond(
            200,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "content": [
                        {"type": "text", "text": json.dumps(value, ensure_ascii=False)}
                    ]
                },
            },
        )

    def tool_error(self, request_id: object, message: str) -> None:
        self.respond(
            200,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {"code": -32000, "message": message},
            },
        )

    def do_POST(self) -> None:  # noqa: N802 - stdlib callback name
        state: MockState = self.server.state  # type: ignore[attr-defined]
        length = int(self.headers.get("Content-Length", "0"))
        request = json.loads(self.rfile.read(length))
        request_id = request.get("id")
        if state.auth_mode == "invalid":
            self.respond(
                401,
                {
                    "jsonrpc": "2.0",
                    "id": None,
                    "error": {"code": -32001, "message": "Unauthorized"},
                },
            )
            return
        if state.auth_mode == "rate":
            self.respond(
                429,
                {
                    "jsonrpc": "2.0",
                    "id": None,
                    "error": {"code": -32002, "message": "Too many invalid attempts"},
                },
                {"Retry-After": "120"},
            )
            return

        method = request.get("method")
        if method == "tools/list":
            state.calls.append(("tools/list", None))
            tools = [tool for tool in TOOLS if not (state.omit_update_tool and tool["name"] == "update_post")]
            self.respond(
                200,
                {"jsonrpc": "2.0", "id": request_id, "result": {"tools": tools}},
            )
            return
        if method != "tools/call":
            self.respond(
                404,
                {
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "error": {"code": -32601, "message": "Method not found"},
                },
            )
            return

        params = request["params"]
        name = params["name"]
        arguments = params.get("arguments", {})
        state.calls.append(("tools/call", name))
        if name == "get_schema":
            self.tool_result(request_id, SCHEMA)
        elif name == "get_post":
            post = state.posts.get(arguments["slug"])
            if post is None:
                self.tool_error(request_id, "Cannot coerce the result to a single JSON object")
            else:
                returned = dict(post)
                if state.corrupt_readback and arguments["slug"] != MCP.DEFAULT_REFERENCE:
                    returned["title"] = "Corrupted readback"
                self.tool_result(request_id, returned)
        elif name == "create_post":
            slug = arguments["slug"]
            if slug in state.posts:
                self.tool_error(request_id, f"slug 중복: {slug}")
                return
            job_field = arguments.get("job_field")
            if isinstance(job_field, str) and not job_field.startswith("{"):
                self.tool_error(request_id, f'malformed array literal: "{job_field}"')
                return
            state.create_count += 1
            post = {
                **arguments,
                "id": f"post-{state.create_count}",
                "published": arguments.get("published", False),
                "job_field": MCP.normalize_job_field(job_field),
            }
            state.posts[slug] = post
            if state.close_after_create:
                self.close_connection = True
                return
            if state.expire_after_mutation:
                state.auth_mode = "invalid"
            self.tool_result(request_id, {"id": post["id"], "slug": slug})
        elif name == "update_post":
            slug = arguments["slug"]
            if slug not in state.posts:
                self.tool_error(request_id, f"slug 없음: {slug}")
                return
            job_field = arguments.get("job_field")
            if isinstance(job_field, str) and not job_field.startswith("{"):
                self.tool_error(request_id, f'malformed array literal: "{job_field}"')
                return
            changed = dict(arguments)
            if "job_field" in changed:
                changed["job_field"] = MCP.normalize_job_field(changed["job_field"])
            state.posts[slug].update(changed)
            if state.expire_after_mutation:
                state.auth_mode = "invalid"
            self.tool_result(request_id, {"id": state.posts[slug]["id"], "slug": slug})
        else:
            self.tool_error(request_id, "unknown tool")


@contextmanager
def mock_server() -> Iterator[tuple[str, MockState]]:
    state = MockState()
    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    server.state = state  # type: ignore[attr-defined]
    worker = threading.Thread(target=server.serve_forever, daemon=True)
    worker.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}/api/mcp", state
    finally:
        server.shutdown()
        worker.join(timeout=5)
        server.server_close()


def request_value(**changes: Any) -> dict[str, Any]:
    value: dict[str, Any] = {
        "operation": "create",
        "slug": "vector-search-lessons",
        "title": "벡터 검색과 정확한 검색의 역할",
        "description": "두 검색 방식의 장단점을 근거로 비교",
        "content": "## 결론\n\n정확한 검색과 의미 검색의 역할을 나눠야 한다.\n",
        "published": False,
        "job_field": "web",
    }
    value.update(changes)
    return value


class DraftDevlogUnitTests(unittest.TestCase):
    def test_defaults_are_unpublished_and_hive_context_is_rejected(self) -> None:
        normalized = MCP.validate_request(
            {"operation": "create", "slug": "clean-post", "title": "안전한 글"},
            allow_publish=False,
        )
        self.assertFalse(normalized["published"])
        self.assertEqual(normalized["category"], MCP.DEFAULT_CATEGORY)
        self.assertEqual(normalized["job_field"], "web")
        for text in ("Aigent Hive 0.10.0", "KRG10-014", "feature/0.10.0"):
            with self.subTest(text=text), self.assertRaises(MCP.WorkflowError) as raised:
                MCP.validate_request(request_value(content=text), allow_publish=False)
            self.assertEqual(raised.exception.code, "content-policy-failed")

    def test_secrets_unsafe_mdx_and_unknown_components_are_rejected(self) -> None:
        hostile = (
            "Authorization: Bearer pf_agent_abcdefghijklmnopqrstuvwxyz123456\n"
            "<script>alert(1)</script><Widget onClick={x} />"
        )
        with self.assertRaises(MCP.WorkflowError) as raised:
            MCP.validate_request(request_value(content=hostile), allow_publish=False)
        codes = {finding["code"] for finding in raised.exception.data["findings"]}
        self.assertIn("bearer-header", codes)
        self.assertIn("script-tag", codes)
        self.assertIn("unknown-jsx:Widget", codes)

    def test_json_round_trip_preserves_backticks_ansi_and_korean(self) -> None:
        content = r'''## 예시

```cpp
const char* clear = "\033[2J";
const char* color = "\x1b[31m";
```

한글 본문과 `inline` 표기.
'''
        request = MCP.validate_request(request_value(content=content), allow_publish=False)
        encoded = MCP.canonical_bytes(request)
        self.assertEqual(json.loads(encoded)["content"], content)

    def test_source_code_is_data_but_raw_html_outside_fences_is_rejected(self) -> None:
        source_example = """## Example

```tsx
import Widget from './Widget';
const sample = '<script onClick={x}>';
```
"""
        self.assertEqual(
            MCP.validate_request(
                request_value(content=source_example), allow_publish=False
            )["content"],
            source_example,
        )
        with self.assertRaises(MCP.WorkflowError) as raised:
            MCP.validate_request(
                request_value(content="<table><tr><td>unsafe</td></tr></table>"),
                allow_publish=False,
            )
        self.assertIn(
            "raw-html", {item["code"] for item in raised.exception.data["findings"]}
        )

    def test_publish_and_public_update_require_explicit_authority(self) -> None:
        with self.assertRaises(MCP.WorkflowError) as raised:
            MCP.validate_request(request_value(published=True), allow_publish=False)
        self.assertEqual(raised.exception.code, "publish-not-authorized")
        self.assertTrue(
            MCP.validate_request(request_value(published=True), allow_publish=True)[
                "published"
            ]
        )

    def test_redaction_removes_token_shaped_values(self) -> None:
        value = MCP.redact(
            "Authorization: Bearer pf_agent_abcdefghijklmnopqrstuvwxyz123456"
        )
        self.assertNotIn("abcdefghijklmnopqrstuvwxyz", value)
        self.assertIn("[REDACTED]", value)

    def test_missing_token_stops_before_network(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=True), mock.patch.object(
            MCP.getpass, "getpass", side_effect=EOFError
        ), self.assertRaises(MCP.WorkflowError) as raised:
            MCP.read_token(MCP.PRODUCTION_ENDPOINT)
        self.assertEqual(raised.exception.code, "token-required")


class DraftDevlogMcpTests(unittest.TestCase):
    def setUp(self) -> None:
        self.environment = mock.patch.dict(
            os.environ, {"DRAFT_DEVLOG_TEST_BEARER": "fixture-token"}, clear=False
        )
        self.environment.start()

    def tearDown(self) -> None:
        self.environment.stop()

    def client(self, endpoint: str) -> Any:
        return MCP.McpClient(endpoint, MCP.read_token(endpoint), timeout=2)

    def test_preflight_order_and_reference_inspection(self) -> None:
        with mock_server() as (endpoint, state), tempfile.TemporaryDirectory() as temporary:
            result = MCP.run_inspect(
                argparse.Namespace(
                    endpoint=endpoint,
                    timeout=2,
                    state_dir=Path(temporary),
                    reference_slug=[MCP.DEFAULT_REFERENCE],
                )
            )
            self.assertEqual(result["status"], "success")
            self.assertEqual(
                state.calls[:3],
                [
                    ("tools/list", None),
                    ("tools/call", "get_schema"),
                    ("tools/call", "get_post"),
                ],
            )
            inspection = json.loads((Path(temporary) / "inspection.json").read_text("utf-8"))
            self.assertNotIn("fixture-token", json.dumps(inspection))

    def test_expired_and_rate_limited_tokens_are_typed(self) -> None:
        with mock_server() as (endpoint, state):
            state.auth_mode = "invalid"
            with self.assertRaises(MCP.WorkflowError) as expired:
                self.client(endpoint).tools_list()
            self.assertEqual(expired.exception.code, "token-expired-or-invalid")
            state.auth_mode = "rate"
            with self.assertRaises(MCP.WorkflowError) as limited:
                self.client(endpoint).tools_list()
            self.assertEqual(limited.exception.code, "mcp-rate-limited")
            self.assertEqual(limited.exception.data["retry_after_seconds"], "120")

    def test_schema_drift_fails_before_mutation(self) -> None:
        with mock_server() as (endpoint, state):
            state.omit_update_tool = True
            with self.assertRaises(MCP.WorkflowError) as raised:
                MCP.preflight(self.client(endpoint))
            self.assertEqual(raised.exception.code, "mcp-schema-drift")
            self.assertEqual(state.create_count, 0)

    def test_create_falls_back_for_job_field_and_is_idempotently_verified(self) -> None:
        with mock_server() as (endpoint, state), tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            request_path = root / "request.json"
            request_path.write_text(json.dumps(request_value(), ensure_ascii=False), "utf-8")
            arguments = argparse.Namespace(
                endpoint=endpoint,
                timeout=2,
                request=request_path,
                state_dir=root / "state",
                allow_publish=False,
            )
            first = MCP.run_apply(arguments)
            second = MCP.run_apply(arguments)
            self.assertEqual(first["status"], "success")
            self.assertEqual(second["status"], "success")
            self.assertFalse(first["published"])
            self.assertEqual(first["job_field"], ["web"])
            self.assertEqual(state.create_count, 1)
            for path in (root / "state").glob("*.json"):
                self.assertNotIn("fixture-token", path.read_text("utf-8"))

    def test_slug_conflict_never_updates_existing_post(self) -> None:
        with mock_server() as (endpoint, state), tempfile.TemporaryDirectory() as temporary:
            state.posts["vector-search-lessons"] = {
                "id": "existing",
                "slug": "vector-search-lessons",
                "title": "Existing",
                "content": "unchanged",
                "published": False,
                "job_field": ["web"],
            }
            root = Path(temporary)
            request_path = root / "request.json"
            request_path.write_text(json.dumps(request_value(), ensure_ascii=False), "utf-8")
            with self.assertRaises(MCP.WorkflowError) as raised:
                MCP.run_apply(
                    argparse.Namespace(
                        endpoint=endpoint,
                        timeout=2,
                        request=request_path,
                        state_dir=root / "state",
                        allow_publish=False,
                    )
                )
            self.assertEqual(raised.exception.code, "slug-conflict")
            self.assertEqual(state.posts["vector-search-lessons"]["title"], "Existing")

    def test_published_post_update_requires_authority(self) -> None:
        with mock_server() as (endpoint, state), tempfile.TemporaryDirectory() as temporary:
            state.posts["published-post"] = {
                "id": "published",
                "slug": "published-post",
                "title": "Published",
                "content": "public content",
                "published": True,
                "job_field": ["web"],
            }
            root = Path(temporary)
            request_path = root / "request.json"
            request_path.write_text(
                json.dumps(
                    {
                        "operation": "update",
                        "slug": "published-post",
                        "title": "Changed",
                    },
                    ensure_ascii=False,
                ),
                "utf-8",
            )
            arguments = argparse.Namespace(
                endpoint=endpoint,
                timeout=2,
                request=request_path,
                state_dir=root / "state",
                allow_publish=False,
            )
            with self.assertRaises(MCP.WorkflowError) as raised:
                MCP.run_apply(arguments)
            self.assertEqual(raised.exception.code, "published-update-not-authorized")
            self.assertEqual(state.posts["published-post"]["title"], "Published")

            arguments.allow_publish = True
            result = MCP.run_apply(arguments)
            self.assertEqual(result["status"], "success")
            self.assertEqual(state.posts["published-post"]["title"], "Changed")

    def test_lost_create_response_reconciles_without_replay(self) -> None:
        with mock_server() as (endpoint, state), tempfile.TemporaryDirectory() as temporary:
            state.close_after_create = True
            root = Path(temporary)
            request_path = root / "request.json"
            request_path.write_text(json.dumps(request_value(), ensure_ascii=False), "utf-8")
            result = MCP.run_apply(
                argparse.Namespace(
                    endpoint=endpoint,
                    timeout=2,
                    request=request_path,
                    state_dir=root / "state",
                    allow_publish=False,
                )
            )
            self.assertEqual(result["status"], "success")
            self.assertEqual(state.create_count, 1)
            mutation = json.loads((root / "state/mutation.json").read_text("utf-8"))
            self.assertTrue(mutation["result"]["reconciled"])

    def test_token_expiry_after_mutation_preserves_safe_resume_state(self) -> None:
        with mock_server() as (endpoint, state), tempfile.TemporaryDirectory() as temporary:
            state.expire_after_mutation = True
            root = Path(temporary)
            request_path = root / "request.json"
            request_path.write_text(json.dumps(request_value(), ensure_ascii=False), "utf-8")
            with self.assertRaises(MCP.WorkflowError) as raised:
                MCP.run_apply(
                    argparse.Namespace(
                        endpoint=endpoint,
                        timeout=2,
                        request=request_path,
                        state_dir=root / "state",
                        allow_publish=False,
                    )
                )
            self.assertEqual(raised.exception.code, "mutation-succeeded-unverified")
            mutation = (root / "state/mutation.json").read_text("utf-8")
            self.assertNotIn("fixture-token", mutation)
            self.assertEqual(state.create_count, 1)

    def test_readback_mismatch_is_not_completion(self) -> None:
        with mock_server() as (endpoint, state), tempfile.TemporaryDirectory() as temporary:
            state.corrupt_readback = True
            root = Path(temporary)
            request_path = root / "request.json"
            request_path.write_text(json.dumps(request_value(), ensure_ascii=False), "utf-8")
            with self.assertRaises(MCP.WorkflowError) as raised:
                MCP.run_apply(
                    argparse.Namespace(
                        endpoint=endpoint,
                        timeout=2,
                        request=request_path,
                        state_dir=root / "state",
                        allow_publish=False,
                    )
                )
            self.assertEqual(raised.exception.code, "readback-mismatch")
            self.assertIn("title", raised.exception.data["mismatches"])

    def test_no_product_projection_exists(self) -> None:
        self.assertFalse((ROOT / "harness/skills/draft-devlog").exists())
        self.assertFalse((ROOT / "harness/template/.agents/skills/draft-devlog").exists())
        self.assertFalse((ROOT / "harness/plugins/aigent-hive/skills/draft-devlog").exists())


if __name__ == "__main__":
    unittest.main()
