---
schema_version: 1
pair_id: skill-routing
topic_slug: skill-routing
language: ko
counterpart: ../en/skill-routing.md
title: "Skill Routing과 Consent"
summary: "좁은 intent routing, optional Skill approval과 안전한 source-consumer Skill 재사용."
tags: [consent, routing, skills]
aliases: ["Skill 라우팅"]
sources:
  - "repo:AGENTS.md#sha256:28626a77b614ca70cd09afdeb8be3d0767e5ca088ab52e942bf2af269d7b9cb2"
  - "repo:docs/architecture/skill-consent.md#sha256:062425d9110c2c52abf9f6b61d06c110f288f415b86706eecbf11439d8ac1c37"
  - "repo:harness/skills/catalog.yml#sha256:defd5f3ec25aace12c12bcb599f838e30820d56cc6b7e213bfac7abc55cabc4e"
links: [knowledge, plugin-lifecycle, workflow]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Skill Routing과 Consent

Narrow Skill description과 typed catalog entry를 통한 explicit task intent routing. Duplicate
model-based prompt classifier 추가 금지. Self-contained simple question은 project memory,
unrelated Skill과 orchestration을 load하지 않는 direct-answer path 유지.

Optional third-party 또는 generated Skill approval payload: name, immutable source, revision,
content digest, requested capability, approved capability와 approval time 결합. Identity 또는
capability 변경 시 fresh approval 필요. Invalid consent 상태의 Skill은 inert 유지.

Hive-owned Skill의 consumer-source 이동 조건: scope, safety, consent와 conformance review.
Shared canonical source 위치: `harness/skills/`. Exact source projection 가능 위치:
`.agents/skills/`. Installed consumer state, user knowledge와 runtime data의 source material 사용
금지.
