---
schema_version: 1
pair_id: marketing-deck-record
topic_slug: marketing-deck-record
language: ko
counterpart: ../en/marketing-deck-record.md
title: "Marketing deck record"
summary: "External deck의 safe locator·resume 기준을 보존하는 tracked handoff."
tags: [artifact, marketing]
aliases: ["LumaDeck handoff"]
sources:
  - "repo:docs/state/artifacts/aigent-hive-marketing-deck.md#sha256:ffc5ef8f86100b42662a789e6f93878ab7f6b09f60bc1f6231b9306cdcd3e1ba"
links: [product-purpose, test-distribution, v0-9-skill-suite-plan]
reviewed_revision: "git:c949c754ccb602f10468ae30bb3e402e4e01f39d"
status: active
---

# Marketing deck record

External LumaDeck artifact는 `0.9.0-test.5` 기준 91장·60분 `aigent-hive-overview` 발표자료다.
공개 short Skill name 22개 각각의 설명·직후 예시, README 기반 설치 선택, 구현 원리와 저장소의
계획·ADR·prefix·workflow·verification convention을 다룬다. Embedded notes와 별도 발표 대본을
포함하며 production build와 1280×720 전체 91장 overflow 검증을 통과했다. Source corpus는
safe locator, 범위, 버전 기준, 검증 결과, exact resume condition만 보존한다.
