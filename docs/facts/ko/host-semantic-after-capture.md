---
schema_version: 1
pair_id: host-semantic-after-capture
topic_slug: host-semantic-after-capture
language: ko
counterpart: ../en/host-semantic-after-capture.md
title: "저장 뒤 호스트 관계 분석"
summary: "저장 성공을 보존하는 제한된 호스트 검토와 파생 관계의 원자적 적용."
tags: [graph, knowledge, provenance]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge/semantic_graph.rs#sha256:1005e13b2b4b354a8846e3b6d080ba6e738b87e4cb54a88f2e2b8bb5316dd0bd"
  - "repo:crates/hive-wiki/src/semantic_graph.rs#sha256:211c9a8cee0dd8217be535dcdc4c355d4fd50442693f90fd22945674b153adad"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:40d08d401f60962d70f8bb4fcb83b4f17b3b067d165e24db9f579cf364b63b2d"
  - "repo:tests/results/semantic-host-0.11.2.json#sha256:b8e9e8f9e987993f82ed584c2dea46e6b8d70ca0ab51f86b55b48b435ccdaaac"
links: [knowledge-storage, source-graph-public-qualification]
reviewed_revision: "git:52e63238cb6063312240d1c9b5ed1006d8a23a7b"
status: active
---

# 저장 뒤 호스트 관계 분석

- 정본 저장 완료 뒤 활성 범위의 변경 대기, 같은 내용 재저장의 새 분석 제외
- 현재 호스트에 변경 문서 최대 10개·권한 안의 관련 본문 합계 16KiB 전달
- 범위·원문 지문·UTF-8 바이트 근거 위치·결과 확인 기록 검증 뒤 원자적 적용, 잘못된 결과 교정 1회
- 관계의 정본 승격·제공자 API·상시 실행기·재귀 저장 제외
- Source Wiki·공유·프로젝트 비공개·기밀 분리. 삭제 관계 숨김과 분석 없는 빈 결과 정리
- Windows CLI·현재 Codex의 두 문서 검토 통과. 새 대화 스킬 발견·Antigravity 전달·독립 호스트 증명 미검증
