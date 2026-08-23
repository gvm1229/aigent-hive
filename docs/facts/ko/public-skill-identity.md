---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Skill identity"
summary: "명시적 한국어 윤문 humanize-kor를 포함한 Aigent Hive product-only 27개 Skill과 안정된 영문 ID 계약"
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:b79d42a472aedc3cc05ce9d5439aebd5c99171798cf9e26faca1c17ac0f3558a"
  - "repo:docs/archive/plans/foundations/knowledge-skill-display-names-next-release.md#sha256:517f1f10a17537698d1e4e1a30b59bda9fd2488e3062576d01b4cf641dea0e76"
  - "repo:docs/skills.md#sha256:76e70020fd1492cf59530fc27e1c537dca4c59ddb57bb241f9710e7b667cf535"
  - "repo:harness/skills/catalog.yml#sha256:76ed4b4d220db932da8e0e63aee700875f460b72922715a56b83bae1b9065273"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# Skill identity

Aigent Hive product Skill 정본: 27개. 실행·설정 호환용 기존 영문 ID 유지. `humanize-kor`:
결정적 보존 gate 기반 명시적 한국어 윤문. 지식 Skill 표시 이름: 한국어 기능만 표시, 설명 첫머리:
정본 ID. Historical release inventory byte 변경 없음.
