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
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:958df30cdb8c19d4591eed2324587cd0cdb3fb90a30da8ef01daee015efc150e"
  - "repo:tests/results/acceptance-0.12.0-test.3.json#sha256:56a95cce7790c624305121943ac216f2651350f2b9325b8f61a24db566b89304"
links: [source-development, v0-11-2-scope]
reviewed_revision: "git:032b1b5083970fa092d85fcd49ea7847c185b15e"
status: active
---

# 안정판만의 이전 버전 호환성

- 사용자 결정: 시험판은 이전 안정판 호환성 대상에서 제외
- Orireki 공식0.11.0-test.6 기준본의 제품 번호0.11.0만 남아 출처 구분 유실
- 개발0.12.0: 정확한 컴파일 시점 시험 패키지 번호를 기준본 지문에 보존, 다른 시험판은 변경 없이 거부
- 옛 안정판의 전체 바이트 인증 유지. 출처 누락만으로 안정판 출처 단정 금지
- test.6 호환 예외·소비자 기준본 조작 없음. 설치 안정판0.11.1·Orireki 파일 보존
- 공개test.3의 Windows·Linux·macOS 출처·변조 거부·과거 시험판 제외·안정판 보존 검사 통과. 실제 Orireki 갱신 제외
