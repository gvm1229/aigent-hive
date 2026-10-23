---
schema_version: 1
pair_id: usage-default-off-scope
topic_slug: usage-default-off-scope
language: en
counterpart: ../ko/usage-default-off-scope.md
title: "Usage Guard and Interactive CLI Scope"
summary: "The maintainer selected 0.14.0 for interactive user and project setup with explicit usage choices."
tags: [guard, planning, usage]
aliases: []
sources:
  - "repo:docs/decisions/interactive-cli-0.14.0.md#sha256:bd680e226a4a3391750ae3d2cd921fbaa3c4b36f298d6b61ad9f88dfd452aef7"
  - "repo:docs/plans/0.14.0-implementation.md#sha256:8f818ae7be8db3539f5722d6aa257a068999604f701340229101d7b476093f32"
  - "repo:docs/research/interactive-cli-0.14.0.md#sha256:ffd6dd3695c3ef9128e006d2191d93460fa177856b2c90c6621ce3c7ddc4297a"
links: [usage-guard-stop-layers]
reviewed_revision: "git:c8aeb4cb1dd2c3f5e18c2f94c6c4f9c20d02732c"
status: active
---

# Usage Guard and Interactive CLI Scope

The 0.14.0 plan includes interactive user and project setup, questions during the first update from public 0.13.1, and local setting-origin records. New usage stops default off. Existing active or unknown-origin choices require CLI review; unattended updates preserve choices and defer questions. Uninstall previews and confirms removal while preserving user data. Direct CLI setup targets zero model calls; AI-assisted setup needs measured input reduction and smaller-model qualification. This is a saved implementation plan, not implemented behavior. The next public test is 0.14.0-test.1.
