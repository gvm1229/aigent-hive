---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup must verify and resolve the signed CLI before questions, preserve every answer, and own the user-selected usage-guard threshold while projects may request only earlier stops."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:67c09e54e76df72ee9ac6acbde5b88fbb0a6653e1d7172e3f789a8d99c2434b7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:48a76cd5503858a2327c7562879de259334b182687aace98ec1df06b71dd1600"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:5a60d254c760db58049da72530895a981708d549700b02656c7ff51224140f5f"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:6742a157cb9665b6b3accffce536dd447642ec367c41fa06c7d5bc7ef6ca0910"
  - "repo:harness/skills/configure/SKILL.md#sha256:abeb032e21d2576366025465d54080966767fb7e17cca57848acf093eaa83eaf"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:6ed32f63fa3c67bed31164b9d15259f48443341a"
status: active
---

# Global Onboarding

Order: install CLI, activate a host, run global setup, then set up a project. Global Wiki source is
local Markdown; global setup never inspects a project.

Usage guard belongs to global setup. The user selects the safety threshold for every registered
project. A project can only choose a higher threshold; it stops earlier and cannot weaken
protection. No profile value exists.

Windows 11 test.5 found unresolved npm CLI lookup, schema guessing, 17+ temporary answer
files, lost progress, and no completed apply. The accepted repair requires verified CLI resolution
and a signed setup description before questions, progress after every answer, one cleaned OS-temp
file, and a fresh Windows numbered-test pass before stable 0.9.0. Unknown or edited bytes remain
unchanged.
