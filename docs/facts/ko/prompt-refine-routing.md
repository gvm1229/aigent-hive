---
schema_version: 1
pair_id: prompt-refine-routing
topic_slug: prompt-refine-routing
language: ko
counterpart: ../en/prompt-refine-routing.md
title: "Prompt refine 승인 routing"
summary: "명시적 프롬프트 작성과 일반 작업의 조사·계속 진행 경계 분리."
tags: [prompt, routing, skill]
aliases: ["Prompt approval gate"]
sources:
  - "repo:crates/hive-projection/src/lib.rs#sha256:c80df9705880a64e92a2af923ac394fccbdbd19385b4edd8fecfbf7f8c0dce67"
  - "repo:harness/skills/prompt-refine/SKILL.md#sha256:bbd9a76fed57e1276aa94266709d79f656eafebc98e58723c5f8286b979399ed"
links: [orchestration-ownership, skill-routing]
reviewed_revision: "git:c5155855b9fb416db1b6a50286175c5cea930238"
status: active
---

# 프롬프트 작성과 작업 실행 경계

명시적 프롬프트 작성 요청: `refine-only`. 결과 전달로 작성 작업 완료.
`awaiting-approval`은 이후 실행에만 적용. 작성 문자열의 로컬 지문 계산 허용,
프로젝트 검사·쓰기·기억 저장·실제 작업 실행 제외.
실행 권한: 같은 요청의 명시적 실행 지시 또는 정확한 후속 승인.

모호한 일반 작업: `RunWork`와 호스트 기본 경로 유지, 개선 제안만 선택적으로 표시.
승인된 조사 지속, 실질적 사용자 선택·새 권한만 질문.
`0.10.2` 지침 감사로 이전 자동 개선 선택 정책 대체. 프롬프트 분류 hook 추가 없음.
