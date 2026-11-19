---
schema_version: 1
pair_id: fable-advisor-ownership
topic_slug: fable-advisor-ownership
language: en
counterpart: ../ko/fable-advisor-ownership.md
title: "Fable 검토 소유권"
summary: "Fable 검토 요청의 관리자 원문 인용과 소유자 판단, 제안 기록·계획 연결"
tags: [directives, review]
aliases: []
sources:
  - "repo:.agents/directives/references/agent-delegation.md#sha256:698fc94a7f2304a6e19ab4d96c8ac0337f4e0856aa378cf6573b5657e1be76a9"
links: [agent-directive-ownership]
reviewed_revision: "git:23107415bcc1287ae88d92fdee2d303e732ef0a6"
status: active
---

# Fable 검토 소유권

Every Fable advisor or plain-Fable stage-review brief quotes the maintainer ownership statement
in agent-delegation.md. The read-only advisor owns product purpose and architecture decisions,
and may propose large changes and concrete tasks for delegation. The orchestrator records all
proposals in review evidence and routes accepted ones to the owning plan or a new stage item.
Executors edit; existing safety, plan ownership and version-specific approvals remain in force.
