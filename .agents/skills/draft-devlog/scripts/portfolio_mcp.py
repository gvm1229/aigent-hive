#!/usr/bin/env python3
"""Safe PortareFolium MCP transport and draft verifier for draft-devlog."""

from __future__ import annotations

import argparse
import getpass
import hashlib
import http.client
import json
import os
import re
import socket
import sys
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any


PRODUCTION_ENDPOINT = "https://gvm1229-portfolio.vercel.app/api/mcp"
DEFAULT_REFERENCE = "supabase-storage-cloudflare-r2-image-cdn-migration"
DEFAULT_CATEGORY = "Harness 개발 일지"
DEFAULT_JOB_FIELD = "web"
DEFAULT_TAGS = ["AI", "Agent", "Harness"]
SAFE_JOB_FIELD = re.compile(r"^[a-z0-9][a-z0-9_-]{0,63}$")
SAFE_SLUG = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
CHECKLIST_ID = re.compile(r"\b(?:KRG|VEC|VQR|REL|KOR|SDB)\d{2,}(?:-\d+)?\b", re.I)
COMMIT_ID = re.compile(r"(?<![0-9a-f])[0-9a-f]{7,40}(?![0-9a-f])", re.I)
SECRET_PATTERNS = (
    ("bearer-header", re.compile(r"\bAuthorization\s*:\s*Bearer\b", re.I)),
    ("bearer-value", re.compile(r"\bBearer\s+[A-Za-z0-9._~+/-]{8,}", re.I)),
    ("portfolio-token", re.compile(r"\bpf_agent_[A-Za-z0-9_-]{16,}\b", re.I)),
    ("provider-key", re.compile(r"\b(?:sk|ghp|github_pat)_[A-Za-z0-9_-]{16,}\b", re.I)),
)
HIVE_PATTERNS = (
    ("aigent-hive", re.compile(r"\baigent[- ]?hive\b", re.I)),
    ("hive-name", re.compile(r"\bhive\b|에이전트\s*하이브|아이전트\s*하이브", re.I)),
    ("hive-prerelease", re.compile(r"\b0\.10\.0-test(?:\.\d+)?\b", re.I)),
    ("internal-checklist", CHECKLIST_ID),
    ("internal-branch", re.compile(r"\b(?:feature|fix|release|docs|test|refactor|build|chore)/[A-Za-z0-9._/-]+", re.I)),
    ("internal-path", re.compile(r"(?:[A-Za-z]:\\Users\\|/Users/|/home/|docs/plans/|tests/work/|\.agents/)", re.I)),
    ("commit-id", COMMIT_ID),
)
UNSAFE_MDX = (
    ("module-statement", re.compile(r"^\s*(?:import|export)\s", re.I | re.M)),
    ("script-tag", re.compile(r"<\s*script\b", re.I)),
    ("javascript-url", re.compile(r"javascript\s*:", re.I)),
    ("event-handler", re.compile(r"\bon[A-Za-z]+\s*=", re.I)),
    ("raw-html", re.compile(r"<\/?[a-z][A-Za-z0-9-]*\b")),
)
CAPITALIZED_COMPONENT = re.compile(r"<\/?([A-Z][A-Za-z0-9]*)\b")
FENCED_CODE = re.compile(r"```[A-Za-z0-9_+-]*\r?\n[\s\S]*?```")
INLINE_CODE = re.compile(r"`[^`\r\n]+`")
POST_FIELDS = {
    "slug",
    "title",
    "description",
    "content",
    "pub_date",
    "category",
    "tags",
    "job_field",
    "thumbnail",
    "published",
    "meta_title",
    "meta_description",
    "og_image",
}
TEXT_FIELDS = ("title", "description", "content", "meta_title", "meta_description")
REQUIRED_TOOLS = {"get_schema", "get_post", "create_post", "update_post"}


@dataclass
class WorkflowError(Exception):
    code: str
    message: str
    exit_code: int = 5
    data: dict[str, Any] | None = None

    def __str__(self) -> str:
        return self.message


