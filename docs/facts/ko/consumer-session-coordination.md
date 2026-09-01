---
schema_version: 1
pair_id: consumer-session-coordination
topic_slug: consumer-session-coordination
language: ko
counterpart: ../en/consumer-session-coordination.md
title: "소비자 세션 조정"
summary: "직접 사용자 편집 통제 주장 없이 작은 경로 점유로 소비자 프로젝트의 겹치는 자동 편집을 조정하고 `0.10.0`에서 host-owned 프로젝트 Skill 예약을 추가하는 Hive 원칙"
tags: [consumer-harness, preservation, session, upgrade]
aliases: ["CHS93"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:17b4e24061b7214faa292fa50e65e9b0d9902270bdbe86fdc06ae53b7970bf05"
  - "repo:crates/hive-cli/src/session.rs#sha256:affa286cb1b1d23c2de042061af7092a89a137f1a1a5fa5762cc92bd5897e7af"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:1967454efd3e2e815003b3731ab5a328b0008c8bab746eb05f2f1670ecd7f5a1"
  - "repo:harness/directives/03-session-coordination.md#sha256:06736ffa5a1619bc238a39ecad068ae85b2136a8e33ccb0829badf932d8c19f3"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:f17a658f423c8df0f5ca2b1960c3ea53fec57cb2859459bfe77a049510e9adf2"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:3315a1ca6957fbd5dd34a8d1323f1d385226facf530ad83e10170a2938c17ad1"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:a0f288b6b962cd5bede27065fa39f708764a621f"
status: active
---

# 소비자 세션 조정

`hive session begin|check|update|close|recover`: `.hive/runtime/active-sessions/` 아래 Git 제외
일시 경로 점유. 활성 Hive 세션 사이의 상위·하위·동일 경로 충돌은 거부. 직접 사용자·외부 편집기 쓰기는
Hive 통제 범위 밖. 프로젝트 갱신은 직접 모순되는 Hive-owned directive clause만 미리 보기·적용하며,
사용자 작성·foreign·비충돌 local byte를 보존.

`0.10.0` 범위: host-owned 프로젝트 Skill 예약 계약. Codex·Antigravity:
`.agents/skills/<safe-skill>/...`만 허용. Claude: `.claude/skills/<safe-skill>/...`만 허용.
호스트 불일치: `hive.session-host-owned-namespace`의 명시 결과. 세션 해결 안내: live 또는
unverifiable reservation 한정.
