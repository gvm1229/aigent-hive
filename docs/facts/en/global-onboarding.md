---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup stores multiple user contexts without choosing project workflows, refreshes authenticated Hive-only drift automatically, keeps the v0.9 Wiki Markdown-only, lists all partial-reconfiguration settings including Discord children, and retains Korean product terms and all built-in Skill defaults."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:67c09e54e76df72ee9ac6acbde5b88fbb0a6653e1d7172e3f789a8d99c2434b7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:48a76cd5503858a2327c7562879de259334b182687aace98ec1df06b71dd1600"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:315391aa3b280409c6c19185aff55bcd21af1fb724de89a7007fc84c73a44aa3"
  - "repo:harness/skills/configure/SKILL.md#sha256:abeb032e21d2576366025465d54080966767fb7e17cca57848acf093eaa83eaf"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:dbae17b5e5bb39d068891b823dcd14f42ae23e10"
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
