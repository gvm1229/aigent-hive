---
schema_version: 1
pair_id: codex-file-hook-acceptance
topic_slug: codex-file-hook-acceptance
language: ko
counterpart: ../en/codex-file-hook-acceptance.md
title: "Codex 파일 훅 실제 수용의 확인 범위"
summary: "Windows Codex에서 형식 3의 정상·검사기 부재 파일 편집 동작 확인"
tags: [acceptance, codex, hooks]
aliases: []
sources:
  - "repo:tests/results/runs/20260920T030341-ee67c1f8f88f.md#sha256:dd85153bd78b4a6d6b8ceb9dbfed4038c21d60b3269129c2f6d076b4cf21dac2"
  - "repo:tests/results/runs/20260920T030607-c02090c781e6.md#sha256:9d18a93c6ebc187150fe692ca491818f257ede74107a47b6f1f80b986dfd53a6"
links: [native-hook-launcher-failure]
reviewed_revision: "git:2c8ed3149a2bcf31dd9068f4fc9ac0d87dca2ab7"
status: active
---

# Codex 파일 훅 실제 수용의 확인 범위

- 형식 3 신뢰 완료 뒤 Windows Codex `0.155.0-alpha.9.2`에서 실제 시험
- 정상 상태: 일반 편집 1건 허용·보호 편집 1건 거부
- 검사기 부재: 두 편집 모두 거부·파일 바이트 유지, 시험 뒤 실행 파일·자료 복원
- 동결한 `0.11.0` 검사기와 네 시도에 한정한 근거
- 다른 호스트·훅 미로드·호스트 시간 초과·다른 도구·실제 턴 취소의 증명 제외
