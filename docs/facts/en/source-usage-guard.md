---
schema_version: 1
pair_id: source-usage-guard
topic_slug: source-usage-guard
language: en
counterpart: ../ko/source-usage-guard.md
title: "Source Session Usage Guard"
summary: "Source development keeps every execution-boundary check while migrating to the single product usage guard."
tags: [guard, source, usage]
aliases: ["Source quota safeguard"]
sources:
  - "repo:docs/guides/source-usage-guard.md#sha256:febe2420d8bd962cf11efaec3aa85df76bce57248e38068809acde71a3c80f8c"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:fed21b2de4b06f8034974ea611ce0afb2c0b09244a57c16238a2a1c662a131f8"
links: [automatic-dispatch-guard, source-development, windows-watcher-identity]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Source Session Usage Guard

The current source guard checks the development session before each tool, mutation, external write,
push, and final answer. The planned migration preserves those boundaries in the single product
`usage-guard`, using the user's global threshold and the repository's optional earlier-stop
override. The source-only Skill, adapter, and threshold state are then removed. A bypass still
requires explicit intent and remains bound to the current session and process.
