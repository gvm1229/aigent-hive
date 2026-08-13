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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:4213e512a4d14323b92f0fef7f4ae77055c441c064b84da17f9b883dbb38ff3c"
  - "repo:crates/hive-cli/src/session.rs#sha256:5fb63a89a7b8ef6c7da74e31221ad5d573939d633932ddb235f12383ce452fe6"
  - "repo:harness/directives/03-session-coordination.md#sha256:06736ffa5a1619bc238a39ecad068ae85b2136a8e33ccb0829badf932d8c19f3"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:376242cfffb02880c994804d608a176bfbdef3e57d2f40493a86cdf5405f798e"
  - "repo:tests/conformance/test_project_lifecycle.py#sha256:7862526b5478758639c55e6ac966e0e24e1f7d9b6d5ef26e99109a33c7abca49"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:a506aeec8a0c461cf8936e604304a85154fc6763"
status: active
---

# Consumer Session Coordination

`hive session begin|check|update|close|recover` keeps ephemeral, Git-ignored path reservations
under `.hive/runtime/active-sessions/`. Parent, child, and identical paths conflict across live Hive
sessions; direct user or external-editor writes remain outside Hive control. Project upgrades preview
and apply only a directly conflicting Hive-owned directive clause, preserving user-authored, foreign,
and non-conflicting local bytes.
