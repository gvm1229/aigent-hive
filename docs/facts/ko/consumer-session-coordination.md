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
  - "repo:crates/hive-cli/src/session.rs#sha256:174a8786fb00816745e2526eb91746a12558ddec4634151b314d1c305c009372"
  - "repo:docs/decisions/product-release-decisions.md#sha256:25bd2880270b2dd21bf09d5efe576f4164b8d02fadd8366f8649d8d50d38bded"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:5b4bf1a8b5815856e7bfb549c90eb279eb39db25bfda4a385c72d04244babfc2"
  - "repo:harness/directives/03-session-coordination.md#sha256:06736ffa5a1619bc238a39ecad068ae85b2136a8e33ccb0829badf932d8c19f3"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:5a4e0367ff8d52dd58221f15e16aa16a1ddee89fcc6cfef3fabe0c47f0e1babb"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:b13c85c9c9b7d4ad9980e3bd4b0299d2382a08bc0d8fd682e381d3c2ab87eb9d"
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
