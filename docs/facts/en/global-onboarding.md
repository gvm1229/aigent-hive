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
  - "repo:README.md#sha256:30e7d1dece221c145e4a75fe9e05ec9520ca3ab58b7d1311088b9c4ad72759ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a4d69016bdf3c5f0e8ee75839c7076b804ce55a8583fa56aabd933545d148611"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:03fad2cb09cd32e0f9ecc6c586a1f088fbfec2d0a01094af320dc4cf4d9200d5"
  - "repo:harness/skills/setup-hive/SKILL.md#sha256:2851a369d75eaa79fe50ba9295787c09edbf7b25163e6fb64260ceba472db843"
  - "repo:harness/user-setup/catalog.yml#sha256:af1147b8468f48eb81ec77ed4a14d5eba2fd31a4302e5459544fec3b2e22b595"
  - "repo:schemas/user-setup.schema.json#sha256:b94594a2597f8eab3bcb778c24b892ee45c3856ce421043a79c3861b59cb99ee"
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
