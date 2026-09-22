---
schema_version: 1
pair_id: simple-explanation-default
topic_slug: simple-explanation-default
language: ko
counterpart: ../en/simple-explanation-default.md
title: "쉬운 설명 기본값"
summary: "소스 에이전트와 설치 사용자 지침의 대화·설명형 글: 다섯 살 이해 수준, 핵심 기술 이름·정확성·한계 보존"
tags: [communication, guidance, projection]
aliases: ["구체적 예시", "쉬운 말 설명"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:7679dd5603fdfa1104b0017e9fe7c7acb6a81f9e4095233ccddf3db01a325af8"
  - "repo:.agents/directives/08-human-documentation-style.md#sha256:a4f9c9d280a596786fb93cd0ee71bc7b5987f3bafe3be99b1184997f7af6465f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:4afd5ba4483d98f63ae42c065837f9b010506657"
status: active
---

# 쉬운 설명 기본값

첫 순서: 사용자가 보거나 조작할 파일, 설정·지식 영향, 안전한 다음 행동.
소스 에이전트와 설치 사용자 지침의 대화·가이드·블로그·보고서: 배경지식 없이 이해할 쉬운 말, 짧은 문장, 목적 뒤 작동 이유와 과정.
`digest` 같은 핵심 용어는 첫 등장에 풀이. 필요한 비유·예시·단계·비교 활용, 아기 말투·분량 강제 금지.
수치·명령·조건·불확실성과 검증 한계 보존, 전송·저장 전 이해 가능성 확인.
정본 `01-behavior`, 문서 `08`에서 참조. 목록 항목별 한 줄, 독립 선택지 분리.
