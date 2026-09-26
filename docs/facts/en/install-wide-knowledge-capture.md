---
schema_version: 1
pair_id: install-wide-knowledge-capture
topic_slug: install-wide-knowledge-capture
language: en
counterpart: ../ko/install-wide-knowledge-capture.md
title: "Install-wide knowledge capture"
summary: "Hive user-level capture and recall apply in every selected-host project immediately after installation, without project setup."
tags: [capture, knowledge, retrieval, user-root]
aliases: ["Setup-independent knowledge", "Unregistered project recall"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:0ffab09ca1eaac47b41608e04003fdbc51ee7e5edbaf2386554c885d9f55ec58"
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:9010d4a0aa30eec68c1069f7b94a373846bb7a756bfe5d099183ce500164bde2"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a72952ecc2423f82bd6710e9d88a88cd6b59f0d87500b30c1a1d68151879bcf1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:3fbde6d532df67e0de9b15d1b480db9005f0a3247545bc1baf7e16afa7a7fed4"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:b1c993c5c9332596f2f71f0db2e40f42a434e210f71f4e8bb9f38da763c4b673"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:1cdebf8aef9dde963f5aa88a36042186fbf37e80"
status: active
---

# Install-wide knowledge capture

- Global Wiki user guidance reviews every turn in every selected-host folder after installation.
- Project setup, a Hive harness, a marker, and an attached collection are not user-root capture
  prerequisites.
- Unregistered-target retrieval searches user-root and shared knowledge while excluding private
  and confidential knowledge.
- Capture is foreground, agent-reviewed, normalized, and bounded; no raw-prompt recorder.
- `0.9.1` Windows acceptance: an ordinary PortareFolium career statement created a user-root claim
  and receipt; a separate fresh Codex session recalled it automatically.
