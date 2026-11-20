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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:4de73afc7e39e6cfb92df9241c16f48f71c2870eedf93b55c6e81317cf30c8cb"
  - "repo:crates/hive-cli/src/session.rs#sha256:017435bb594baffef5c36d9b90404aa676429ef00bd674173b4fd46a982f0aae"
  - "repo:docs/archive/plans/release-0.10.0.md#sha256:2b8007e0cbf5a0f89ebb654ee7f6b44a1b203eee905205fe7ea90629941e4cad"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:harness/directives/03-session-coordination.md#sha256:bdef62f4f837e3c0c84a794a9f4c7bca1b8947a92b5b3ea66ff8314bb2c7dfec"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:01cf16fb17646efbb3fa714d6f851dcb6b19b22914547f5316136fafb145611a"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:d2232126a1ba7a68c97ecf4092447d527b564979acc291bb581b2ae304222212"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
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
