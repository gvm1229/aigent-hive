---
schema_version: 1
pair_id: agent-autonomous-continuation
topic_slug: agent-autonomous-continuation
language: ko
counterpart: ../en/agent-autonomous-continuation.md
title: "Agent 자율 실행 지속"
summary: "사용량 차단 우선, 정상 사용량에서 독립 작업 지속과 안정판 명시 승인"
tags: [agent, completion, regression]
aliases: ["성급한 작업 종료 방지"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:54f993c4403e7de3612efa12f499bbf1c3a957ee5893875cb0c50204505b1f40"
  - "repo:AGENTS.md#sha256:8b785024ed26e2d916e39dc4b96c90c64171c8eb28f671c4c7f8af1de77d0a68"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:cd9973f27fdd4952ee1936257cb4ed0f2f98d6b8033aa1f89fc2b93eefeb34fd"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a7caad0d660d83378b297ec3678312fa741b411b8115ed9eca98400d1db703d0"
  - "repo:crates/hive-render/src/lib.rs#sha256:0f57f976119a94d844faa5fbcab16eae37ed838041ffc9d089ebeb5b8a36ae3f"
  - "repo:harness/directives/00-project-harness.md#sha256:5d4e8b3c34a968b86735f59d449b194021c16ff8e9b4741b4c3ccaf130aa94b6"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:f901c85a4769dba3021471341fe9a14c0daba392661212bd233d852bfed80304"
  - "repo:harness/template/AGENTS.md.jinja#sha256:02cfaea5d05fc9e51b5a368cc70290f6f2b041912433bf640ad2ccaa0f6b2c39"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:5a20040c568570b7501cf6715838becb35f24adea960c4f63bd6e245b358ef13"
links: [automated-user-handoff, source-development, steering-preservation]
reviewed_revision: "git:f6bec784d94c76ef79cda8ff28e60a0b05765805"
status: active
---

# Agent 자율 실행 지속

- source·소비자 Agent 공통 지속 범위: 독립 조사·수정·검증·commit·허용된 push·CI 관찰·승인된 게시 작업
- 일부 host·fixture·외부 증거 결손: 해당 criterion 기록과 독립 작업 지속
- 전체 Goal·task `blocked`: 독립 Agent 소유 criterion `0건` closure 뒤 가능
- 안정판 tag·protected branch 통합·게시·설치: 현재 요청 안 버전명 포함 명시 승인 전 금지

사용량 차단은 작업 지속·종료 정리보다 우선. 소진 확인 뒤 짧은 중단 안내만 허용, 체크포인트·지식 저장·검사·Git·정리·인계 도구는 새 요청의 정상 사용량 확인 뒤로 연기.
