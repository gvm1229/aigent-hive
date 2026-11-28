---
schema_version: 1
pair_id: install-wide-knowledge-capture
topic_slug: install-wide-knowledge-capture
language: ko
counterpart: ../en/install-wide-knowledge-capture.md
title: "설치 범위 지식 수집"
summary: "Hive 사용자 범위 수집·조회는 프로젝트 설정 없이 설치 직후 선택 호스트의 모든 프로젝트에 적용."
tags: [capture, knowledge, retrieval, user-root]
aliases: ["미등록 프로젝트 조회", "설정 독립 지식"]
sources:
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:849e3187ee34c5dc9b68740c5fb269a9dd3988ba67bb0d46a105796cbd490061"
  - "repo:crates/hive-cli/src/knowledge/remember.rs#sha256:1ebf35ebbd0ef47b43df98cd9d35c55f41bd303f84be92bbaad72f5ce9c97c14"
  - "repo:crates/hive-cli/src/knowledge/retrieve.rs#sha256:4f3d3dbb7d7b4e456dfb827f116e5f68ce113834c28bfcaaec0fcd3e377ea1a3"
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:4fd5dd55857ac73d960d702e697b1cd05502f68806844c17c1bd199f8d1f4021"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9c5854224d063f07088155f4a61c672587e8bf26c2a67b1c4fb88be9e6a4d4e4"
  - "repo:docs/archive/facts/usage-guard-before-0.13.2.md#sha256:2932a83938b49c193b67e38ba1400c25d3890070dba480674284dc9e2dee07fa"
  - "repo:docs/archive/plans/releases/0.9.0/v0.9.0-knowledge-autocapture-regression.md#sha256:44fcfa9e2c19c626eb8a7885afcaeb6405b454748e62349c1459958d4180236c"
  - "repo:harness/skills/knowledge-capture/SKILL.md#sha256:cfd7fcf8297830b07b54c687060acef03cbc115c20fd952bb0d93562b8211f33"
  - "repo:harness/skills/knowledge-recall/SKILL.md#sha256:d80f5871428941be1d05b12d488857f40bb894752c0c4354cee12f15a46578f6"
  - "repo:harness/skills/usage-guard/references/control.md#sha256:d263c84680b1b11fc49175ee181955a2b18fd90e819b1929212fd818b176e3fd"
links: [global-knowledge-rag, knowledge-storage, shared-index]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# 설치 범위 지식 수집

전역 위키 지침의 모든 선택 호스트 폴더 적용. 안전한 사용자 전역 지식 기록에 프로젝트 설정·하네스·표시 파일·연결 자료 모음 불필요. 미등록 조회의 비공개·기밀 제외. 에이전트 검토·정규화·제한된 전경 기록, 대화 원문 자동 수집 제외. 소진 확인 시 추가 도구 없이 기록·검사 연기, 새 요청의 정상 사용량 확인 뒤 재개.0.9.1 자동 기록·새 세션 회수 근거는 보존 문서 참조.
