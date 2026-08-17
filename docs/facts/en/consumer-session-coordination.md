---
schema_version: 1
pair_id: consumer-session-coordination
topic_slug: consumer-session-coordination
language: en
counterpart: ../ko/consumer-session-coordination.md
title: "Consumer Session Coordination"
summary: "Hive coordinates overlapping automated consumer-project edits through small path reservations without claiming control over direct user edits."
tags: [consumer-harness, preservation, session, upgrade]
aliases: ["CHS93"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:af09aadf2ddfabc082dfac9ae6c8233c2fe48f964db8996063848838f04f68c5"
  - "repo:crates/hive-cli/src/session.rs#sha256:1a6fd68e66a00f5f3343b801479f564f4667a7123793b43f9d2d2c94648f0b9d"
  - "repo:harness/directives/03-session-coordination.md#sha256:06736ffa5a1619bc238a39ecad068ae85b2136a8e33ccb0829badf932d8c19f3"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:5a4e0367ff8d52dd58221f15e16aa16a1ddee89fcc6cfef3fabe0c47f0e1babb"
  - "repo:tests/conformance/test_project_lifecycle.py#sha256:6907af3716ce13652850f367a8effc5c92910c909ee34ae94341f2b5b50b5b52"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:a52362971c8fa646b428449dd85681491eaeb184"
status: active
---

# Consumer Session Coordination

`hive session begin|check|update|close|recover` keeps ephemeral, Git-ignored path reservations
under `.hive/runtime/active-sessions/`. Parent, child, and identical paths conflict across live Hive
sessions; direct user or external-editor writes remain outside Hive control. Project upgrades preview
and apply only a directly conflicting Hive-owned directive clause, preserving user-authored, foreign,
and non-conflicting local bytes.
