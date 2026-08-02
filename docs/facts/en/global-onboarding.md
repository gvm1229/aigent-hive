---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Initial setup chooses language first and records user-scope host preferences."
tags: [onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:44401a82ba3bd9f2bc4048876f5480157720bc5fce005a7c0b63f4d960f63bf1"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Global Onboarding

Initial setup asks for English or Korean first. It then records the selected language,
hosts, profile, persona, Skills, Wiki preference, and optional usage-guard consent at
user scope.
