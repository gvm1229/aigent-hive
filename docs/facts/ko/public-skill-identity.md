---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Skill identity"
summary: "Aigent Hive의 product-only 22개 Skill, source 개발의 제품 Skill 재사용, 폐기 source ID의 안전한 이관 계약."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dea6123b7b193eb760a37b198566f9318d868fd7035491ac10756de0d4315530"
  - "repo:docs/plans/PLAN.md#sha256:fdad19dd0058d2768a022746e11fd5e85d783b5c3de309cad336d09188e5bfb1"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:5ae5f5e9e3ac9f2d9891393a75820fd1adbe8293e467a452d9778cba7fcb0468"
  - "repo:docs/skills.md#sha256:3ac35c43bee2bd83980415464b852253f271e95794d17fa81074fb2db0f88ec7"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:daa4ab56b05f7403bc1f5f2b44d8471fb99866af"
status: active
---

# Skill identity

Aigent Hive Skill 정본: product-only 22개. Source 개발: 설치된 product Skill과 저장소 directive
사용. 최종 tracked source Skill: `0건`. Historical `hive-loop-engineering`: product `ralph-loop`로
이관. Product `ship`: 각 저장소의 Git 규칙 탐색. `amend-directive`: compiled security·integrity
경계를 약화하지 않는 범위에서 사용자가 소유한 behavior 변경.

폐기 ID route: one-to-one·merge·split·no-Skill 지원. Source Wiki 작업: 세 product knowledge
Skill과 `hive source-wiki` 사용. Code·Git의 read-only 검토: host 기본 저장소 도구 사용.
Historical release byte: 변경 금지. 검증 불가 old path: write 없는 conflict.
