---
schema_version: 1
pair_id: skill-routing
topic_slug: skill-routing
language: ko
counterpart: ../en/skill-routing.md
title: "스킬 선택과 승인"
summary: "간단한 질문과 분리한 좁은 스킬 선택, 사용자 승인의 별도 적용."
tags: [consent, routing, skill]
aliases: ["Approved Skill routing"]
sources:
  - "repo:docs/architecture/skill-consent.md#sha256:062425d9110c2c52abf9f6b61d06c110f288f415b86706eecbf11439d8ac1c37"
  - "repo:docs/research/project-skill-user-acceptance-0.11.1.md#sha256:015374d4bbb3920fca12c9a698ebbe7ee9fbf0175bcf8da9fc0c3332269562a9"
links: [orchestration-ownership, project-onboarding]
reviewed_revision: "git:62f70999f0bf83f7f9eacfb5add2cdb47a0b5716"
status: active
---

# 스킬 선택과 승인

간단한 질문과 분리한 뒤, 좁은 요청 범위에 맞는 승인 스킬 최대 한 개 선택. 선택적 외부·생성 스킬은 명시적 미리보기와 사용자 승인 후 활성화.

사용자가 제공한 새 Windows·Codex 대화 두 개에서 프로젝트 ship의 명시·자연어 선택과 기존 검사·새 커밋 분리 규칙 활용 확인. 요청된 출시 수용의 읽기 전용 근거, 실제 검사·Unity·커밋 실행이나 모든 미래 요청의 성공 증명과 구분.
