---
schema_version: 1
pair_id: source-usage-guard
topic_slug: source-usage-guard
language: en
counterpart: ../ko/source-usage-guard.md
title: "Source Session Usage Guard"
summary: "Source development checks the active session quota at every execution boundary."
tags: [guard, source, usage]
aliases: ["Source quota safeguard"]
sources:
  - "repo:docs/guides/source-usage-guard.md#sha256:febe2420d8bd962cf11efaec3aa85df76bce57248e38068809acde71a3c80f8c"
links: [automatic-dispatch-guard, source-development]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Source Session Usage Guard

The source-only guard checks the current development session before each tool,
mutation, external write, push, and final answer. A bypass requires explicit intent
and is bound only to the current session and process.
