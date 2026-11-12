---
schema_version: 1
pair_id: inert-summon-contract
topic_slug: inert-summon-contract
language: ko
counterpart: ../en/inert-summon-contract.md
title: "비활성 소환 계약"
summary: "실행 활성화 없는 서명 Prepare와 호스트 기능 자료의 지문 결합"
tags: [orchestration, roster]
aliases: []
sources:
  - "repo:crates/hive-cli/src/orchestration.rs#sha256:5617551fbaaffc3495297108a19f05c7aae3e4b0b08b9d56578aaa06bdbcdd84"
  - "repo:crates/hive-cli/src/summon.rs#sha256:6cb0331f2e20b87233d0e8bd9ee8a4dc3f51148e8301867ed0a7333daadfadc9"
  - "repo:crates/hive-core/src/roster.rs#sha256:eaeb59413d1900bcc00ef36e40e35ddd06baff94a8ce129993d622c3991232ea"
  - "repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:ce73d78bda06047d4fe899e1f62f669389978675d668fe37a5a6800bfb76bd10"
links: [hook-inbox-evidence, orchestration-ownership]
reviewed_revision: "git:386bda71844336fe758acc906994991fa5c6f5f0"
status: active
---

# 비활성 소환 계약

0.15.0 summon: 스키마 검사한 호스트 자료의 정확한 지문·작업/대상 지문·역할·사건 머리·권한 결합. Prepare 사건·머리 확정 뒤 작업·수신 봉투 게시. 작업자 프로필은 결과/작업 폴더 자리표시자와 역할별 파일 수정 권한 포함, 서비스 등급은 사용자 Codex 설정 소유. 프로필 왕복 미검증. 미리보기·거부·성공은 ActionResult 규격, 로컬 시험의 실제 호스트 실행 수용 대체 불가.
