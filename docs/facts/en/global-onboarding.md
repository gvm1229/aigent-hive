---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Optional bootstrap preserves scope; Korean setup keeps exact terms, user context, and all built-in Skills by default."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:6ddf3dd877c31e3f6e525ea6a659fdf90233cbf008cfc3be355f271267c9fa94"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2c61916f31b5a6ae66f6c2a615c41bcf4ac91ea2ca95d388f5d357cd5d872269"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0292baa97d8ec193709ae756e56393af34085d781d7c341fe5d0d1ab0ed244e0"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2e064212050d755bf101322fdcc94f8a737db7b59204b75bb6bfcd64d8e32ceb"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:a30eb472f72773109da16706f4dbcb81cef76421"
status: active
---

# Global Onboarding

Manual order: CLI installation, host activation, global setup, explicit project setup. The optional
one-prompt path starts global setup without project inspection.

Supported legacy recovery requires matching saved-preference and live-file evidence; other bytes
remain unchanged. Korean setup retains `Skill` and `Wiki` exactly with canonical regression samples.

Global profiles retain nonexclusive user context only. Project workflow, technical choices, and
work priority belong to project scope.

Global setup enables all built-in Skills by default. Per-Skill toggles replace profile-bound
recommended suites. The typed user configuration accepts only `all|individual`; a saved legacy
suite preserves its recorded closure until an approved preview writes the new form.
