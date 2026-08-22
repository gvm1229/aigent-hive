---
schema_version: 1
pair_id: simple-explanation-default
topic_slug: simple-explanation-default
language: ko
counterpart: ../en/simple-explanation-default.md
title: "쉬운 설명 기본값"
summary: "소스 에이전트의 사용자 대면 파일·데이터 영향·안전한 다음 행동 우선 설명, 내부 구현 용어의 뒤이은 쉬운 말 정의."
tags: [communication, guidance, projection]
aliases: ["구체적 예시", "쉬운 말 설명"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:dd66d053a9edd60c2f04e96283f4f95e5429dbf24e6b2d98c025bbf89039d5df"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6ee84b034e6a23171889dc33e8be8f839594edd080168ad7601e8c5fd9e5c9cc"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:536f5076534cedcdb9ea3d118830792fe61cd75e"
status: active
---

# 쉬운 설명 기본값

첫 순서: 사용자가 보거나 조작할 파일, 설정·지식 영향, 안전한 다음 행동.
`projection`, `manifest`, `digest`: 진단상 필요할 때만 같은 문장의 쉬운 말 정의와 함께 제시.
사용자 대면 목록: Markdown 항목당 한 줄. 독립 선택지의 comma-separated paragraph 금지.
