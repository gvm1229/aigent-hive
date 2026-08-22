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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:af09aadf2ddfabc082dfac9ae6c8233c2fe48f964db8996063848838f04f68c5"
  - "repo:crates/hive-cli/src/session.rs#sha256:1a6fd68e66a00f5f3343b801479f564f4667a7123793b43f9d2d2c94648f0b9d"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3fbe246c3a5b7d2b8ec002d40f73874c056c48ae3a888dede3e40db12eddddac"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:75ece4a12c890f3950d876c96ec605a2a80ebeecfc2ed7255ff3797cc2a33c2e"
  - "repo:harness/directives/03-session-coordination.md#sha256:06736ffa5a1619bc238a39ecad068ae85b2136a8e33ccb0829badf932d8c19f3"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:5a4e0367ff8d52dd58221f15e16aa16a1ddee89fcc6cfef3fabe0c47f0e1babb"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:c066a315884e9c93499da3db8dad0b0abf41279c89bbb467e1a6deb6b2a0842f"
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
