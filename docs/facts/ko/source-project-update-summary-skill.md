---
schema_version: 1
pair_id: source-project-update-summary-skill
topic_slug: source-project-update-summary-skill
language: ko
counterpart: ../en/source-project-update-summary-skill.md
title: "Source 프로젝트 전용 업데이트 요약 Skill"
summary: "소스 전용 update-summary의 검증 기반 제품 홍보: 새 기능·개선 구분, 핵심 기술 이름과 사용 이점 강조, 개발자 전용 변경 제외"
tags: [development, release-notes, skill]
aliases: ["update-summary"]
sources:
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:38ded441028bdf57b22aba843eb87e1db6330c8ed8d434b2440759a601f1329b"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:f54809bb95a14604efa062d134ba8b9197cc578d"
status: active
---

# Source 프로젝트 전용 업데이트 요약 Skill

`update-summary`: 유지보수자 승인 소스 전용 Skill. 제품 목록·배포 묶음·소비자 투영 제외.
검증된 비교 버전 근거로 한국어 구독자 홍보 작성. 새 기능·개선·수정·이름 변경 구분.
제목에 핵심 기술과 이점, 하위 목록에 예시·선택 사항·비용 배치. 에이전트 검토 표현: `리뷰`.
개발자 전용 게시 형식·CI·기록·계획 제외. 미출시 대상은 검토용 초안, 기능 과장·발행 권한 확대 금지.
안정판 문구 정본: `docs/releases/<version>.subscriber.ko.md`.
