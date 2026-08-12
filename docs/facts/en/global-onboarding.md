---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup recommends the usage guard, hides CodexBar until native failure, and preserves user data across reinstall."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:748b53ba0cfdf7ed37da6ddf80bbf9b9659fd5ed5711b8d43d1f37f669aa4494"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:756a903f18f96a554c05fdc72b76e0bc515446e3046bea1de6632087e99370aa"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/decisions/product-release-decisions.md#sha256:7e6acd0973f56e3a15e4aad766a907c76a1511f6f5931c36f71ba8d979e90beb"
  - "repo:docs/plans/active/native-usage-sensor.md#sha256:8131d6eba753cae4bfc38ec30013a44385c92b50ed29a57de8a96c8b7395c246"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:7b64cee13b39806a519ee9d8387972a1e69da108e1075b8b0b873581d46c439b"
  - "repo:docs/plans/active/user-onboarding-shared-index.md#sha256:889f7b0e5f374b1c78117486dcd24bd02df5d96b00c340402f2c672eb54b3b61"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:422649ef3ca475aca9e3a86a2ddd2bbbb3895221d7bc39fe4417010664dee47f"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:90a8ecca713a1b1963b5f1863f76d32d5c5b9532ca72922c2705ee9b63520307"
  - "repo:harness/user-setup/catalog.yml#sha256:4926655a12591cae061e674d774557e96f000d149f8dec1c2b1b650ba235f494"
  - "repo:schemas/user-setup.schema.json#sha256:57a426a58c822271f1c6297c2c607e532e83c5652ca92ef68bdbcd8b95d357fd"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:01df1d580d987e7fb0f34978076cd000263fd99f"
status: active
---

# Global Onboarding

- The usage guard is recommended; expedited setup uses 20% remaining, and custom setup asks.
- CodexBar stays hidden until a post-initialization native failure is confirmed.
- Setup, update, and uninstall converge to the current Skill closure, remove retired empty shells,
  and preserve knowledge, preferences, and foreign bytes.
