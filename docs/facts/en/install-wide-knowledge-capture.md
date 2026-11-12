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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:f7b7ae3eb3045a598684dcfac3fec94c9ce649a87d745c11a650da06e428fd2c"
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:b3f259173a92bf050ca26df5b462330ea4ce9d96bb9bca431b6a54700ddfea36"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a72952ecc2423f82bd6710e9d88a88cd6b59f0d87500b30c1a1d68151879bcf1"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2527e7ceccba9293bc6f0ca7a7ca53ae986ec82066f1f4c62abc8426e4f18ab8"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:7231f60877fd7d269ff3d3b9cc7851daefb42f75baef1682dacb4d9644feffe6"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:b1c993c5c9332596f2f71f0db2e40f42a434e210f71f4e8bb9f38da763c4b673"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:4c71dbbc592620844bff81765151014d4a657924d03aea602c08bc517e1c8727"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# Install-wide knowledge capture

Global Wiki guidance reviews turns in every selected-host folder. Project setup, a harness, marker or attached collection is unnecessary for safe user-root capture. Unregistered retrieval excludes private and confidential knowledge. Capture is foreground, agent-reviewed, normalized and bounded; no raw-prompt recorder. Observed exhaustion defers capture and lint without another tool call until a new request receives fresh allowed usage. The archived 0.9.1 Windows evidence covers automatic capture and fresh-session recall.
