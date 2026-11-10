---
schema_version: 1
pair_id: project-knowledge-storage-policy
topic_slug: project-knowledge-storage-policy
language: ko
counterpart: ../en/project-knowledge-storage-policy.md
title: "프로젝트 지식 저장과 접근"
summary: "검색 공개 범위·승격 승인과 저장 위치·쓰기 권한의 구분."
tags: [index, knowledge, policy]
aliases: []
sources:
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:b3f259173a92bf050ca26df5b462330ea4ce9d96bb9bca431b6a54700ddfea36"
  - "repo:crates/hive-wiki/src/shared.rs#sha256:4e65063d1f7d0dca6db2361f465136c892d8d357fa641c8337f29ba68a356965"
  - "repo:docs/plans/details/project-local-global-readonly/design.md#sha256:18defa9a2f898ab073eebdf6345a2e64a449619879f2b4f46af074708b335a12"
links: [knowledge-cross-project-access, shared-index]
reviewed_revision: "git:dd581c30013516a6649e74e3c0d80bf7d46cb278"
status: active
---

# 프로젝트 저장 경계

0.14.0의 프로젝트 비공개 사실·검색 자료는 사용자 루트에 보관.
빈 승격 범주와 전역 읽기 전용 권한의 구분. 초기화한 별도 사용자 루트는
사용 가능하지만 두 저장소의 읽기 전용 결합 정책은 별도 기능.
로컬 쓰기 요청의 조사 기준: 저장 위치·검색 공개·승격·쓰기 권한의 분리,
최종 지문 외 파일 사건의 관측. 범위·한계는
[진단](../../research/project-local-global-readonly-0.14.0/README.md) 참조.

계획 조건: 읽기 전용은 명시 선택만 적용, 기존 읽기·쓰기 동작·기본값 보존.
공통 연결 상태 수정은 별도 작업.
