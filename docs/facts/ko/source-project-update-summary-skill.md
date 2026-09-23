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
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:ca4bbfd9f46d7e92b9bdd3bc388935236c69e07730689ee8735ad068b0c56f0e"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
  - "repo:docs/releases/0.10.0.subscriber.ko.md#sha256:ce658d7a5addabc93d69c99d3bea80fd0137c61d3141c9880c05fa1e50d4e426"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:3a0d9e2e61d1867e0f38d8855ae8b064fa449f09"
status: active
---

# Source 프로젝트 전용 업데이트 요약 Skill

`update-summary`: 소스 전용, 제품 목록·배포·소비자 투영 제외. 검증된 새 기능·개선·수정·이름 변경과
핵심 기술·이점·하위 예시·비용 강조. 미출시 내용은 초안, 문구 승인과 출시 권한 분리.
필수 예시: 2026-09-01 승인 0.10.0 안내. 문구 승인 뒤 기존 `gh` 인증과
`register-stable-summary-approval.py`로 외부 지문 자동 등록. 출시마다 GitHub 수동 설정 불필요.
재시도는 동일 승인, 문구 변경은 새 승인 필요. 발송 전 원문·sidecar·외부 지문 대조, 값 자동 갱신 금지.

Hive 내부를 모르는 독자 기준. 제목·첫 설명은 사용 경험의 이점, 검사 근거는 별도 기록.
