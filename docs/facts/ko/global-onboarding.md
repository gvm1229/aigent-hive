---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "선택형 one-prompt bootstrap과 번호 설정이 global/project scope·지원 legacy user projection 안전 복구를 보존."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:072bfc2c939e2a2e2e26f897b4cca9a876bd9d4be28adc8db14bafe7e5bb941b"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:35f3cace4b6297a298b8b59db208b3d8ecfd82331758fb6bd34dd1ec03aa8ec7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:12fbe0128457b6c9d0a4f32744eb3eb678c715129bb04bfc64d6f8cef5c073bc"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2f653de651b5a7b540efab9522e5808156cc527ac6b3cda20df4a3f943b66c07"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:d211300dea66781251306e376e43bf9e798504ef"
status: active
---

# Global onboarding

수동 첫 설정 순서: CLI 설치, terminal host activation, global user-scope setup, 명시 project setup.
선택형 one-prompt 경로: exact release 선택, Node.js/npm 확인, host activation, project inspection 없는
global setup 시작.

Schema-1 `0.7.0` 복구 조건: saved preference digest·legacy inventory·live file digest 일치. later
Codex metadata 추가와 schema-2 base 기록. Legacy local edit·unknown inventory: migration 차단과 active byte
보존. `0.9.0-test.3` host recovery: frozen `setup-hive` digest·current selected projection 조합 인증.
