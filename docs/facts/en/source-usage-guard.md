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
  - "repo:docs/guides/source-usage-guard.md#sha256:3feed99484282ad4265e82d2f831859993f8292b92c8369cb57ee7b7b7c04c9d"
  - "repo:docs/plans/active/usage-guard-policy.md#sha256:97d33e84f001d7e456d24cd95b9712bbe8ef5c9133acd5a6f94ca6395981a066"
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
