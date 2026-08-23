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
  - "repo:.agents/directives/01-behavior.md#sha256:20905d49494df815461b4e9ffe6df89ee33ccb774510da2cfa10c98f0508b077"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:359e033f6bad6a6145820efb0a079a6643d4774a6d9b8e1b560d9d4e156df5be"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:f1170037b949896332fdb95f058fde810a00b0474b423e054899a74a5da3b200"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:64125db02505a9a696e870d23fa54feb125b8093"
status: active
---

# 쉬운 설명 기본값

첫 순서: 사용자가 보거나 조작할 파일, 설정·지식 영향, 안전한 다음 행동.
`projection`, `manifest`, `digest`: 진단상 필요할 때만 같은 문장의 쉬운 말 정의와 함께 제시.
사용자 대면 목록: Markdown 항목당 한 줄. 독립 선택지의 comma-separated paragraph 금지.
