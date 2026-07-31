---
schema_version: 1
pair_id: source-watcher-process-replacement
topic_slug: source-watcher-process-replacement
language: ko
counterpart: ../en/source-watcher-process-replacement.md
title: "Source watcher process replacement"
summary: "같은 Codex 작업의 종료된 소유자 감시기만 안전하게 교체."
tags: [guard, process, source]
aliases: ["Stale watcher owner recovery"]
sources:
  - "repo:tests/conformance/test_source_usage_guard.py#sha256:e6d29a38db42c18e49763da9013171caf990bad1c054590ada8a939a699f36bf"
links: [source-usage-guard, windows-watcher-identity]
reviewed_revision: "git:a1fb6e848117b83354144540df01474e68d25aa8"
status: active
---

# Source watcher process replacement

같은 Codex 작업의 host process 교체: 종료된 이전 소유자의 creation identity와
watcher lease 검증 뒤 기존 watcher 회수·새 watcher 시작. 이전 소유자 활성 상태의
교체 거부, PID 재사용·관계없는 process 종료 방지. Session bypass의 process binding과
교체 과정의 non-transfer 유지.
