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
  - "repo:.agents/directives/01-behavior.md#sha256:2418d9cad5ad54ff9fdad0f117c66336826bbd34c19fc0c131340fe64cb31f01"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b70fedb14754fcb1d4c6d570381f845e5ae1be923d49f6ac929356add0e1777f"
  - "repo:docs/guidance-schema.md#sha256:fd8fffda818038ee48b66b0581787e2fd741404b0a9253ca34c0d55f15ad4d15"
  - "repo:harness/template/AGENTS.md.jinja#sha256:64f33fed294900badc58d8ff6b4f7144d0c43bf003884abdcae5c703a60cdd7a"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# 쉬운 설명 기본값

첫 순서: 사용자가 보거나 조작할 파일, 설정·지식 영향, 안전한 다음 행동.
`projection`, `manifest`, `digest`: 진단상 필요할 때만 같은 문장의 쉬운 말 정의와 함께 제시.
사용자 대면 목록: Markdown 항목당 한 줄. 독립 선택지의 comma-separated paragraph 금지.
