---
schema_version: 1
pair_id: source-knowledge-scan-validation
topic_slug: source-knowledge-scan-validation
language: ko
counterpart: ../en/source-knowledge-scan-validation.md
title: "검토 지식 스캔 검증 정합성"
summary: "candidate·apply 공통 credential 검증과 사람용 review ID 오인 방지"
tags: [knowledge, scan, source, v0-9-4, validation]
aliases: ["검토 source 가져오기", "스캔 검증 정합성"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:a9ee28808bdeb2119ecd0b906638d4d1481348c6ef039ef06b6e1d5a84b30bf9"
  - "repo:crates/hive-wiki/src/store.rs#sha256:f9f3dcf6627b0c1a2e08a05596f2e027c9a75c9fbbcafb48fbe8d29c000e1481"
links: [knowledge-cross-project-access, knowledge-portability-scan, source-development]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# 검토 지식 스캔 검증 정합성

`hive knowledge scan --candidates`와 `--apply`의 검토 claim credential 검증 공통화. registry·index
mutation 전 거부, 오류에는 raw source 대신 reviewed claim ID와 statement field 표시.

canonical scan provenance의 사람용 요약에서 review ID 제거, typed metadata 유지. 일반 설명형 ID의
opaque credential 오인 방지. source claim: project-private 유지, explicit collection 조회 필요.
