---
schema_version: 1
pair_id: windows-process-observation
topic_slug: windows-process-observation
language: ko
counterpart: ../en/windows-process-observation.md
title: "Windows 종료 중 관측 경계"
summary: "같은 프로그램의 실제 종료가 확인된 경우만 조회 실패 건너뛰기"
tags: [acceptance, process, windows]
aliases: []
sources:
  - "repo:scripts/qualify-vector-runtime.py#sha256:2d9160ac0ba4cc751a84f9854e78f59d50c2aa61dc01bb0948a0147e2d8b3179"
  - "repo:tests/results/runs/20260920T205130-e7a95052dc21.md#sha256:7d30c302522f60c5def30463e4115ad9f3b0c9f5652ae82c86f726420b398291"
links: [source-development]
reviewed_revision: "git:6296726906d4b24f82243c4fafae43dbf8f3245f"
status: active
---

# Windows 종료 중 관측 경계

0.11.0 공개 수용 마무리 중 Windows 프로그램 종료 직전의 파일 정보 조회 오류 재현. 시험 도구는 같은 프로그램을 가리키는 운영체제 참조인 핸들에서 최대 100ms 대기 후 확인된 종료만 건너뛰기. 살아 있는 프로그램의 조회 실패는 계속 거부, 검증하지 않은 자식의 강제 종료 권한 추가 없음. 같은 컴퓨터의 자식 40개씩 비교에서 수정 전 오류 2건·수정 후 0건. 모든 Windows 프로그램의 무오류 보장 제외.
