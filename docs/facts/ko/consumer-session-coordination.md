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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:d5e36f5d1cb6080fa7952b1cf4354e7d54f0df12bc0799d758ced53d7f083b84"
  - "repo:crates/hive-cli/src/session.rs#sha256:a41fd95ca2576269d347b4635afa064dc9dc53a70d96434daef1eb819e0fbf19"
  - "repo:docs/decisions/product-release-decisions.md#sha256:9f234ef3fede8030ab6ad57fa4b560a71f30f468622c4e4ad56b83864d0e05ce"
  - "repo:docs/plans/active/release-0.10.0.md#sha256:2b8007e0cbf5a0f89ebb654ee7f6b44a1b203eee905205fe7ea90629941e4cad"
  - "repo:harness/directives/03-session-coordination.md#sha256:bdef62f4f837e3c0c84a794a9f4c7bca1b8947a92b5b3ea66ff8314bb2c7dfec"
  - "repo:harness/skills/project-setup/SKILL.md#sha256:aef70c66a054607e84288fa44f5fefca3c089515d41cb0c06934498ebbf31ca1"
  - "repo:tests/conformance/integration/test_project_lifecycle.py#sha256:de505a175585debcf7073da8c014a9ef7af5e363e162f29ddb9f256753695c69"
links: [knowledge-preservation, project-onboarding]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
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
