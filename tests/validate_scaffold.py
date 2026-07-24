#!/usr/bin/env python3
"""Validate schemas and a rendered Phase 1 consumer-harness fixture."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
import tomllib
from pathlib import Path

import yaml
from jsonschema import Draft202012Validator, FormatChecker, ValidationError


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
SCHEMA_DIRECTORY = REPOSITORY_ROOT / "schemas"
APACHE_LICENSE_PATH = REPOSITORY_ROOT / "LICENSES/Apache-2.0.txt"
CONSENT_FIELDS = (
    "consent_version",
    "name",
    "source",
    "revision",
    "content_digest",
    "requested_capabilities",
    "approved_capabilities",
    "approved_at",
)
HOOK_CONSENT_FIELDS = (
    "consent_version",
    "capability",
    "event",
    "path",
    "command",
    "content_digest",
    "approved_at",
)


def read_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as stream:
        return tomllib.load(stream)


def read_yaml(path: Path) -> object:
    with path.open("r", encoding="utf-8") as stream:
        return yaml.safe_load(stream)


def validate_schema_documents() -> None:
    for path in sorted(SCHEMA_DIRECTORY.glob("*.schema.json")):
        schema = json.loads(path.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(schema)


def validate_instance(schema_name: str, instance: object) -> None:
    schema_path = SCHEMA_DIRECTORY / schema_name
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    Draft202012Validator(
        schema,
        format_checker=FormatChecker(),
    ).validate(instance)


def expect_invalid(schema_name: str, instance: object) -> None:
    try:
        validate_instance(schema_name, instance)
    except ValidationError:
        return
    raise AssertionError(f"invalid fixture passed {schema_name}")


def canonical_digest(value: object) -> str:
    """Return the RFC 8785-compatible digest for this integer/string fixture subset."""
    canonical_bytes = json.dumps(
        value,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return f"sha256:{hashlib.sha256(canonical_bytes).hexdigest()}"


def with_capability_digest(value: dict[str, object]) -> dict[str, object]:
    normalized = copy.deepcopy(value)
    normalized.pop("evidence_digest", None)
    normalized["evidence_digest"] = canonical_digest(normalized)
    return normalized


def validate_capability_resolution(value: object) -> None:
    assert isinstance(value, dict)
    validate_instance("capability-matrix.schema.json", value)
    payload = copy.deepcopy(value)
    actual_digest = payload.pop("evidence_digest")
    expected_digest = canonical_digest(payload)
    if actual_digest != expected_digest:
        raise AssertionError(
            f"capability resolution digest mismatch: {actual_digest} != {expected_digest}"
        )


def validate_contract_examples() -> None:
    digest = f"sha256:{'0' * 64}"
    action_result = {
        "schema_version": 1,
        "action": "SetupHarness",
        "status": "success",
        "exit_code": 0,
        "code": "hive.setup.success",
        "message": "setup completed",
        "changed_paths": [],
        "evidence": [],
    }
    validate_instance("action-result.schema.json", action_result)
    expect_invalid(
        "action-result.schema.json",
        {**action_result, "exit_code": 3},
    )

    role_profile = {
        "schema_version": 1,
        "role_id": "reviewer",
        "display_name": "Reviewer",
        "responsibilities": ["verify acceptance criteria"],
        "non_responsibilities": ["implement the reviewed artifact"],
        "context_paths": ["docs/"],
        "allowed_capabilities": ["filesystem-read"],
        "write_scope": [".hive/runs/"],
        "verification_duties": ["attach reproducible evidence"],
        "current_assignment": None,
        "handoff_path": None,
    }
    validate_instance("role-profile.schema.json", role_profile)

    run_status = {
        "schema_version": 1,
        "run_id": "phase-0",
        "revision": 1,
        "state": "resume-ready",
        "required_criteria": ["C1"],
        "passed_criteria": [],
        "failed_criteria": [],
        "active_roles": ["reviewer"],
        "next_action": "verify C1",
        "latest_evidence": [],
        "blocker": None,
        "updated_at": "2026-07-23T00:00:00Z",
    }
    validate_instance("run-status.schema.json", run_status)

    judge_package = {
        "schema_version": 1,
        "subject_id": "phase-0",
        "risk_tier": "elevated",
        "goal": "verify the scaffold",
        "acceptance_criteria": ["C1"],
        "artifact_refs": ["docs/plans/PLAN.md"],
        "evidence_refs": [],
        "known_constraints": [],
        "package_digest": digest,
    }
    validate_instance("judge-package.schema.json", judge_package)

    judge_verdict = {
        "schema_version": 1,
        "subject_id": "phase-0",
        "judge_id": "judge-1",
        "package_digest": digest,
        "verdict": "PASS",
        "findings": [],
        "missing_evidence": [],
        "created_at": "2026-07-23T00:00:00Z",
    }
    validate_instance("judge-verdict.schema.json", judge_verdict)
    expect_invalid(
        "judge-verdict.schema.json",
        {**judge_verdict, "missing_evidence": ["test log"]},
    )

    capability_matrix = with_capability_digest({
        "schema_version": 1,
        "host": "codex",
        "host_version": "fixture",
        "surface": "cli",
        "detection": "available",
        "external_runtime": "omx",
        "resolved_owner": "omx",
        "capabilities": {
            "instructions": "supported",
            "simple-question-isolation": "unverified",
            "subagents": "supported",
            "persistent-role-binding": "best-effort",
            "continuous-loop": "unverified",
            "usage-sensor": "unsupported",
            "independent-judge": "supported",
        },
        "evidence": [
            {
                "source": "host-catalog",
                "locator": "active-host-capability-metadata",
                "outcome": "compatible",
                "digest": digest,
            }
        ],
    })
    validate_capability_resolution(capability_matrix)

    absent = with_capability_digest({
        **capability_matrix,
        "detection": "absent",
        "external_runtime": None,
        "resolved_owner": "host-native",
        "evidence": [
            {
                "source": "host-catalog",
                "locator": "active-host-capability-metadata",
                "outcome": "absent",
                "digest": digest,
            },
            {
                "source": "public-executable",
                "locator": "omx --version",
                "outcome": "absent",
                "digest": digest,
            },
        ],
    })
    validate_capability_resolution(absent)

    missing_public_absence = with_capability_digest({
        **absent,
        "evidence": [absent["evidence"][0]],
    })
    expect_invalid("capability-matrix.schema.json", missing_public_absence)

    for detection, external_runtime, evidence_outcome in (
        ("absent", None, "absent"),
        ("unknown", None, "unavailable"),
        ("incompatible", "omx", "incompatible"),
    ):
        contradictory_evidence = [
            {
                "source": "host-catalog",
                "locator": "active-host-capability-metadata",
                "outcome": evidence_outcome,
                "digest": digest,
            }
        ]
        if detection == "absent":
            contradictory_evidence.append({
                "source": "public-executable",
                "locator": "omx --version",
                "outcome": "absent",
                "digest": digest,
            })
        contradictory_evidence.append({
            "source": "public-executable",
            "locator": "compatible runtime evidence",
            "outcome": "compatible",
            "digest": digest,
        })
        contradictory = with_capability_digest({
            **capability_matrix,
            "detection": detection,
            "external_runtime": external_runtime,
            "resolved_owner": "host-native",
            "evidence": contradictory_evidence,
        })
        expect_invalid("capability-matrix.schema.json", contradictory)

    available_without_compatible = with_capability_digest({
        **capability_matrix,
        "evidence": [
            {
                "source": "host-catalog",
                "locator": "active-host-capability-metadata",
                "outcome": "unavailable",
                "digest": digest,
            }
        ],
    })
    expect_invalid("capability-matrix.schema.json", available_without_compatible)

    raw_locator = f"raw:.hive/knowledge/Raw/source/{'0' * 64}.md#{digest}"
    knowledge_page = {
        "schema_version": 1,
        "id": "phase-two",
        "kind": "concept",
        "summary": "Canonical Markdown knowledge",
        "tags": ["knowledge"],
        "aliases": ["wiki"],
        "sources": [raw_locator],
        "links": [],
        "contradictions": [],
        "status": "active",
        "created_at": "2026-07-24T00:00:00Z",
        "updated_at": "2026-07-24T00:00:00Z",
    }
    validate_instance("knowledge-page.schema.json", knowledge_page)
    expect_invalid(
        "knowledge-page.schema.json",
        {**knowledge_page, "status": "deprecated"},
    )

    suppression = {
        "schema_version": 1,
        "entries": [
            {
                "fingerprint": digest,
                "source_locator": "wiki:phase-two",
                "reason": "obsolete",
                "replacement": None,
                "timestamp": "2026-07-24T00:00:00Z",
            }
        ],
    }
    validate_instance("knowledge-suppression.schema.json", suppression)
    expect_invalid(
        "knowledge-suppression.schema.json",
        {
            **suppression,
            "entries": [{**suppression["entries"][0], "body": "deleted prose"}],
        },
    )


def validate_license_boundary() -> None:
    if (REPOSITORY_ROOT / "LICENSE").read_bytes() != APACHE_LICENSE_PATH.read_bytes():
        raise AssertionError("root Apache license diverged from canonical text")
    if (
        REPOSITORY_ROOT / "harness/LICENSE"
    ).read_bytes() != APACHE_LICENSE_PATH.read_bytes():
        raise AssertionError("harness Apache license diverged from canonical text")

    reuse = read_toml(REPOSITORY_ROOT / "REUSE.toml")
    annotations = reuse["annotations"]
    assert isinstance(annotations, list)
    assert annotations == [
        {
            "path": "**",
            "precedence": "override",
            "SPDX-FileCopyrightText": "2026 Hojin (Tom) Jeong",
            "SPDX-License-Identifier": "Apache-2.0",
        },
    ]

    workspace = read_toml(REPOSITORY_ROOT / "Cargo.toml")
    assert workspace["workspace"]["package"]["license"] == "Apache-2.0"
    for crate_name in ("hive-cli", "hive-core", "hive-render", "hive-wiki"):
        crate = read_toml(REPOSITORY_ROOT / f"crates/{crate_name}/Cargo.toml")
        assert crate["package"]["license"] == {"workspace": True}

    harness_manifest = read_toml(REPOSITORY_ROOT / "harness/manifest.toml")
    assert harness_manifest["license"] == "Apache-2.0"
    licensed_paths = {
        path["pattern"]
        for path in harness_manifest["paths"]
        if path.get("ownership") == "hive-managed-license"
    }
    assert licensed_paths == {
        ".hive/LICENSE-AIGENT-HIVE.txt",
        ".hive/README.md",
    }

    apache_license = APACHE_LICENSE_PATH.read_bytes()
    template_license = (
        REPOSITORY_ROOT
        / "harness/template/.hive/LICENSE-AIGENT-HIVE.txt.jinja"
    ).read_bytes()
    if template_license != apache_license:
        raise AssertionError("rendered Apache license source diverged from canonical text")


def validate_skill_approvals(answers: dict[str, object]) -> None:
    approvals = answers["approved_optional_skills"]
    assert isinstance(approvals, list)
    for approval in approvals:
        assert isinstance(approval, dict)
        requested = set(approval["requested_capabilities"])
        approved = set(approval["approved_capabilities"])
        if not approved <= requested:
            raise AssertionError(
                f"approved capabilities exceed requested capabilities: {approval['name']}"
            )
        if approval["requested_capabilities"] != sorted(requested):
            raise AssertionError(f"requested capabilities are not canonical: {approval['name']}")
        if approval["approved_capabilities"] != sorted(approved):
            raise AssertionError(f"approved capabilities are not canonical: {approval['name']}")

        consent_payload = {field: approval[field] for field in CONSENT_FIELDS}
        expected_digest = canonical_digest(consent_payload)
        if approval["consent_digest"] != expected_digest:
            raise AssertionError(f"consent digest mismatch: {approval['name']}")


def validate_hook_approvals(answers: dict[str, object]) -> None:
    approvals = answers["approved_fallback_hooks"]
    assert isinstance(approvals, list)
    for approval in approvals:
        assert isinstance(approval, dict)
        consent_payload = {field: approval[field] for field in HOOK_CONSENT_FIELDS}
        expected_digest = canonical_digest(consent_payload)
        if approval["consent_digest"] != expected_digest:
            raise AssertionError(
                f"fallback hook consent digest mismatch: {approval['capability']}"
            )


def validate_consent_tamper_detection(approval: dict[str, object]) -> None:
    mutations = {
        "name": "changed-name",
        "source": "https://example.invalid/changed-source",
        "revision": "v1.0.1",
        "content_digest": f"sha256:{'f' * 64}",
        "requested_capabilities": ["filesystem-read", "network", "shell"],
        "approved_capabilities": [],
        "approved_at": "2026-07-23T00:00:01Z",
    }
    for field, changed_value in mutations.items():
        tampered = copy.deepcopy(approval)
        tampered[field] = changed_value
        try:
            validate_skill_approvals({"approved_optional_skills": [tampered]})
        except AssertionError:
            continue
        raise AssertionError(f"consent tamper was not detected: {field}")


def materialize_role(role_seed: dict[str, object]) -> str:
    role_profile = {
        "schema_version": 1,
        **role_seed,
        "current_assignment": None,
        "handoff_path": None,
    }
    validate_instance("role-profile.schema.json", role_profile)
    frontmatter = json.dumps(
        role_profile,
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    )
    return (
        f"---\n{frontmatter}\n---\n"
        f"# {role_profile['display_name']}\n\n"
        "## Current assignment\n\n"
        "_Unassigned._\n\n"
        "## Handoff\n\n"
        "_No handoff yet._\n"
    )


def validate_role_materialization(role_seeds: list[object]) -> None:
    seen_role_ids: set[str] = set()
    for role_seed in role_seeds:
        assert isinstance(role_seed, dict)
        role_id = role_seed["role_id"]
        assert isinstance(role_id, str)
        if role_id.casefold() in seen_role_ids:
            raise AssertionError(f"duplicate role id: {role_id}")
        seen_role_ids.add(role_id.casefold())

        first_render = materialize_role(role_seed)
        second_render = materialize_role(role_seed)
        if first_render != second_render:
            raise AssertionError(f"role materialization is not idempotent: {role_id}")

        if role_id == "reviewer":
            expected_path = (
                REPOSITORY_ROOT / "tests/fixtures/expected/reviewer-role.md"
            )
            if first_render != expected_path.read_text(encoding="utf-8"):
                raise AssertionError("reviewer role materialization changed")


def validate_render(render_root: Path, input_data_path: Path) -> None:
    expected_input = read_yaml(input_data_path)
    assert isinstance(expected_input, dict)

    required_paths = [
        "AGENTS.md",
        ".hive/.gitignore",
        ".hive/LICENSE-AIGENT-HIVE.txt",
        ".hive/README.md",
        ".hive/setup-answers.yml",
        ".hive/config/active-skills.yml",
        ".hive/config/harness.toml",
        ".hive/config/capability-resolution.yml",
        ".hive/config/role-seeds.yml",
        ".hive/config/knowledge-scope.yml",
        ".hive/config/approved-skills.yml",
        ".hive/knowledge/Wiki/index.md",
        ".hive/knowledge/Schema/schema.md",
        ".hive/knowledge/suppression.yml",
        ".hive/team/roles/README.md",
        ".hive/runs/README.md",
    ]
    for relative_path in required_paths:
        path = render_root / relative_path
        if not path.is_file():
            raise AssertionError(f"missing rendered path: {relative_path}")

    rendered_license = render_root / ".hive/LICENSE-AIGENT-HIVE.txt"
    if rendered_license.read_bytes() != APACHE_LICENSE_PATH.read_bytes():
        raise AssertionError("rendered Apache license text changed")
    if b"/runtime/\n" not in (render_root / ".hive/.gitignore").read_bytes():
        raise AssertionError("ephemeral runtime evidence is not ignored")
    if (render_root / ".hive/runtime").exists():
        raise AssertionError("setup rendered ephemeral runtime evidence")
    if (render_root / "LICENSE").exists() or (render_root / "LICENSE.md").exists():
        raise AssertionError("consumer project root license must remain untouched")

    answers = read_yaml(render_root / ".hive/setup-answers.yml")
    assert isinstance(answers, dict)
    validate_instance("setup-answers.schema.json", answers)
    validate_skill_approvals(answers)
    validate_hook_approvals(answers)
    validate_role_materialization(answers["persistent_roles"])

    approvals = answers["approved_optional_skills"]
    assert isinstance(approvals, list)
    for approval in approvals:
        assert isinstance(approval, dict)
        validate_consent_tamper_detection(approval)

    for key, expected_value in expected_input.items():
        if key == "capability_resolution":
            continue
        if answers[key] != expected_value:
            raise AssertionError(f"answer mismatch for {key}")

    with (render_root / ".hive/config/harness.toml").open("rb") as stream:
        harness_config = tomllib.load(stream)
    workspace_version = read_toml(REPOSITORY_ROOT / "Cargo.toml")["workspace"]["package"][
        "version"
    ]
    if (
        harness_config["harness_version"] != workspace_version
        or harness_config["source_release_version"] != workspace_version
    ):
        raise AssertionError("installed harness version differs from Cargo workspace")
    if harness_config["project_name"] != answers["project_name"]:
        raise AssertionError("project_name changed during TOML rendering")
    if (
        harness_config["usage_stop_remaining_percent"]
        != answers["usage_stop_remaining_percent"]
    ):
        raise AssertionError("usage stop threshold changed during TOML rendering")
    if (
        "usage_stop_remaining_percent" not in expected_input
        and answers["usage_stop_remaining_percent"] != 10
    ):
        raise AssertionError("default usage stop threshold must remain 10 percent")

    capability_resolution = read_yaml(
        render_root / ".hive/config/capability-resolution.yml"
    )
    validate_capability_resolution(capability_resolution)
    assert isinstance(capability_resolution, dict)
    expected_capability_resolution = expected_input.get("capability_resolution")
    if (
        expected_capability_resolution is not None
        and capability_resolution != expected_capability_resolution
    ):
        raise AssertionError("capability resolution projection lost input data")
    if capability_resolution["host"] != answers["primary_host"]:
        raise AssertionError("capability resolution host differs from primary_host")
    if (
        harness_config["external_capability_detection"]
        != capability_resolution["detection"]
    ):
        raise AssertionError("capability detection changed during TOML rendering")
    if harness_config["resolved_owner"] != capability_resolution["resolved_owner"]:
        raise AssertionError("resolved owner changed during TOML rendering")
    if (
        harness_config["resolution_evidence_digest"]
        != capability_resolution["evidence_digest"]
    ):
        raise AssertionError("resolution evidence digest changed during TOML rendering")

    hook_approvals = answers["approved_fallback_hooks"]
    assert isinstance(hook_approvals, list)
    approved_hooks_path = render_root / ".hive/config/approved-hooks.yml"
    hook_eligible = capability_resolution["detection"] == "absent"
    if hook_approvals and not hook_eligible:
        raise AssertionError("non-absent capability retained fallback hook approvals")
    if hook_approvals:
        if not approved_hooks_path.is_file():
            raise AssertionError("eligible fallback hook ledger was not rendered")
        approved_hooks = read_yaml(approved_hooks_path)
        validate_instance("hook-consent.schema.json", approved_hooks)
        assert isinstance(approved_hooks, dict)
        if approved_hooks["hooks"] != hook_approvals:
            raise AssertionError("fallback hook ledger lost setup approval data")
        if (
            approved_hooks["resolution_evidence_digest"]
            != capability_resolution["evidence_digest"]
        ):
            raise AssertionError("fallback hook ledger lost capability evidence binding")
        expected_hook_paths = set()
        for approval in hook_approvals:
            assert isinstance(approval, dict)
            descriptor = {
                "capability": approval["capability"],
                "command": approval["command"],
                "event": approval["event"],
                "path": approval["path"],
                "schema_version": 1,
            }
            descriptor_bytes = (
                json.dumps(
                    descriptor,
                    ensure_ascii=False,
                    separators=(",", ":"),
                    sort_keys=True,
                ).encode("utf-8")
                + b"\n"
            )
            descriptor_path = render_root / str(approval["path"])
            expected_hook_paths.add(descriptor_path)
            if not descriptor_path.is_file():
                raise AssertionError(
                    f"approved hook descriptor is missing: {approval['path']}"
                )
            if descriptor_path.read_bytes() != descriptor_bytes:
                raise AssertionError(
                    f"approved hook descriptor bytes changed: {approval['path']}"
                )
            if (
                f"sha256:{hashlib.sha256(descriptor_bytes).hexdigest()}"
                != approval["content_digest"]
            ):
                raise AssertionError(
                    f"approved hook content digest mismatch: {approval['path']}"
                )
        rendered_hook_root = render_root / ".hive/hooks"
        actual_hook_paths = (
            {path for path in rendered_hook_root.iterdir() if path.is_file()}
            if rendered_hook_root.is_dir()
            else set()
        )
        if actual_hook_paths != expected_hook_paths:
            raise AssertionError("rendered hook descriptor tree exceeds approvals")
    elif approved_hooks_path.exists():
        raise AssertionError("fallback hook ledger exists without eligible approval")
    elif (render_root / ".hive/hooks").exists():
        raise AssertionError("fallback hook descriptor tree exists without approval")

    role_seeds = read_yaml(render_root / ".hive/config/role-seeds.yml")
    knowledge_scope = read_yaml(render_root / ".hive/config/knowledge-scope.yml")
    approved_skills = read_yaml(render_root / ".hive/config/approved-skills.yml")
    suppression = read_yaml(render_root / ".hive/knowledge/suppression.yml")

    if role_seeds["roles"] != answers["persistent_roles"]:
        raise AssertionError("persistent role projection lost setup data")
    if knowledge_scope["include"] != answers["knowledge_include_paths"]:
        raise AssertionError("knowledge include projection lost setup data")
    if knowledge_scope["exclude"] != answers["knowledge_exclude_paths"]:
        raise AssertionError("knowledge exclude projection lost setup data")
    if approved_skills["skills"] != answers["approved_optional_skills"]:
        raise AssertionError("optional Skill approval projection lost setup data")
    if suppression != {"schema_version": 1, "entries": []}:
        raise AssertionError("unexpected suppression seed")

    active_skills_path = render_root / ".hive/config/active-skills.yml"
    active_skills = read_yaml(active_skills_path)
    validate_instance("active-skills.schema.json", active_skills)
    assert isinstance(active_skills, dict)
    active_entries = active_skills["skills"]
    assert isinstance(active_entries, list)
    expected_skill_names = [
        "hive-knowledge-capture",
        "hive-knowledge-maintenance",
        "hive-knowledge-query",
        "hive-prompt-refine",
        "hive-role-handoff",
        "hive-run-checkpoint",
        "hive-run-resume",
        "hive-simple-question",
        "setup-harness",
    ]
    if [entry["name"] for entry in active_entries] != expected_skill_names:
        raise AssertionError("Copier activated an unexpected Skill set")
    if active_skills_path.read_bytes() != (
        REPOSITORY_ROOT / "harness/template/.hive/config/active-skills.yml"
    ).read_bytes():
        raise AssertionError("Copier active Skill ledger changed from source bytes")

    primary_host = answers["primary_host"]
    projection_root = ".claude" if primary_host == "claude" else ".agents"
    foreign_projection_root = ".agents" if primary_host == "claude" else ".claude"
    projected_skill_root = render_root / projection_root / "skills"
    if (
        not projected_skill_root.is_dir()
        or {path.name for path in projected_skill_root.iterdir()}
        != set(expected_skill_names)
    ):
        raise AssertionError(
            "Copier projected optional or unexpected local Skill sources"
        )
    for entry in active_entries:
        assert isinstance(entry, dict)
        name = entry["name"]
        source_path = REPOSITORY_ROOT / f"harness/skills/{name}/SKILL.md"
        projection_path = (
            render_root / projection_root / f"skills/{name}/SKILL.md"
        )
        if not projection_path.is_file():
            raise AssertionError(
                f"missing {primary_host} built-in Skill projection: {name}"
            )
        if projection_path.read_bytes() != source_path.read_bytes():
            raise AssertionError(
                f"built-in Skill projection bytes changed: {name}"
            )
        if {path.name for path in projection_path.parent.iterdir()} != {
            "SKILL.md"
        }:
            raise AssertionError(
                f"built-in Skill projection contains unexpected files: {name}"
            )
        expected_content_digest = (
            f"sha256:{hashlib.sha256(source_path.read_bytes()).hexdigest()}"
        )
        if entry["content_digest"] != expected_content_digest:
            raise AssertionError(
                f"built-in Skill digest differs from source bytes: {name}"
            )
        if (
            entry["source_type"] != "built-in"
            or entry["consent_digest"] is not None
        ):
            raise AssertionError(
                f"Copier activated a non-built-in Skill without install flow: {name}"
            )
    if (render_root / foreign_projection_root).exists():
        raise AssertionError(
            f"foreign host projection rendered: {foreign_projection_root}"
        )

    forbidden_outputs = [
        ".hive/index/hive.sqlite",
        ".hive/runtime",
        ".omx",
        ".omc",
        ".codex",
    ]
    for relative_path in forbidden_outputs:
        if (render_root / relative_path).exists():
            raise AssertionError(f"forbidden rendered output: {relative_path}")


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("render_root", type=Path)
    parser.add_argument("input_data", type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    validate_schema_documents()
    validate_contract_examples()
    validate_license_boundary()
    validate_render(arguments.render_root, arguments.input_data)
    print(f"validated scaffold: {arguments.render_root}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
