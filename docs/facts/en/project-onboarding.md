---
schema_version: 1
pair_id: project-onboarding
topic_slug: project-onboarding
language: en
counterpart: ../ko/project-onboarding.md
title: "Project Onboarding"
summary: "Project setup inherits global preferences and asks only unresolved essentials."
tags: [onboarding, project]
aliases: ["Project setup"]
sources:
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:3589ba7f2032870f8d63346312cc0f5358700934de3cf6a84602bf3397cff801"
links: [global-onboarding, shared-index]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Project Onboarding

Project setup inherits operational user preferences, derives answers from canonical
repository evidence, and asks only unresolved essential questions before previewing
the exact Hive-owned write set.
