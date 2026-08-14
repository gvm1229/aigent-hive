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
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:9091a6094f11be32f27108944ec98adbd0dc425afb6faa26ba8cf616f18d8896"
links: [global-onboarding, shared-index]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Project Onboarding

Project setup inherits operational user preferences, derives answers from canonical
repository evidence, and asks only unresolved essential questions before previewing
the exact Hive-owned write set.
