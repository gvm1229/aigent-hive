---
schema_version: 1
pair_id: consumer-session-coordination
topic_slug: consumer-session-coordination
language: ko
counterpart: ../en/consumer-session-coordination.md
title: "소비자 세션 조정"
summary: "직접 사용자 편집 통제 주장 없이 작은 경로 점유로 소비자 프로젝트의 겹치는 자동 편집을 조정하는 Hive 원칙"
tags: [consumer-harness, preservation, session, upgrade]
aliases: ["CHS93"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:4213e512a4d14323b92f0fef7f4ae77055c441c064b84da17f9b883dbb38ff3c"
  - "repo:crates/hive-cli/src/session.rs#sha256:1a6fd68e66a00f5f3343b801479f564f4667a7123793b43f9d2d2c94648f0b9d"
  - "repo:harness/directives/03-session-coordination.md#sha256:06736ffa5a1619bc238a39ecad068ae85b2136a8e33ccb0829badf932d8c19f3"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:376242cfffb02880c994804d608a176bfbdef3e57d2f40493a86cdf5405f798e"
  - "repo:tests/conformance/test_project_lifecycle.py#sha256:7862526b5478758639c55e6ac966e0e24e1f7d9b6d5ef26e99109a33c7abca49"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:a52362971c8fa646b428449dd85681491eaeb184"
status: active
---

# 소비자 세션 조정

`hive session begin|check|update|close|recover`: `.hive/runtime/active-sessions/` 아래 Git 제외
일시 경로 점유. 활성 Hive 세션 사이의 상위·하위·동일 경로 충돌은 거부. 직접 사용자·외부 편집기 쓰기는
Hive 통제 범위 밖. 프로젝트 갱신은 직접 모순되는 Hive-owned directive clause만 미리 보기·적용하며,
사용자 작성·foreign·비충돌 local byte를 보존.
