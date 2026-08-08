---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup stores multiple user contexts without choosing project workflows, refreshes authenticated Hive-only drift automatically, keeps the v0.9 Wiki Markdown-only, and retains Korean product terms and all built-in Skill defaults."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:413ed120770591773c5efab11aa1bc3587687b411eff47a665802b5bf0f5ea2b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:fcbdc8566036c3c7601b661baed7380a5cb27412f22f5d3c2961dce0daa80c3d"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:be6e9fd0b94f9cf8a994cce4bb1e8f5b0e8396420968832e285de366dc8e16f9"
  - "repo:harness/skills/configure/SKILL.md#sha256:7df45794421f603fb274d9fc2ca2a58616043bb79ccab5ced373012a3f99b195"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:34cfb17b238af67733c1250f5de6306cf6c75ef9df41f1934d6f1edc46d4a2da"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:0c0a3fd18bd4b3746202c5a38aa7cb03d4b94908"
status: active
---

# Global Onboarding

Manual order: CLI installation, host activation, global setup, explicit project setup. In v0.9,
the global Wiki uses local Markdown as its only user-visible source of truth. The optional
one-prompt path starts global setup without project inspection.

Supported legacy recovery requires matching saved-preference and live-file evidence; other bytes
remain unchanged. An explicit global setup request automatically previews, applies, and revalidates
an authenticated Hive-only install or saved-answer user-projection refresh without a review-only
question. See `global-user-contexts` for contexts, Skill selection, and Korean product terms.
