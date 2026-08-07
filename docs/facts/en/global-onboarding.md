---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup stores multiple user contexts without choosing project workflows, refreshes authenticated Hive-only drift automatically, and keeps Korean product terms and all built-in Skill defaults fixed."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:413ed120770591773c5efab11aa1bc3587687b411eff47a665802b5bf0f5ea2b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:6de1cf5f473fc0c6e61504b07ac8eb892abb77231b406d7952dc271e0ee23c1b"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:be6e9fd0b94f9cf8a994cce4bb1e8f5b0e8396420968832e285de366dc8e16f9"
  - "repo:harness/skills/configure/SKILL.md#sha256:17a80a35d5f367421c661374dec54147d0cabb4f48c4c5a640b15253bd5f0222"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:87bb452a4240faccdef5c96488b7492c3764f44a2819e8e7733b8c41dadc70b9"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:0c0a3fd18bd4b3746202c5a38aa7cb03d4b94908"
status: active
---

# Global Onboarding

Manual order: CLI installation, host activation, global setup, explicit project setup. The optional
one-prompt path starts global setup without project inspection.

Supported legacy recovery requires matching saved-preference and live-file evidence; other bytes
remain unchanged. An explicit global setup request automatically previews, applies, and revalidates
an authenticated Hive-only install or saved-answer user-projection refresh without a review-only
question. See `global-user-contexts` for contexts, Skill selection, and Korean product terms.
