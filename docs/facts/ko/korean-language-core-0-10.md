---
schema_version: 1
pair_id: korean-language-core-0-10
topic_slug: korean-language-core-0-10
language: ko
counterpart: ../en/korean-language-core-0-10.md
title: "0.10.0 한국어 언어 core"
summary: "고정 im-not-ai 파생 rule pack·결정적 보존 gate·host-owned 국소 rewrite·humanize-kor·승인형 pack rollback 기반 한국어 언어 core"
tags: [korean, language, skill, v0-10]
aliases: ["Korean output gate", "humanize-kor"]
sources:
  - "repo:.github/workflows/public-test-acceptance.yml#sha256:31283b71f7c1a61e23658d2de67d119c5fb5886990ed3b788489a0a80ea14d12"
  - "repo:crates/hive-core/src/korean.rs#sha256:bb575d5e73f1567755656c7e6be98cca871416a052e83e920d95b91e77186188"
  - "repo:docs/architecture/korean-language-core.md#sha256:2ae475f2f4c701f42fe28bff62cb37e60ef516526b2f147d1ec1544e2b32bfa4"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:f93dba421edc980e3a9ca8b5a8ce2ee978806094ae360203371948a27bcadaec"
  - "repo:harness/language-packs/im-not-ai/2.3.2/manifest.json#sha256:50e8bec5fb4c7a479f9e0800f262d49c3e01258ba3c7b9066aab65ba3f7ca34e"
  - "repo:harness/skills/humanize-kor/SKILL.md#sha256:b356691df025bb30def279528450be5c5c9085adf11457efcda87834ef452f67"
  - "repo:scripts/qualify-korean-public-test.py#sha256:f65f27b409d902b3d44beb1fd7f30f843eacbcb7f3acf5c9288bc04bef659a0c"
links: [language-consistency, public-skill-identity, v0-10-product-scope]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# `0.10.0` 한국어 언어 core

- 다섯 검사 유형과 `humanize-kor`: 대상의 검증된 활성 언어 팩 공통 사용
- 규칙 구조·보호 구간·변경률·부정 문맥 검사, 전체 의미 동등성은 호스트의 별도 검토 대상
- 호스트 소유 윤문·자동 재시도 최대 1회·실패 시 정확한 원문 보존
- 고정 `im-not-ai 2.3.2`: 승인형 단계별 활성화·복원 대상 해시 검사·현재 팩 손상 복구
- Windows·macOS·Linux 공개 수용: 기록된 자료의 동작 근거이며 모든 미래 문장의 품질 보장 아님
