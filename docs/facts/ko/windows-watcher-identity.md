---
schema_version: 1
pair_id: windows-watcher-identity
topic_slug: windows-watcher-identity
language: ko
counterpart: ../en/windows-watcher-identity.md
title: "Windows source watcher identity"
summary: "가상환경 launcher를 우회해 lease owner와 PID ownership 일치."
tags: [guard, source, windows]
aliases: ["Windows watcher PID"]
sources:
  - "repo:tests/conformance/test_source_usage_guard.py#sha256:e6d29a38db42c18e49763da9013171caf990bad1c054590ada8a939a699f36bf"
links: [source-usage-guard]
reviewed_revision: "git:a1fb6e848117b83354144540df01474e68d25aa8"
status: active
---

# Windows source watcher identity

Windows source usage watcher: 가상환경 launcher 대신 base CPython executable로 시작.
결과: 기록 PID와 lease owner 일치, bounded lease 대기, startup 실패 process 회수.
동시 atomic state 교체의 transient Windows permission conflict만 같은 bounded window에서
재시도.
