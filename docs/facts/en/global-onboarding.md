---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup verifies and resolves the signed CLI before questions, preserves answers, and reuses saved preferences after a preserving Hive uninstall."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:dbccfb9a0a4920baef62329aa1027751f9bebc5893fdcecdccd5b2cb3237e932"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:2dbd0f956fea6c6e258a275bc89565c48a7bf211819ea8816512215dc2582213"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:4f3676378fafac75f9c6376210c760a2e0200e843ead0825d1b34d7446864e34"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:01df1d580d987e7fb0f34978076cd000263fd99f"
status: active
---

# Global Onboarding

Order: CLI, host activation, global setup, project setup. Global setup skips projects; user thresholds
only stop projects earlier. Windows recovery: verified CLI metadata, saved progress, one OS-temp file,
product-only Skills, shared guard, silent incomplete-marketplace repair, knowledge and preference preservation.

`hive uninstall` removes only Hive-managed setup state. It preserves the knowledge base and saved
preferences, with no full-purge flag. Later user-scope install reuses preferences without setup
questions. Windows test.12 acceptance includes new-session discovery and Discord delivery. The next
release qualification uses a product-owned expedited default profile without contributor input.
