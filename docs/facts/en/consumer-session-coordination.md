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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:1ab23dbb5a8b60322f6c1068ba7cc09f54bc34151915068a748a1cc5a2c38018"
  - "repo:crates/hive-cli/src/session.rs#sha256:a41fd95ca2576269d347b4635afa064dc9dc53a70d96434daef1eb819e0fbf19"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:2b8007e0cbf5a0f89ebb654ee7f6b44a1b203eee905205fe7ea90629941e4cad"
  - "repo:harness/directives/03-session-coordination.md#sha256:bdef62f4f837e3c0c84a794a9f4c7bca1b8947a92b5b3ea66ff8314bb2c7dfec"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:fc6baab602cfbbf88acfc7a59a516d7f97b2f6c4f7067f63b31ad4bec432be89"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:616c9ccf6f3d99bbcec57c338ff27003b2ea731f15a1832da5c580bbb84cd65b"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
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
