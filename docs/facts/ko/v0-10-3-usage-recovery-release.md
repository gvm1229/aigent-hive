---
schema_version: 1
pair_id: v0-10-3-usage-recovery-release
topic_slug: v0-10-3-usage-recovery-release
language: ko
counterpart: ../en/v0-10-3-usage-recovery-release.md
title: "0.10.3 사용량 보호 복구 안정판"
summary: "0.10.3은 기존 halt.json을 같은 위치에서 재측정 결과로 교체하거나 제거해 옛 표식만으로 작업을 막지 않는 안정판."
tags: [release, usage-guard, v0-10-3]
aliases: ["0.10.3 안정판"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:881ae77507817b888cf66ff0da2ee52fbc4048fbb9355641a093dcf6f3d69fc1"
  - "repo:docs/plans/active/usage-recovery-0.10.2.md#sha256:6419915d2b1149efa2613fb0f84c4659f787e1d7f59f3489103b205d7f0a2691"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
links: [release-verification, automatic-test-release-gate]
reviewed_revision: "git:8afe730759325bc003475f81545da7afc88cfb18"
status: active
---

# 0.10.3 사용량 보호 복구 안정판

- 옛·이전 프로세스·손상된 일반 `halt.json`: 현재 사용량 재측정
- 허용 측정: 같은 경로의 표식 제거
- 부족·불명확 측정: 같은 경로의 현재 형식 표식 원자적 교체
- 심볼릭 링크·경로 이탈·미지원 미래 형식: 사용량 부족과 구분한 안전 오류
- 공개 수용: `0.10.3-test.1`의 Windows·macOS·Linux와 실제 기존 설치
- 안정판 공개: npm `latest=0.10.3`, GitHub `v0.10.3` 정식 Release
