---
schema_version: 1
pair_id: shared-index-target-safety
topic_slug: shared-index-target-safety
language: ko
counterpart: ../en/shared-index-target-safety.md
title: "공유 색인 대상 경로 안전"
summary: "공유 지식 명령은 정규화 전 연결된 소비자 대상을 거부하고 폐기된 프로젝트 stale 표지를 무시."
tags: [index, security, symlink]
aliases: ["공유 지식 대상 보호", "과거 stale 표지"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:676c827ebe9ed1c4e51d73cf35289b8dfa9149f52287bcfad1a33d0b872b3be2"
links: [knowledge-storage, shared-index]
reviewed_revision: "git:e5c2c599562121ed3dc43143c16a0b1f063cefa2"
status: active
---

# 공유 색인 대상 경로 안전

공유 지식 질의·변경: 입력한 소비자 대상의 기존 경로 요소를 정규화 전에 검사하고
symbolic link 포함 시 거부. 폐기된 프로젝트 로컬 `.hive/index/.stale` 표지: 공유
질의·재구축에서 읽기·변경 없음. 완료 기준: 등록 프로젝트 연결 별칭은 프로젝트
변경 없이 실패, 과거 표지 연결은 공유 색인 작업 성공 중에도 byte 보존. 요청 배경:
권한 없는 로컬 Windows 환경에서 미실행됐던 연결 검사의 운영체제별 CI 실패.
