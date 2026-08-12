---
schema_version: 1
pair_id: source-usage-guard
topic_slug: source-usage-guard
language: en
counterpart: ../ko/source-usage-guard.md
title: "Source Session Usage Guard"
summary: "Source development keeps every execution-boundary check through its repository gate and the single product usage guard."
tags: [guard, source, usage]
aliases: ["Source quota safeguard"]
sources:
  - "repo:docs/guides/source-usage-guard.md#sha256:5f3fb38548cc8c96cdf9cfe273b77dd4b11c3bea4e0d379c1fefdf40193a0213"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:7b64cee13b39806a519ee9d8387972a1e69da108e1075b8b0b873581d46c439b"
links: [automatic-dispatch-guard, source-development, windows-watcher-identity]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Source Session Usage Guard

The repository source guard checks the development session before each tool, mutation, external
write, push, and final answer. Product `usage-guard` owns user-facing usage controls, using the
user's global threshold and the repository's optional earlier-stop override. No source-only Skill,
adapter, or threshold state remains. A bypass still requires explicit intent and remains bound to
the current session and process.
