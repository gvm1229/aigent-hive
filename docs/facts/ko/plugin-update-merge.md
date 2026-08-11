---
schema_version: 1
pair_id: plugin-update-merge
topic_slug: plugin-update-merge
language: ko
counterpart: ../en/plugin-update-merge.md
title: "Plugin update merge"
summary: "Signed historical base 기반 local-priority three-way projection update."
tags: [merge, plugin, update]
aliases: ["Projection upgrade merge"]
sources:
  - "repo:docs/decisions/ADR-0009-user-plugin-project-knowledge-boundary.md#sha256:da1f8b5e6323b945a0a85740f32a8e4332cb3c9e9d446c6a1d5acc9846653454"
links: [project-onboarding, update-transaction]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Plugin update merge

Authenticated historical base로 unmodified·user-modified projection 구분. Unmodified:
exact incoming replace. Modified: overlapping local edit 우선, non-conflicting incoming
hunk만 추가.
