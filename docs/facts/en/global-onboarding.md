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
  - "repo:README.md#sha256:a03aae178a8c1060d3f4301d4ed592a24e8cf9e9e95a7b87afa434804ad4ecbb"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:cb42f6c3bd643bc236f3af89f4388ffdbc08db66af88123a38267b904d7b9d01"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:d30564f33f2ead463cfe9e18aa68b697cb07b6c419ee42c9b583fcc11edaf966"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:4f61520861d38b63448a45b91dd96443dfba20c79b3d8abade6099460956d3ed"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:4f3676378fafac75f9c6376210c760a2e0200e843ead0825d1b34d7446864e34"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:e83e5f318a5b6ffcc08cfe0898a2b6138512c6bfb0eea99c6070b134f3712f47"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
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
