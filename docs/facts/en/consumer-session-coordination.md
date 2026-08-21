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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:af09aadf2ddfabc082dfac9ae6c8233c2fe48f964db8996063848838f04f68c5"
  - "repo:crates/hive-cli/src/session.rs#sha256:1a6fd68e66a00f5f3343b801479f564f4667a7123793b43f9d2d2c94648f0b9d"
  - "repo:docs/decisions/product-release-decisions.md#sha256:8d73ee9596c08dbf23bab845b1d1c1bed0a86f8d297510b505c9c54dcdee90e1"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:6b7c846152acb98e56c5b5f548d550087ab50eb6bdb4f92d4c9ba0ada79092dc"
  - "repo:harness/directives/03-session-coordination.md#sha256:06736ffa5a1619bc238a39ecad068ae85b2136a8e33ccb0829badf932d8c19f3"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:5a4e0367ff8d52dd58221f15e16aa16a1ddee89fcc6cfef3fabe0c47f0e1babb"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:c066a315884e9c93499da3db8dad0b0abf41279c89bbb467e1a6deb6b2a0842f"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:69697caef2ce83ce939c828e64b55fa349329f82"
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
