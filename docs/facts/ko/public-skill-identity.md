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
  - "repo:crates/hive-projection/src/lib.rs#sha256:4d4d64b5b580a455b365a1039de568b39ff395072dc0d653e8ea495238a73fe4"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:c0faff5fbecfd6cde693ccec60ea07412eac4502c4cb2d5d6e3ced1bf83ee746"
  - "repo:harness/skills/catalog.yml#sha256:2598c30eb9870eba93ae14b5439c87c09d67c4a1b0cb2fabcdff8a40702871ba"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Skill identity

Aigent Hive 제품 Skill 정본: 37개. 옛 ID의 직접 이관 별칭 유지.
`knowledge-scan`: 새 지식 추출. `knowledge-transfer`: 기존 지식 이전.
`humanize-kor`: 보존 검사를 거치는 명시적 한국어 윤문.
지식 Skill 표시 이름은 한국어 기능, 설명 첫머리는 정본 ID. 과거 배포 대장 바이트 보존.
