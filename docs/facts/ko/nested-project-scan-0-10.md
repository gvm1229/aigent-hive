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
  - "repo:crates/hive-cli/src/knowledge_scan.rs#sha256:8502081a51a31982649e9b945277dcceae0db3d058fcadba3a38be5e81ae9f29"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/nested-project-knowledge-scan-0.10.0.md#sha256:09e75e39def220648906afa58722a15a1997ca9013eeeb02f579b8eb4b1aaf8f"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:acc6b48abc6da40d145cc9a70ffca5b78a73d625cb7d383883eb23ea29290c33"
  - "repo:docs/research/ai-learning-hive-application-candidates-2026-08-21.md#sha256:14eb21209b147e7ca9947eae8afb09c059d53aedf353c1802620bf8bf4cc0038"
  - "repo:tests/conformance/integration/test_wiki_cli_e2e.py#sha256:7fe5b4532dfb5d4e60bc63fcac462a4b0eca0bd18a3a430edd693f1ac83862b5"
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
