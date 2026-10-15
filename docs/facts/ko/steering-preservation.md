---
schema_version: 1
pair_id: steering-preservation
topic_slug: steering-preservation
language: ko
counterpart: ../en/steering-preservation.md
title: "추가 지시의 이전 조건 보존"
summary: "Hive 0.12.1의 추가 지시 보존·충돌 확인 규칙 전달"
tags: [agent, directives]
aliases: []
sources:
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:e3f955e48e354aaac598a5237679689fb7c827366574d9330bc6dd4b661735ea"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7b8afc5527b300e665054ccd0f1fc6f34df02fee75e5e87f7198100cfd55caa9"
  - "repo:harness/directives/00-project-harness.md#sha256:ee4f9721824c4caad76c21b7d43e04f708637f3df96a095bf8058ef46b122ace"
links: [agent-autonomous-continuation]
reviewed_revision: "git:29e5241e0983e13d222703731533b43409327e6a"
status: active
---

# 추가 지시의 이전 조건 보존

- 사용자 요구: 이전 지시 유지, 명시적 교체 없는 충돌의 선택 질문
- 0.12.1 구현: 프로젝트 공통 지침과 영어·한국어 전역 지침 전달
- 새 메시지의 기본 추가, 명시적 교체 범위만 변경. 충돌 행동은 답변 전 보류, 독립 작업 지속
- 취소 즉시 존중, 요약 뒤 미해결 조건 보존
- Windows 검사: 위키 비활성 포함 세 호스트 투영 확인. 실제 모델 준수·모든 도구 차단 증명 제외
