---
schema_version: 1
pair_id: install-wide-knowledge-capture
topic_slug: install-wide-knowledge-capture
language: ko
counterpart: ../en/install-wide-knowledge-capture.md
title: "설치 범위 지식 수집"
summary: "Hive 사용자 범위 수집·조회는 프로젝트 설정 없이 설치 직후 선택 호스트의 모든 프로젝트에 적용."
tags: [capture, knowledge, retrieval, user-root]
aliases: ["미등록 프로젝트 조회", "설정 독립 지식"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:32db20fc6252cd4a5dcf6fac16ff4e7da275a8c0ca41ddd93c3eb6f6b1f28f68"
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:b2dab4ddd8cc0336bccd28c1bd61ba66901a2984b5306040ef994cdf355b1e35"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:a0cf1b55b6cfc9d02920f117d25fb752e25267093d1be960b33267b8f6b0a94c"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:2bfa879fe9bc0d682e6562c3d323d2400daf80478140a3a1d1fc5aa7b09621ed"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7409f07c8378e2d0ff228f8f94305f81dbfcd08e379d7dbcd25e88c58c3a5582"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:3ddc5bf3e58c774266ce2bb11654fa54c80e0a09c519347b9de54d61eedaaa11"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:d80f5871428941be1d05b12d488857f40bb894752c0c4354cee12f15a46578f6"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:41d5e94d3a19590d65d4bd66cda74777509aa70a5debc128923f3806c7867edc"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 설치 범위 지식 수집

전역 위키 지침의 모든 선택 호스트 폴더 적용. 안전한 사용자 전역 지식 기록에 프로젝트 설정·하네스·표시 파일·연결 자료 모음 불필요. 미등록 조회의 비공개·기밀 제외. 에이전트 검토·정규화·제한된 전경 기록, 대화 원문 자동 수집 제외. 소진 확인 시 추가 도구 없이 기록·검사 연기, 새 요청의 정상 사용량 확인 뒤 재개.0.9.1 자동 기록·새 세션 회수 근거는 보존 문서 참조.
