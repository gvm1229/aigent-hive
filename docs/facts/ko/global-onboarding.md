---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "선택형 bootstrap의 global/project scope 보존, 한국어 exact term·비배타 사용자 맥락."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:072bfc2c939e2a2e2e26f897b4cca9a876bd9d4be28adc8db14bafe7e5bb941b"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:35f3cace4b6297a298b8b59db208b3d8ecfd82331758fb6bd34dd1ec03aa8ec7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:12fbe0128457b6c9d0a4f32744eb3eb678c715129bb04bfc64d6f8cef5c073bc"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:a14989780a0783c98c953418c01f242eca5fe97254d6fbc01508f6d4ca153ef3"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:d211300dea66781251306e376e43bf9e798504ef"
status: active
---

# Global onboarding

수동 첫 설정 순서: CLI 설치, host activation, global setup, 명시 project setup. 선택형 one-prompt 경로:
project inspection 없는 global setup 시작.

지원 legacy 복구: saved preference·live file evidence 일치 조건. 그 외 active byte 보존. 한국어 setup:
`Skill`·`Wiki` exact 유지와 canonical regression sample 관리.

Global profile: 비배타 사용자 맥락 전용. Project workflow·기술 선택·작업 우선순위: project scope 전용.
