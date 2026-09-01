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
  - "repo:crates/hive-projection/src/lib.rs#sha256:edbfa7b5ce8edc6b853262d6440c309817e6463509b58bf298af038c4c3219fb"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:b1d168024659e23bc1fee30c46e2b628e607522b9b0da2f59229a277eff2a702"
  - "repo:harness/skills/catalog.yml#sha256:5949525e029f37e08f5ef49302f698be45674b94959f4f5aa301d7138c4e1570"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# Skill identity

Aigent Hive 제품 Skill 정본: 28개. 옛 ID의 직접 이관 별칭 유지.
`knowledge-scan`: 새 지식 추출. `knowledge-transfer`: 기존 지식 이전.
`humanize-kor`: 보존 검사를 거치는 명시적 한국어 윤문.
지식 Skill 표시 이름은 한국어 기능, 설명 첫머리는 정본 ID. 과거 배포 대장 바이트 보존.
