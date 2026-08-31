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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:676c827ebe9ed1c4e51d73cf35289b8dfa9149f52287bcfad1a33d0b872b3be2"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:1518c1b9ac4f68d114a59603a490491221b0459e36137fb380d2c247f9e1ab1a"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:9e86075240574d1e589329ae724c97fac32dab2e2d367b7d878bd84e69d4b483"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:9e169f3daff2b4fbe6cff4d9a93d7e45cca6e9a6e78d1784b83458b50d3aa267"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
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
