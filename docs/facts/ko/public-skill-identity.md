---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Skill identity"
summary: "28개 제품 Skill과 직접 이름 이관, 지식 스캔·이전 역할 분리"
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:d1d5ba5f4976ce15f4d9a5e2e0a90c8d44365ca432ca1707757a26ba8c2b863a"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:2b05f1b8ff3ba18a6ec22307dbee2ce5eea6b8da21fbca9de898d56d889a3ee0"
  - "repo:harness/skills/catalog.yml#sha256:2598c30eb9870eba93ae14b5439c87c09d67c4a1b0cb2fabcdff8a40702871ba"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# Skill identity

Aigent Hive 제품 Skill 정본: 37개. 옛 ID의 직접 이관 별칭 유지.
`knowledge-scan`: 새 지식 추출. `knowledge-transfer`: 기존 지식 이전.
`humanize-kor`: 보존 검사를 거치는 명시적 한국어 윤문.
지식 Skill 표시 이름은 한국어 기능, 설명 첫머리는 정본 ID. 과거 배포 대장 바이트 보존.
