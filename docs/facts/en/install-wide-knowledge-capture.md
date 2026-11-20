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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:32db20fc6252cd4a5dcf6fac16ff4e7da275a8c0ca41ddd93c3eb6f6b1f28f68"
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:b2dab4ddd8cc0336bccd28c1bd61ba66901a2984b5306040ef994cdf355b1e35"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a0cf1b55b6cfc9d02920f117d25fb752e25267093d1be960b33267b8f6b0a94c"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:3064034c88ce11c947486618e149d977963e8bc15f4a7c3a0acbe1b428f2fcf8"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:3ddc5bf3e58c774266ce2bb11654fa54c80e0a09c519347b9de54d61eedaaa11"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:d80f5871428941be1d05b12d488857f40bb894752c0c4354cee12f15a46578f6"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:6b0b2a3765c90078e1e530e8b70322d531c17053731c5f4c9c9753423270a509"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Install-wide knowledge capture

Global Wiki guidance reviews turns in every selected-host folder. Project setup, a harness, marker or attached collection is unnecessary for safe user-root capture. Unregistered retrieval excludes private and confidential knowledge. Capture is foreground, agent-reviewed, normalized and bounded; no raw-prompt recorder. Observed exhaustion defers capture and lint without another tool call until a new request receives fresh allowed usage. The archived 0.9.1 Windows evidence covers automatic capture and fresh-session recall.
