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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:dff827b04011d7ad8ff7412543aaccfe570d63a10bb0370f4c52409544504cb4"
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:b3f259173a92bf050ca26df5b462330ea4ce9d96bb9bca431b6a54700ddfea36"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a72952ecc2423f82bd6710e9d88a88cd6b59f0d87500b30c1a1d68151879bcf1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d16fe3cf14d23b5781bbb960884146b17c3770916a5572215e4922a4423f28d1"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:40d08d401f60962d70f8bb4fcb83b4f17b3b067d165e24db9f579cf364b63b2d"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:b1c993c5c9332596f2f71f0db2e40f42a434e210f71f4e8bb9f38da763c4b673"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
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
