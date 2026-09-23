---
schema_version: 1
pair_id: consumer-session-coordination
topic_slug: consumer-session-coordination
language: en
counterpart: ../ko/consumer-session-coordination.md
title: "Consumer Session Coordination"
summary: "Hive coordinates overlapping automated consumer-project edits through small path reservations without claiming control over direct user edits; the 0.10.0 scope adds host-owned project Skill reservation support."
tags: [consumer-harness, preservation, session, upgrade]
aliases: ["CHS93"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:d5e36f5d1cb6080fa7952b1cf4354e7d54f0df12bc0799d758ced53d7f083b84"
  - "repo:crates/hive-cli/src/session.rs#sha256:a41fd95ca2576269d347b4635afa064dc9dc53a70d96434daef1eb819e0fbf19"
  - "repo:docs/decisions/product-release-decisions.md#sha256:9f234ef3fede8030ab6ad57fa4b560a71f30f468622c4e4ad56b83864d0e05ce"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:2b8007e0cbf5a0f89ebb654ee7f6b44a1b203eee905205fe7ea90629941e4cad"
  - "repo:harness/directives/03-session-coordination.md#sha256:bdef62f4f837e3c0c84a794a9f4c7bca1b8947a92b5b3ea66ff8314bb2c7dfec"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:aef70c66a054607e84288fa44f5fefca3c089515d41cb0c06934498ebbf31ca1"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:62e3928ff0df3ba848043b773f04ef00df40c26cd6e7c32148169094b96f73ed"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Consumer Session Coordination

`hive session begin|check|update|close|recover` keeps ephemeral, Git-ignored path reservations
under `.hive/runtime/active-sessions/`. Parent, child, and identical paths conflict across live Hive
sessions; direct user or external-editor writes remain outside Hive control. Project upgrades preview
and apply only a directly conflicting Hive-owned directive clause, preserving user-authored, foreign,
and non-conflicting local bytes.

`0.10.0`: Codex·Antigravity `.agents/skills/<safe-skill>/...`, Claude
`.claude/skills/<safe-skill>/...` reservation. A host mismatch returns
`hive.session-host-owned-namespace`; recovery advice is only for live or unverifiable reservations.
