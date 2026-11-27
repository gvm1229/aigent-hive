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
  - "repo:crates/hive-projection/src/lib.rs#sha256:2ebda9014906433468b9b07a89627938f32aedfe08f56e0203539d8b9e3a3ad9"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:c8436b3c92a4b0e3b11ea57d4cb1ea9b9f410b518a83feff81a31685d1cc0559"
  - "repo:harness/skills/catalog.yml#sha256:c0b54d4bd885f7f40cca464be778c30d5c594d7ffa94b9c09952297b8d5b6f51"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Skill identity

Aigent Hive 제품 Skill 정본: 31개. 옛 ID의 직접 이관 별칭 유지.
`knowledge-capture`: 대화 기록과 새 지식 추출. `knowledge-manage`: 기존 지식 정비·공유·이전.
`humanize-kor`: 보존 검사를 거치는 명시적 한국어 윤문.
지식 Skill 표시 이름은 한국어 기능, 설명 첫머리는 정본 ID. 과거 배포 대장 바이트 보존.
