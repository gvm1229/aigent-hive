---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "전역 설정: 질문 전 signed CLI 확인, 답변별 진행 상태 보존, 전역 사용량 보호 한도 관리. 프로젝트는 더 이른 중지만 선택 가능."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:67c09e54e76df72ee9ac6acbde5b88fbb0a6653e1d7172e3f789a8d99c2434b7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:48a76cd5503858a2327c7562879de259334b182687aace98ec1df06b71dd1600"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:87db5fb3f07e5a346d0060eee545bcd22135963c850afbf0e1fd737ba243b1d1"
  - "repo:docs/plans/active/windows-global-setup-hardening.md#sha256:6742a157cb9665b6b3accffce536dd447642ec367c41fa06c7d5bc7ef6ca0910"
  - "repo:harness/skills/configure/SKILL.md#sha256:abeb032e21d2576366025465d54080966767fb7e17cca57848acf093eaa83eaf"
  - "repo:harness/user-setup/catalog.yml#sha256:7dc82dbf559075ce4286e7dd19aec0ddc22e04f35ad4a8a60f43129a4dba2a1f"
  - "repo:schemas/user-setup.schema.json#sha256:46b360a9f91e154d1440e2997b56a964edd122383ccfc9b105b4e2ae4f8939f9"
links: [project-onboarding, test-distribution, usage-guard-thresholds]
reviewed_revision: "git:a5dd671385c2a1e09d511fb1de6c737261210df7"
status: active
---

# Global onboarding

순서: CLI 설치, host 활성화, global setup, 명시 project setup. v0.9 전역 Wiki 정본은 local
Markdown이며 global setup은 project를 검사하지 않음.

사용량 보호: 전역 설정의 기본 안전 한도. 등록 project는 남은 사용량의 더 높은 한도만 선택 가능.
따라서 project별 조기 중지는 가능하지만 전역 보호 약화는 불가.

Windows 11 test.5 감사에서 npm CLI 탐색 실패, schema 추측, 임시 답안 17개 이상, 진행 손실,
apply 실패 확인. 수정 조건: 질문 전 CLI·signed setup 설명 확인, 답변마다 진행 저장, OS 임시 파일
하나와 cleanup, stable 0.9.0 전 fresh Windows numbered-test 통과. 인증 불가·사용자 수정 byte는 보존.
