---
schema_version: 1
pair_id: stable-only-backward-compatibility
topic_slug: stable-only-backward-compatibility
language: ko
counterpart: ../en/stable-only-backward-compatibility.md
title: "안정판만의 이전 버전 호환성"
summary: "안정판만의 이전 버전 호환성"
tags: [compatibility, provenance, release, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:30e453d8375c22b0ec9ec56998fc472dace3fff8cea63ad7ad360d5ff4c19898"
  - "repo:crates/hive-render/src/lib.rs#sha256:62174ca2ea76cf5c379e1638e3ef2c33cc332f3ae312f03a983711f9abe1a1d5"
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:72f2989faf99a78eb09b3f84fe6feb8842e3004b575c7267e4048a2213ee3214"
links: [source-development, v0-11-2-scope]
reviewed_revision: "git:8aefd94546860569fcb5337105381f065b3f887a"
status: active
---

# 안정판만의 이전 버전 호환성

- 사용자 결정: 시험판은 이전 안정판 호환성 대상에서 제외
- Orireki 공식0.11.0-test.6 기준본의 제품 번호0.11.0만 남아 출처 구분 유실
- 개발0.12.0: 정확한 컴파일 시점 시험 패키지 번호를 기준본 지문에 보존, 다른 시험판은 변경 없이 거부
- 옛 안정판의 전체 바이트 인증 유지. 출처 누락만으로 안정판 출처 단정 금지
- test.6 호환 예외·소비자 기준본 조작 없음. 설치 안정판0.11.1·Orireki 파일 보존
