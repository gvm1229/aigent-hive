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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:17b4e24061b7214faa292fa50e65e9b0d9902270bdbe86fdc06ae53b7970bf05"
  - "repo:crates/hive-cli/src/session.rs#sha256:affa286cb1b1d23c2de042061af7092a89a137f1a1a5fa5762cc92bd5897e7af"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:699afb145c350c75d63009f295441dcbdda20449bee4d4cfd68c1c392e3ff0fe"
  - "repo:harness/directives/03-session-coordination.md#sha256:06736ffa5a1619bc238a39ecad068ae85b2136a8e33ccb0829badf932d8c19f3"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:f17a658f423c8df0f5ca2b1960c3ea53fec57cb2859459bfe77a049510e9adf2"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:3315a1ca6957fbd5dd34a8d1323f1d385226facf530ad83e10170a2938c17ad1"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
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
