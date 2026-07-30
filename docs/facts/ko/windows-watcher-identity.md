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
  - "repo:tests/conformance/test_source_usage_guard.py#sha256:6b3a36c7bf6d41839d463f8239ef701221fc49c6303f5c849d2e589e8eadca7d"
links: [source-usage-guard]
reviewed_revision: "git:2c3485e6442d06871c9d61aec6c896c8fc93db11"
status: active
---

# Windows source watcher identity

Windows source usage watcher: 가상환경 launcher 대신 base CPython executable로 시작.
결과: 기록 PID와 lease owner 일치, bounded lease 대기, startup 실패 process 회수.
동시 atomic state 교체의 transient Windows permission conflict만 같은 bounded window에서
재시도.
