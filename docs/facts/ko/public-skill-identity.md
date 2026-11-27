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
  - "repo:crates/hive-projection/src/lib.rs#sha256:231667ff59ebdf77e225a02c36e12a911a4aa4ce23eb5290d94f7f560306b662"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:c8a9253f37738a5031244de86ea5fc53ae2814ea9b76e1f4eef081741f26401e"
  - "repo:harness/skills/catalog.yml#sha256:6e817b733d042b45e8f1700d948c91fb58d75bb619e1c47ab381b9bd446ac652"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Skill identity

Aigent Hive 제품 Skill 정본: 31개. 옛 ID의 직접 이관 별칭 유지.
`knowledge-capture`: 대화 기록과 새 지식 추출. `knowledge-manage`: 기존 지식 정비·공유·이전.
`humanize-kor`: 보존 검사를 거치는 명시적 한국어 윤문.
지식 Skill 표시 이름은 한국어 기능, 설명 첫머리는 정본 ID. 과거 배포 대장 바이트 보존.
