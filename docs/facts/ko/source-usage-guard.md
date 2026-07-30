---
schema_version: 1
pair_id: source-usage-guard
topic_slug: source-usage-guard
language: ko
counterpart: ../en/source-usage-guard.md
title: "Source session usage guard"
summary: "Source execution boundary별 current session quota 확인."
tags: [guard, source, usage]
aliases: ["Source quota safeguard"]
sources:
  - "repo:docs/guides/source-usage-guard.md#sha256:febe2420d8bd962cf11efaec3aa85df76bce57248e38068809acde71a3c80f8c"
links: [automatic-dispatch-guard, source-development]
reviewed_revision: "git:722c8e46dbde5710155b394ef33820ebccd3b85c"
status: active
---

# Source session usage guard

Source-only guard 확인 경계: tool, mutation, external write, push, final answer.
Bypass 조건: explicit intent와 current session·process binding.
