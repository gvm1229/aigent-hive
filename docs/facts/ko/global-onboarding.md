---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "선택형 bootstrap의 scope 보존, 한국어 exact term·사용자 맥락·all built-in 기본 활성화."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:6ddf3dd877c31e3f6e525ea6a659fdf90233cbf008cfc3be355f271267c9fa94"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2c61916f31b5a6ae66f6c2a615c41bcf4ac91ea2ca95d388f5d357cd5d872269"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:0292baa97d8ec193709ae756e56393af34085d781d7c341fe5d0d1ab0ed244e0"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2e064212050d755bf101322fdcc94f8a737db7b59204b75bb6bfcd64d8e32ceb"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:a30eb472f72773109da16706f4dbcb81cef76421"
status: active
---

# Global onboarding

수동 첫 설정 순서: CLI 설치, host activation, global setup, 명시 project setup. 선택형 one-prompt 경로:
project inspection 없는 global setup 시작.

지원 legacy 복구: saved preference·live file evidence 일치 조건. 그 외 active byte 보존. 한국어 setup:
`Skill`·`Wiki` exact 유지와 canonical regression sample 관리.

Global profile: 비배타 사용자 맥락 전용. Project workflow·기술 선택·작업 우선순위: project scope 전용.

Global setup: 모든 built-in Skill 기본 활성화. Profile-bound recommended suite 제거, Skill별 toggle.
Typed user config: `all|individual`만 허용. Saved legacy suite: approved preview로 새 형식 저장 전
recorded closure 유지.
