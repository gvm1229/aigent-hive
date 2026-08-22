---
schema_version: 1
pair_id: model-routed-custom-subagents
topic_slug: model-routed-custom-subagents
language: ko
counterpart: ../en/model-routed-custom-subagents.md
title: "Model-routed custom subagent"
summary: "Native host 분담 기능과 별개로 exact runtime attestation을 요구하는 Hive 사용자 정의 agent activation 경계."
tags: [antigravity, claude, codex, model-routing, subagent, v0-10, v0-9]
aliases: ["Sol Advisor 기능 동등성", "Task별 model routing"]
sources:
  - "repo:crates/hive-cli/src/custom_agent_cli.rs#sha256:5726ce3e28f3198b267fc017cba94d53c4a8703efa74544e5499be7c9488d9dd"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:786da31401085e9445495aa37defe7cedf781bc8457211a6addd23016c0bf922"
  - "repo:crates/hive-core/src/native_workflow.rs#sha256:246f845d21fe73c070abdfa4ffa78d28e829d84b3da498dcc1530355a54a0900"
  - "repo:docs/archive/plans/foundations/model-routed-custom-subagents.md#sha256:9fe4b79c4f4e0be1706600e06b74ab93ee8bbce01e767a38790bbf8bdd21b251"
  - "repo:docs/research/host-work-delegation-2026-08-20.md#sha256:00e8c2821082ececec3cbef81538030fc9487a8ba0903f1ee1fb378d73aa6c74"
links: [judge-verification, orchestration-ownership, role-state, skill-routing]
reviewed_revision: "git:a06262284f558a9ba955c44167bdcc8577102c77"
status: active
---

# Model-routed custom subagent

`hive agent recommend`: 서명된 host model catalog·분리 attestation·trust root 필수.
위조 서명·exact mapping 누락·지원 밖 lifecycle은 activation 전 거부. Judge strict terminal
acceptance: 두 정책 모두 허용. Material-risk: `implicit`만 허용. 2026-08-20 조사 결과:
Codex·Claude·Antigravity native 분담 기능 확인, exact role·model·effort·definition digest를
결합한 외부 검증 가능 receipt 부재로 Hive 자동 activation 보류.
