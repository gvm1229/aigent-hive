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
  - "repo:.agents/directives/01-behavior.md#sha256:9d8adb7c75015fd24df8cb226a16180548c600dc963ee154c0a4af408e9fa48c"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2960ab32ce7831e993fee381e664d20eb5f673a62f006582a8e601476ccdb0fa"
  - "repo:docs/guidance-schema.md#sha256:f5fc6aa2c36274d78d9703693a362c2f8d8eb81204d37f8a224434c14d1b196b"
  - "repo:harness/template/AGENTS.md.jinja#sha256:d706dc6585c1bbaa820d328ebfaae919cd02496adac0acec373ee4d0e37afe56"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:536f5076534cedcdb9ea3d118830792fe61cd75e"
status: active
---

# 쉬운 설명 기본값

첫 순서: 사용자가 보거나 조작할 파일, 설정·지식 영향, 안전한 다음 행동.
`projection`, `manifest`, `digest`: 진단상 필요할 때만 같은 문장의 쉬운 말 정의와 함께 제시.
사용자 대면 목록: Markdown 항목당 한 줄. 독립 선택지의 comma-separated paragraph 금지.
