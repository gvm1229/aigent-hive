---
schema_version: 1
pair_id: usage-threshold-provenance
topic_slug: usage-threshold-provenance
language: ko
counterpart: ../en/usage-threshold-provenance.md
title: "임계값 확인 기록의 일치"
summary: "0.14.0 전용 임계값 변경의 확인 지문 갱신 누락. 0.14.1은 게시 전 보정 적용."
tags: [preferences, setup, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:5bc49609f63d2ee8d6047c46ea830d3875ada61b9d394c3f9c79bebc8c09cd75"
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/research/usage-threshold-provenance-0.14.0/reproduce.py#sha256:5460ffb967d9381497534988fd6b20f2f89b9faff8c6fa5f2f220df8fd5128c5"
links: [usage-guard-thresholds]
reviewed_revision: "git:dd581c30013516a6649e74e3c0d80bf7d46cb278"
status: active
---

# 임계값 확인 기록의 일치

0.14.0 전용 명령은 값만 변경하고 정상 확인 지문은 유지.
재읽기의 기록 보정 뒤 전체 비교 실패·되돌림 발생.
Windows 가짜 설정의 explicit·legacy-unknown에서 미완료 기록 없이 재현.
일반 부분 답안은 확인 기록도 갱신. 미완료 기록 정리는 제품 수정과 구분.
원인 조사·추가 옵션 계획의 별도 결함 범위는
[근거](../../research/usage-threshold-provenance-0.14.0/README.md) 참조.
0.14.1은 게시 전 후보에 같은 보정을 적용([수정](../../research/overhaul/OV141-002.md)).
