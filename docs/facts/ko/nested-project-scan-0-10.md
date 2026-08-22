---
schema_version: 1
pair_id: nested-project-scan-0-10
topic_slug: nested-project-scan-0-10
language: ko
counterpart: ../en/nested-project-scan-0-10.md
title: "Nested project scan 0.10.0 범위"
summary: "0.9.5의 0.9 release line 마감과 상위 Git repository 아래 registered project의 안전한 knowledge scan을 0.10.0으로 편입."
tags: [knowledge, release, scan, v0-10]
aliases: ["Nested Vault scan"]
sources:
  - "repo:crates/hive-cli/src/knowledge_scan.rs#sha256:61bf8cc01a6e0701b89e047ffd0f0118c676a84e8edc7b316cd9c424bbae4f48"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3fbe246c3a5b7d2b8ec002d40f73874c056c48ae3a888dede3e40db12eddddac"
  - "repo:docs/plans/active/nested-project-knowledge-scan-0.10.0.md#sha256:09e75e39def220648906afa58722a15a1997ca9013eeeb02f579b8eb4b1aaf8f"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:9ad8cc57c6fc06818272c37bdeabdd00c03b10df953dace3e1da693487eca295"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:tests/conformance/integration/test_wiki_cli_e2e.py#sha256:f737ec5b335045a43839360e02c5ed9c2c52d0b9f59394123087fd2063727c12"
links: [knowledge-portability-scan, version-policy]
reviewed_revision: "git:d019b6023bb5b8705da027af638a87b8da3de13d"
status: active
---

# Nested project scan `0.10.0` 범위

- Release 결정: `0.9.5`가 마지막 `0.9.x`, `0.9.6` 게시 계획 없음
- `SCP10-003`: 상위 Git repository 아래 registered project root의 knowledge scan 구현·검증 완료
- 회귀 증거: nested target inventory와 foreign sibling sentinel byte 불변
- 수락 경계: 등록 root 안 한정, sibling 접근·전역 Git 설정 변경 `0건`
- 탈출 방어: symlink·junction·reparse point 거부
