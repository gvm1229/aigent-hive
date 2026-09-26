---
schema_version: 1
pair_id: native-hook-shell-boundary
topic_slug: native-hook-shell-boundary
language: ko
counterpart: ../en/native-hook-shell-boundary.md
title: "파일 훅과 셸 쓰기의 검사 경계"
summary: "Codex 파일 훅의 셸 쓰기 보호 미지원 확인"
tags: [acceptance, codex, hooks]
aliases: []
sources:
  - "repo:tests/results/runs/20260920T084743-7e741284a754.md#sha256:825cc29760132bc4c76286242ba585eb312342e6abafc42a0d834f16940ebef2"
links: [codex-file-hook-acceptance]
reviewed_revision: "git:47d631d8cbe1f21186225d5760a2854f75ec1239"
status: active
---

# 파일 훅과 셸 쓰기의 검사 경계

승인된 격리 시험에서 Windows Codex `0.155.0-alpha.9.2`의 셸 명령으로 합성 보호 파일 변경 성공. 종료 0과 바이트 변경 독립 확인 후 원본 복원. 등록된 `apply_patch` 선택식의 셸 경로 보호 미지원 근거이며, 보호 성공·다른 호스트의 증명 제외. 남은 호스트 수용 완료 요청에 따른 실제 적용 범위 확인.
