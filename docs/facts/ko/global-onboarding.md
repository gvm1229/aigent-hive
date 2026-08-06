---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "Host activation·global preference·project setup 번호 순서 기반 명시 scope routing."
tags: [onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:5d76a2698ec20d359181c065e44105cf91264d943aaf748077971da14613173c"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:da6bf1991539dadff877be04330aaf96b61f2112bb0acf8d2f69cba2cdc2a692"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:1fa7abad6925bcf17c8b253458e024733e5de1f6"
status: active
---

# Global onboarding

첫 설정 순서: CLI 설치, terminal host activation, global user-scope setup, 명시 project setup.
Global prompt: ambient project inspection 없음. Project prompt: 이름 또는 absolute path의 repository만
write preview 뒤 처리. Known prior ownership snapshot은 update 가능, unknown·modified manifest는
preview·mutation 전 차단.
