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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:a00d240fa71fecf28877a43253cdc20190279d9e3d5d0b63bf0ad8a47ab9b7de"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:665b5cbf2f5f7d0fdb59bbe7515b5264e8cba4a24d2ad980d5401e6965d66d16"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:d2e23636ac998bf0b8cca29cf2466e761ab6abe6f20313ad0c9b4d2c6cf71459"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:f06146778f6faf907e402462008e970bc82cf134f9e8cb9c31a3b727b20e66ec"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:527434f7364b6be38e7b6941bf48df207c58b32c"
status: active
---

# Install-wide knowledge capture

When the global Wiki is enabled, Hive's selected-host user guidance reviews every turn in every
folder immediately after installation. Project setup, a Hive harness, a project marker, and an
attached collection are not prerequisites for safe user-root capture. Automatic retrieval from an
unregistered target searches user-root and shared collections while excluding project-private and
confidential knowledge. Capture remains foreground, agent-reviewed, normalized, and bounded; Hive
does not run a background raw-prompt recorder.
