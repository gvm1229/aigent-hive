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
  - "repo:crates/hive-core/src/korean.rs#sha256:16037a43c32e9fd1c777c6f7aabb7fa7bcf0fb265086fe84fc0bc35a93f07bda"
  - "repo:docs/architecture/korean-language-core.md#sha256:2ae475f2f4c701f42fe28bff62cb37e60ef516526b2f147d1ec1544e2b32bfa4"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:aaf1355c1b691a83f047164caed5923bcc5a9769ffb44aecfe8b4c3d247af46c"
  - "repo:harness/language-packs/im-not-ai/2.3.2/manifest.json#sha256:50e8bec5fb4c7a479f9e0800f262d49c3e01258ba3c7b9066aab65ba3f7ca34e"
  - "repo:harness/skills/humanize-kor/SKILL.md#sha256:8805da50d3370fa953a1325a0a6c5294247ab037cb173ab09266c67d09aa659a"
  - "repo:scripts/qualify-korean-public-test.py#sha256:96fda477a4c490aa15ad704f2a5117cfa1d63c00252c282408134de6219f498d"
links: [language-consistency, public-skill-identity, v0-10-product-scope]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# `0.10.0` 한국어 언어 core

완성 한국어 text의 다섯 profile 검사와 활성 host의 국소 rewrite 최대 1회. 결정적 검증 기반
서법·수치·인용·링크·code·명령·경로·출처 보존, 실패 시 정확한 draft 선택. `humanize-kor`:
사용자 선택 text에 같은 계약 적용. 고정 `im-not-ai 2.3.2` 변환 pack의 preview·exact consent·
staging activation·rollback 지원, raw·floating upstream 설치 금지.
번호 공개 시험은 Windows x64·macOS arm64·Linux musl x64에 exact npm byte 설치. Gold corpus·
보존 거부·sanitize·update preview·rollback 재실행.
