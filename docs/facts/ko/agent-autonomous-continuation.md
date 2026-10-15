---
schema_version: 1
pair_id: agent-autonomous-continuation
topic_slug: agent-autonomous-continuation
language: ko
counterpart: ../en/agent-autonomous-continuation.md
title: "Agent 자율 실행 지속"
summary: "독립 Agent 소유 작업 잔존 상태: 전체 Goal·task 차단 금지와 안정판 명시 승인"
tags: [agent, completion, regression]
aliases: ["중간 종료 방지"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:352a41e8879f9993888a178725a2d2b0b8dbae5f8c1edf2cdeeb5e3045cbc2a9"
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:e3f955e48e354aaac598a5237679689fb7c827366574d9330bc6dd4b661735ea"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7b8afc5527b300e665054ccd0f1fc6f34df02fee75e5e87f7198100cfd55caa9"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:crates/hive-render/src/lib.rs#sha256:62174ca2ea76cf5c379e1638e3ef2c33cc332f3ae312f03a983711f9abe1a1d5"
  - "repo:harness/directives/00-project-harness.md#sha256:ee4f9721824c4caad76c21b7d43e04f708637f3df96a095bf8058ef46b122ace"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:461be3769de1d76f7dee1aefc4f2cae3b360a339f40b41ef13cff404f571b2dd"
links: [automated-user-handoff, source-development, steering-preservation]
reviewed_revision: "git:29e5241e0983e13d222703731533b43409327e6a"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지
