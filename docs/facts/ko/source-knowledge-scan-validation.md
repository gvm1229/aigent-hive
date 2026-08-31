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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:f0e47ded9439c9d2fcb2c1be6eb93d11609e942d5320f452fd45feecc7bf7d8a"
  - "repo:crates/hive-wiki/src/store.rs#sha256:6d6a377a6cd0c0c38ca48a85e89e871210ef4e87bbe05cf80c17713a566ae9a0"
links: [knowledge-cross-project-access, knowledge-portability-scan, source-development]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
status: active
---

# 검토 지식 스캔 검증 정합성

`hive knowledge scan --candidates`와 `--apply`의 검토 claim credential 검증 공통화. registry·index
mutation 전 거부, 오류에는 raw source 대신 reviewed claim ID와 statement field 표시.

canonical scan provenance의 사람용 요약에서 review ID 제거, typed metadata 유지. 일반 설명형 ID의
opaque credential 오인 방지. source claim: project-private 유지, explicit collection 조회 필요.
