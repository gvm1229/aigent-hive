---
schema_version: 1
pair_id: simple-explanation-default
topic_slug: simple-explanation-default
language: ko
counterpart: ../en/simple-explanation-default.md
title: "쉬운 설명 기본값"
summary: "소스·소비자 에이전트의 쉬운 말 기본 설명과 이해에 도움 되는 구체적 예시 사용, 기술적 정확성 보존."
tags: [communication, guidance, projection]
aliases: ["구체적 예시", "쉬운 말 설명"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:aff2586323a4db2acad51cd0225b9791e4d0a974cd1a2e96d92eeaafbacdf5d6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:282c641531d7a72009ee3973801ee8d20ea1305fd7e6f836f202e6a4e22508d4"
  - "repo:docs/guidance-schema.md#sha256:fd8fffda818038ee48b66b0581787e2fd741404b0a9253ca34c0d55f15ad4d15"
  - "repo:harness/template/AGENTS.md.jinja#sha256:64f33fed294900badc58d8ff6b4f7144d0c43bf003884abdcae5c703a60cdd7a"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:35e6b79a024350487f823780101a28be24a9f4c7"
status: active
---

# 쉬운 설명 기본값

소스 개발·설치된 소비자 에이전트의 기본 설명: 쉬운 말 사용. 이해에 실질적인 도움이
될 때만 구체적 예시 추가. 관련 없는 예시 강제와 기술적 정확성 약화 금지. 완료 기준:
모든 사용자 대면 목록: Markdown 항목당 한 줄. 독립 선택지는 comma-separated paragraph 금지.
소스·소비자 지침 생산자, 선택 언어별 사용자 지침, 직접 회귀 검사에 같은 제한 규칙
투영. 요청 배경: 이 설명 방식을 기본값으로 지정한 유지관리자 요청.
