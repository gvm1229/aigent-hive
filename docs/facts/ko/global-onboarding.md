---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "선택형 one-prompt bootstrap과 번호 설정이 global/project scope·안전한 user projection 갱신을 보존."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:072bfc2c939e2a2e2e26f897b4cca9a876bd9d4be28adc8db14bafe7e5bb941b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:b51f51d18455515f964727fad6efb6bb2181826f2f30794ae2bbc27c7c48207d"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:183d58c69302e82b0cc911fab51a75137dad54439c3de4a46e38de13a13adf7b"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:005fa7c6aa9f7f43e5edfde31dffc146f1efe25b"
status: active
---

# Global onboarding

수동 첫 설정 순서: CLI 설치, terminal host activation, global user-scope setup, 명시 project setup.
선택형 one-prompt 경로: exact stable 또는 test release 선택, 필요 시 Node.js/npm 확인, 감지 host 한 개
활성화, project inspection 없는 global setup 시작. 초기 설정은 interface language 질문 우선,
재설정은 부분 변경 또는 전체 검토 선택 우선.

User projection refresh: release base·local bytes·incoming bytes 비교. Vanilla base는 exact 교체,
disjoint local 변경은 merge, overlap 변경은 local text 보존·omitted incoming hunk 보고. 인증 가능한
base 부재 시 write 전 중단.
