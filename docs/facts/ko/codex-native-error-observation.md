---
schema_version: 1
pair_id: codex-native-error-observation
topic_slug: codex-native-error-observation
language: ko
counterpart: ../en/codex-native-error-observation.md
title: "Codex 훅 오류의 실제 관측"
summary: "실행 표식으로 JSON 거부와 합성 훅 실패를 구분"
tags: [acceptance, codex, hooks]
aliases: []
sources:
  - "repo:tests/results/runs/20260920T190935-ea2a51ed1c04.md#sha256:71d74c6c4f1b4ef40415b51a1b8d092fb7440e34fec807245ead029518c09091"
links: [codex-file-hook-acceptance, powershell-hook-exit-status]
reviewed_revision: "git:9051144140b4b892721b7ce8a8300e35c6ba7932"
status: active
---

# Codex 훅 오류의 실제 관측

Windows Codex 0.155.0-alpha.9.2의 여섯 합성 사례에서 각각 실행·선택 표식 확인. JSON 거부는 편집 차단·바이트 불변. 자식 종료 1/2/3·제한 시간 초과 대기·손상 JSON 사례는 편집 진행. 바깥 셸의 종료 코드 변환과 별도 판정. 일반 파일의 추가 진단 훅 결과이며 제품 Hive 검사기 내부의 오류 주입 증명은 제외. 원래 정의와 여섯 기준 파일 복원 완료.
