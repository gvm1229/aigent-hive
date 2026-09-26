---
schema_version: 1
pair_id: codex-native-cancellation
topic_slug: codex-native-cancellation
language: ko
counterpart: ../en/codex-native-cancellation.md
title: "Codex 수동 중지 검증 범위"
summary: "Windows Codex의 수동 중지와 같은 작업 후속 실행 확인"
tags: [acceptance, cancellation, codex]
aliases: []
sources:
  - "repo:docs/research/codex-native-cancellation-2026-09-20.md#sha256:e227bccdcf6da8693275ab7a6c3f074404c0c94e3a20c06c12895eb5eb8bc49d"
links: [codex-file-hook-acceptance]
reviewed_revision: "git:a42459c1d8b6701aefcd7ad527c430184ce9949f"
status: active
---

# Codex 수동 중지 검증 범위

Windows Codex `0.155.0-alpha.9.2`에서 사용자 중지 버튼 뒤 `interrupted` 확인. 같은 작업의 후속 실행은 `completed`, 원래 실행은 중단 상태 유지. 호스트 상태 조회로 수동 취소·이어 가기 검증. Hive 자동 중단·15초 감시·Stop 훅 실행·다른 호스트의 증명 제외. 재개 응답은 시각 조회를 보고했지만 조회 API에 해당 도구 항목은 미노출.
