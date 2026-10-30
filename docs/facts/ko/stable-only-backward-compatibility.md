---
schema_version: 1
pair_id: stable-only-backward-compatibility
topic_slug: stable-only-backward-compatibility
language: ko
counterpart: ../en/stable-only-backward-compatibility.md
title: "안정판 호환성과 정확한 시험판 예외"
summary: "전체 출처 인증의 안정판 호환과0.14.0의 정확한 공개0.13.2-test.2 예외"
tags: [compatibility, provenance, release, v0-12-0]
aliases: []
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:1ab23dbb5a8b60322f6c1068ba7cc09f54bc34151915068a748a1cc5a2c38018"
  - "repo:crates/hive-render/src/lib.rs#sha256:ae4532f02e5c3bd00a88534dfb974cbda2a7d6c02d5b43f43d50e156a9137904"
  - "repo:docs/decisions/ADR-0025-0.11.2-scope.md#sha256:958df30cdb8c19d4591eed2324587cd0cdb3fb90a30da8ef01daee015efc150e"
  - "repo:tests/results/acceptance-0.12.0-test.3.json#sha256:56a95cce7790c624305121943ac216f2651350f2b9325b8f61a24db566b89304"
  - "repo:tests/results/runs/20261025T093057-ad8ea764ff85.md#sha256:967efbf8264cbd3006be34c738a08c80dc76bc482bbeb4e64141fe95a7ba1369"
links: [source-development, v0-11-2-scope]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# 안정판 호환성과 정확한 시험판 예외

- 기본 호환 대상은 안정판. 0.14.0의 예외는 전체 바이트 인증을 마친 공개0.13.2-test.2만 허용. 출처 누락의 안정판 단정 금지
- Orireki 공식0.11.0-test.6의 제품 번호0.11.0만 남아 구분 유실. 0.12.0은 컴파일 시험 번호를 지문에 보존, 다른 시험판은 무변경 거부
- test.6 예외·기준본 조작 없음, 설치0.11.1·Orireki 파일 보존
- 공개0.12.0-test.3의 Windows·Linux·macOS 출처/변조 거부/과거 시험판 제외/안정판 보존 통과. 실제 Orireki 갱신 제외