def redact(value: str) -> str:
    redacted = re.sub(r"pf_agent_[A-Za-z0-9_-]+", "[REDACTED]", value)
    return re.sub(r"Bearer\s+[^\s\"']+", "Bearer [REDACTED]", redacted, flags=re.I)


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")


def write_json_atomic(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    handle = tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", newline="\n", dir=path.parent, delete=False
    )
    temporary = Path(handle.name)
    try:
        with handle:
            handle.write(payload)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def load_object(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise WorkflowError("invalid-json", f"cannot read {label}: {error}", 2) from error
    if not isinstance(value, dict):
        raise WorkflowError("invalid-json", f"{label} must be one JSON object", 2)
    return value


def endpoint_is_loopback(endpoint: str) -> bool:
    host = urllib.parse.urlparse(endpoint).hostname
    return host in {"127.0.0.1", "localhost", "::1"}


def validate_endpoint(endpoint: str) -> None:
    if endpoint == PRODUCTION_ENDPOINT:
        return
    if endpoint_is_loopback(endpoint) and os.environ.get("DRAFT_DEVLOG_TEST_BEARER"):
        return
    raise WorkflowError("unsafe-endpoint", "only the production endpoint or a loopback test endpoint is allowed", 2)


def read_token(endpoint: str) -> str:
    test_token = os.environ.get("DRAFT_DEVLOG_TEST_BEARER")
    if test_token is not None:
        if not endpoint_is_loopback(endpoint):
            raise WorkflowError("unsafe-test-token", "test token environment use requires a loopback endpoint", 2)
        token = test_token
    else:
        try:
            token = getpass.getpass("Temporary PortareFolium Bearer token: ")
        except (EOFError, KeyboardInterrupt) as error:
            raise WorkflowError("token-required", "a user-supplied temporary token is required", 3) from error
    token = token.strip()
    if not token or any(character in token for character in "\r\n\0"):
        raise WorkflowError("token-required", "a user-supplied temporary token is required", 3)
    return token


class McpClient:
    def __init__(self, endpoint: str, token: str, timeout: float = 30.0) -> None:
        validate_endpoint(endpoint)
        self.endpoint = endpoint
        self._token = token
        self.timeout = timeout
        self._request_id = 0

    def _next_id(self) -> int:
        self._request_id += 1
        return self._request_id

    def _request(self, method: str, params: dict[str, Any], *, safe_retry: bool) -> dict[str, Any]:
        request_id = self._next_id()
        payload = canonical_bytes(
            {"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}
        )
        attempts = 2 if safe_retry else 1
        for attempt in range(attempts):
            request = urllib.request.Request(
                self.endpoint,
                data=payload,
                method="POST",
                headers={
                    "Authorization": f"Bearer {self._token}",
                    "Accept": "application/json, text/event-stream",
                    "Content-Type": "application/json",
                },
            )
            try:
                with urllib.request.urlopen(request, timeout=self.timeout) as response:
                    status = response.status
                    headers = response.headers
                    body = response.read()
            except urllib.error.HTTPError as error:
                status = error.code
                headers = error.headers
                body = error.read()
            except (
                TimeoutError,
                socket.timeout,
                urllib.error.URLError,
                http.client.RemoteDisconnected,
                ConnectionError,
            ) as error:
                if safe_retry and attempt + 1 < attempts:
                    continue
                raise WorkflowError(
                    "transport-uncertain" if not safe_retry else "transport-failed",
                    f"MCP transport failed: {redact(str(error))}",
                    5,
                    {"mutation_uncertain": not safe_retry},
                ) from error

            try:
                value = json.loads(body.decode("utf-8"))
            except (UnicodeError, json.JSONDecodeError) as error:
                raise WorkflowError("invalid-mcp-response", "MCP returned invalid JSON", 5) from error
            if status == 401 or value.get("error", {}).get("code") == -32001:
                raise WorkflowError(
                    "token-expired-or-invalid",
                    "the temporary token is expired, revoked, or invalid; ask the user for a new token",
                    3,
                )
            if status == 429 or value.get("error", {}).get("code") == -32002:
                retry_after = headers.get("Retry-After") if headers else None
                raise WorkflowError(
                    "mcp-rate-limited",
                    "MCP rejected too many invalid attempts; do not retry automatically",
                    4,
                    {"retry_after_seconds": retry_after},
                )
            if status < 200 or status >= 300:
                raise WorkflowError("mcp-http-error", f"MCP returned HTTP {status}", 5)
            if value.get("jsonrpc") != "2.0" or value.get("id") != request_id:
                raise WorkflowError("invalid-mcp-response", "MCP response identity mismatch", 5)
            if "error" in value:
                error_value = value["error"]
                code = error_value.get("code")
                message = redact(str(error_value.get("message", "MCP tool failed")))
                raise WorkflowError(f"mcp-tool-{code}", message, 5, {"mcp_code": code})
            result = value.get("result")
            if not isinstance(result, dict):
                raise WorkflowError("invalid-mcp-response", "MCP result must be an object", 5)
            return result
        raise AssertionError("request loop exhausted")

    def tools_list(self) -> list[dict[str, Any]]:
        result = self._request("tools/list", {}, safe_retry=True)
        tools = result.get("tools")
        if not isinstance(tools, list):
            raise WorkflowError("invalid-tools-list", "tools/list did not return a tool array", 5)
        return [tool for tool in tools if isinstance(tool, dict)]

    def tool_call(self, name: str, arguments: dict[str, Any], *, mutation: bool = False) -> Any:
        result = self._request(
            "tools/call",
            {"name": name, "arguments": arguments},
            safe_retry=not mutation,
        )
        content = result.get("content")
        if not isinstance(content, list) or not content or not isinstance(content[0], dict):
            raise WorkflowError("invalid-tool-result", f"{name} returned no text content", 5)
        text = content[0].get("text")
        if not isinstance(text, str):
            raise WorkflowError("invalid-tool-result", f"{name} returned non-text content", 5)
        try:
            return json.loads(text)
        except json.JSONDecodeError as error:
            raise WorkflowError("invalid-tool-result", f"{name} text is not JSON", 5) from error


def tool_map(tools: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    return {str(tool.get("name")): tool for tool in tools if tool.get("name")}


def validate_tools(tools: list[dict[str, Any]]) -> None:
    missing = sorted(REQUIRED_TOOLS - set(tool_map(tools)))
    if missing:
        raise WorkflowError("mcp-schema-drift", f"required MCP tools are missing: {missing}", 5)


def normalize_request(raw: dict[str, Any]) -> dict[str, Any]:
    request = dict(raw)
    operation = request.get("operation", "create")
    if operation not in {"create", "update"}:
        raise WorkflowError("invalid-operation", "operation must be create or update", 2)
    request["operation"] = operation
    if operation == "create":
        request.setdefault("published", False)
        request.setdefault("category", DEFAULT_CATEGORY)
        request.setdefault("job_field", DEFAULT_JOB_FIELD)
        request.setdefault("tags", list(DEFAULT_TAGS))
    request.setdefault("allowed_components", [])
    return request


def scan_forbidden(request: dict[str, Any]) -> list[dict[str, str]]:
    findings: list[dict[str, str]] = []
    allowed_components = request.get("allowed_components", [])
    allowed = {str(value) for value in allowed_components if isinstance(value, str)}
    for field in TEXT_FIELDS:
        value = request.get(field)
        if not isinstance(value, str):
            continue
        for name, pattern in (*SECRET_PATTERNS, *HIVE_PATTERNS):
            if pattern.search(value):
                findings.append({"field": field, "code": name})
        mdx_surface = INLINE_CODE.sub("", FENCED_CODE.sub("", value))
        for name, pattern in UNSAFE_MDX:
            if pattern.search(mdx_surface):
                findings.append({"field": field, "code": name})
        for component in CAPITALIZED_COMPONENT.findall(mdx_surface):
            if component not in allowed:
                findings.append({"field": field, "code": f"unknown-jsx:{component}"})
        if field == "content" and value.count("```") % 2:
            findings.append({"field": field, "code": "unbalanced-code-fence"})
    return findings


def validate_request(raw: dict[str, Any], *, allow_publish: bool) -> dict[str, Any]:
    forbidden_keys = {"token", "bearer", "authorization", "headers"} & {
        str(key).casefold() for key in raw
    }
    if forbidden_keys:
        raise WorkflowError("secret-field-forbidden", "request JSON cannot contain authentication fields", 2)
    request = normalize_request(raw)
    slug = request.get("slug")
    if not isinstance(slug, str) or not SAFE_SLUG.fullmatch(slug):
        raise WorkflowError("invalid-slug", "slug must be lowercase ASCII words separated by hyphens", 2)
    if request["operation"] == "create":
        if not isinstance(request.get("title"), str) or not request["title"].strip():
            raise WorkflowError("title-required", "create requires a title", 2)
    elif not any(key in POST_FIELDS - {"slug"} for key in request):
        raise WorkflowError("empty-update", "update requires at least one changed post field", 2)
    if request.get("published") is True and not allow_publish:
        raise WorkflowError("publish-not-authorized", "published=true requires current-request publication authority", 3)
    tags = request.get("tags")
    if tags is not None and (
        not isinstance(tags, list) or any(not isinstance(tag, str) for tag in tags)
    ):
        raise WorkflowError("invalid-tags", "tags must be a string array", 2)
    job_field = request.get("job_field")
    if job_field is not None and (
        not isinstance(job_field, str) or not SAFE_JOB_FIELD.fullmatch(job_field)
    ):
        raise WorkflowError("invalid-job-field", "job_field must be one safe registered id", 2)
    components = request.get("allowed_components")
    if not isinstance(components, list) or any(
        not isinstance(component, str) or not re.fullmatch(r"[A-Z][A-Za-z0-9]*", component)
        for component in components
    ):
        raise WorkflowError("invalid-components", "allowed_components must contain safe component names", 2)
    findings = scan_forbidden(request)
    if findings:
        raise WorkflowError(
            "content-policy-failed",
            "draft contains secrets, Hive-internal context, unsafe MDX, or malformed Markdown",
            5,
            {"findings": findings},
        )
    return request


def post_arguments(request: dict[str, Any]) -> dict[str, Any]:
    return {key: request[key] for key in POST_FIELDS if key in request}


def schema_components(schema: Any) -> set[str]:
    if not isinstance(schema, dict):
        return set()
    components = schema.get("content_components")
    return set(components) if isinstance(components, dict) else set()


def normalize_job_field(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, list):
        return [str(item) for item in value]
    if isinstance(value, str):
        stripped = value.strip("{}")
        return [stripped] if stripped else []
    return [str(value)]


def verify_post(post: Any, request: dict[str, Any]) -> dict[str, Any]:
    if not isinstance(post, dict):
        raise WorkflowError("readback-invalid", "get_post returned a non-object", 5)
    mismatches: list[str] = []
    for key, expected in post_arguments(request).items():
        if key == "job_field":
            if normalize_job_field(post.get(key)) != normalize_job_field(expected):
                mismatches.append(key)
        elif post.get(key) != expected:
            mismatches.append(key)
    content = post.get("content", "")
    if not isinstance(content, str):
        mismatches.append("content")
        content = ""
    readback_request = dict(request)
    readback_request["title"] = str(post.get("title", ""))
    readback_request["description"] = str(post.get("description") or "")
    readback_request["content"] = content
    readback_request["meta_title"] = str(post.get("meta_title") or "")
    readback_request["meta_description"] = str(post.get("meta_description") or "")
    policy_findings = scan_forbidden(readback_request)
    if policy_findings:
        mismatches.append("content-policy")
    return {
        "post_id": post.get("id"),
        "slug": post.get("slug"),
        "published": post.get("published"),
        "category": post.get("category"),
        "tags": post.get("tags"),
        "job_field": post.get("job_field"),
        "content_chars": len(content),
        "content_digest": digest_bytes(content.encode("utf-8")),
        "mismatches": sorted(set(mismatches)),
        "policy_findings": policy_findings,
    }


def preflight(client: McpClient) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    tools = client.tools_list()
    validate_tools(tools)
    schema = client.tool_call("get_schema", {})
    if not isinstance(schema, dict):
        raise WorkflowError("mcp-schema-drift", "get_schema returned a non-object", 5)
    return tools, schema


def run_inspect(args: argparse.Namespace) -> dict[str, Any]:
    token = read_token(args.endpoint)
    client = McpClient(args.endpoint, token, args.timeout)
    tools, schema = preflight(client)
    references: dict[str, Any] = {}
    for slug in args.reference_slug:
        references[slug] = client.tool_call("get_post", {"slug": slug})
    inspection = {
        "schema_version": 1,
        "endpoint": args.endpoint,
        "tools": tools,
        "schema": schema,
        "references": references,
    }
    path = args.state_dir / "inspection.json"
    write_json_atomic(path, inspection)
    return {
        "action": "inspect",
        "status": "success",
        "inspection_path": str(path),
        "tool_names": sorted(tool_map(tools)),
        "reference_slugs": sorted(references),
        "schema_digest": digest_bytes(canonical_bytes(schema)),
    }


def run_validate(args: argparse.Namespace) -> dict[str, Any]:
    request = validate_request(load_object(args.request, "request"), allow_publish=args.allow_publish)
    return {
        "action": "validate",
        "status": "success",
        "operation": request["operation"],
        "slug": request["slug"],
        "published": request.get("published"),
        "request_digest": digest_bytes(canonical_bytes(request)),
        "content_digest": digest_bytes(str(request.get("content", "")).encode("utf-8")),
    }


def compatibility_arguments(arguments: dict[str, Any]) -> dict[str, Any]:
    job_field = arguments.get("job_field")
    if isinstance(job_field, str) and SAFE_JOB_FIELD.fullmatch(job_field):
        return {**arguments, "job_field": "{" + job_field + "}"}
    return arguments


def call_mutation_with_job_field_compatibility(
    client: McpClient, tool: str, arguments: dict[str, Any]
) -> Any:
    try:
        return client.tool_call(tool, arguments, mutation=True)
    except WorkflowError as error:
        if "malformed array literal" not in error.message or not isinstance(
            arguments.get("job_field"), str
        ):
            raise
        return client.tool_call(tool, compatibility_arguments(arguments), mutation=True)


def write_mutation_state(
    state_dir: Path, operation: str, request: dict[str, Any], result: Any
) -> Path:
    path = state_dir / "mutation.json"
    state = {
        "schema_version": 1,
        "operation": operation,
        "slug": request["slug"],
        "request_digest": digest_bytes(canonical_bytes(request)),
        "result": result,
    }
    write_json_atomic(path, state)
    return path


def run_apply(args: argparse.Namespace) -> dict[str, Any]:
    request = validate_request(load_object(args.request, "request"), allow_publish=args.allow_publish)
    token = read_token(args.endpoint)
    client = McpClient(args.endpoint, token, args.timeout)
    tools, schema = preflight(client)
    allowed = set(request.get("allowed_components", []))
    unsupported = sorted(allowed - schema_components(schema))
    if unsupported:
        raise WorkflowError("mcp-schema-drift", f"requested MDX components are unavailable: {unsupported}", 5)

    args.state_dir.mkdir(parents=True, exist_ok=True)
    request_digest = digest_bytes(canonical_bytes(request))
    mutation_path = args.state_dir / "mutation.json"
    mutation_state = load_object(mutation_path, "mutation state") if mutation_path.exists() else None
    if mutation_state and mutation_state.get("request_digest") != request_digest:
        raise WorkflowError("state-conflict", "mutation state belongs to another request", 5)

    operation = request["operation"]
    if operation == "update":
        current = client.tool_call("get_post", {"slug": request["slug"]})
        if not isinstance(current, dict):
            raise WorkflowError("readback-invalid", "existing post is not an object", 5)
        if current.get("published") is True and not args.allow_publish:
            raise WorkflowError(
                "published-update-not-authorized",
                "editing an already published post requires current-request authority",
                3,
            )
        before_path = args.state_dir / "before.json"
        if not before_path.exists():
            write_json_atomic(before_path, current)

    mutation_result: Any = mutation_state.get("result") if mutation_state else None
    if mutation_state is None:
        tool = "create_post" if operation == "create" else "update_post"
        arguments = post_arguments(request)
        try:
            mutation_result = call_mutation_with_job_field_compatibility(client, tool, arguments)
        except WorkflowError as error:
            if error.code == "transport-uncertain":
                try:
                    candidate = client.tool_call("get_post", {"slug": request["slug"]})
                except WorkflowError as reconcile_error:
                    raise WorkflowError(
                        "mutation-uncertain",
                        "mutation response was lost and exact slug verification did not resolve it",
                        5,
                        {"reconcile_code": reconcile_error.code},
                    ) from error
                verification = verify_post(candidate, request)
                if verification["mismatches"]:
                    raise WorkflowError(
                        "mutation-uncertain",
                        "slug exists after a lost mutation response but does not match the request",
                        5,
                        verification,
                    ) from error
                mutation_result = {"slug": request["slug"], "reconciled": True}
            elif operation == "create" and "slug 중복" in error.message:
                raise WorkflowError(
                    "slug-conflict",
                    "slug already exists; ask the user whether to update it or choose another slug",
                    3,
                ) from error
            else:
                raise
        write_mutation_state(args.state_dir, operation, request, mutation_result)

    try:
        post = client.tool_call("get_post", {"slug": request["slug"]})
    except WorkflowError as error:
        if error.code == "token-expired-or-invalid":
            raise WorkflowError(
                "mutation-succeeded-unverified",
                "mutation receipt exists but read-back needs a new user-supplied token",
                3,
                {"mutation_path": str(mutation_path)},
            ) from error
        raise
    verification = verify_post(post, request)
    receipt = {
        "schema_version": 1,
        "action": operation,
        "status": "success" if not verification["mismatches"] else "verification-failed",
        "endpoint": args.endpoint,
        "request_digest": request_digest,
        "tool_names": sorted(tool_map(tools)),
        "schema_digest": digest_bytes(canonical_bytes(schema)),
        **verification,
    }
    receipt_path = args.state_dir / "receipt.json"
    write_json_atomic(receipt_path, receipt)
    receipt["receipt_path"] = str(receipt_path)
    if verification["mismatches"]:
        raise WorkflowError("readback-mismatch", "saved post differs from the exact request", 5, receipt)
    return receipt


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--endpoint", default=PRODUCTION_ENDPOINT)
    root.add_argument("--timeout", type=float, default=30.0)
    commands = root.add_subparsers(dest="command", required=True)

    inspect = commands.add_parser("inspect")
    inspect.add_argument("--state-dir", type=Path, required=True)
    inspect.add_argument(
        "--reference-slug", action="append", default=[DEFAULT_REFERENCE]
    )
    inspect.set_defaults(run=run_inspect)

    validate = commands.add_parser("validate")
    validate.add_argument("--request", type=Path, required=True)
    validate.add_argument("--allow-publish", action="store_true")
    validate.set_defaults(run=run_validate)

    apply = commands.add_parser("apply")
    apply.add_argument("--request", type=Path, required=True)
    apply.add_argument("--state-dir", type=Path, required=True)
    apply.add_argument("--allow-publish", action="store_true")
    apply.set_defaults(run=run_apply)
    return root


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        result = args.run(args)
    except WorkflowError as error:
        payload = {
            "schema_version": 1,
            "status": "error",
            "code": error.code,
            "message": redact(error.message),
            "data": error.data,
        }
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True))
        return error.exit_code
    print(json.dumps({"schema_version": 1, **result}, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
