---
schema_version: 1
pair_id: interactive-binary-update
topic_slug: interactive-binary-update
language: ko
counterpart: ../en/interactive-binary-update.md
title: "대화형 바이너리 갱신"
summary: "Bare hive update의 인증된 현재 설치 소유자 위임과 명시적 수락."
tags: [installation, update]
aliases: ["설치 소유자 갱신"]
sources:
  - "repo:README.md#sha256:a8326c3d7cf53451e09dcca9bc54f34b00b0428cf3d606e8e0c40ff3adf7b845"
links: [test-distribution, update-discovery, update-transaction]
reviewed_revision: "git:1144b25b9653cbb3e2a39bc6716acd13239f4ac7"
status: active
---

# 대화형 바이너리 갱신

Bare `hive update`: 대화형 터미널과 선택 언어 사용. npm `latest` 확인 뒤 실행 중인
npm 패키지 명세 또는 직접 설치 영수증 인증. 기존 `0.8.0-test.N` 소유권도 인증하며
같은 제품의 exact `0.8.0`보다 낮은 순서로 판정. 정확한 설치 소유자 작업 미리보기와
명시적 수락 뒤 설치, 활성 소유자·버전 재검증. 거절·EOF·비대화형 호출의 설치 변경
없음.
