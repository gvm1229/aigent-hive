---
schema_version: 1
pair_id: native-hook-launcher-failure
topic_slug: native-hook-launcher-failure
language: ko
counterpart: ../en/native-hook-launcher-failure.md
title: "호스트 훅 검사기 실행 실패의 차단 경계"
summary: "검사기 실행 실패의 명시적 거부와 호스트 로드·시간 초과의 별도 검증 경계"
tags: [hooks, policy, safety]
aliases: []
sources:
  - "repo:crates/hive-cli/src/policy/configure.rs#sha256:802181187d27fc8b9ab5d2d6d90f449725d327b285dae21ef8c5565e98931633"
  - "repo:schemas/host-policy-intent.schema.json#sha256:0e1877e37080235b1ebc905b02ba7b6b7c245cce7c110b46cb94b6c9b8aacb4d"
  - "repo:tests/conformance/contracts/test_native_policy_hooks.py#sha256:f1365fbdc97d0b4f8e1b18b9dde043a83f609272f48edd485d598301ed04b7bb"
links: [foundation-refactor, project-policy-enforcement]
reviewed_revision: "git:a7359c38fc5510b5d5d34e77217900482f76e712"
status: active
---

# 호스트 훅 검사기 실행 실패의 차단 경계

- 0.11.0 실제 훅 검증에서 검사기 부재 시 보호 편집 허용 발견
- 생성한 `PreToolUse` 명령: 검사기 부재·실패 종료·빈 출력의 명시적 거부, 부분 출력·원본 오류의 전달 제외
- 정상 호스트 권한 응답 유지
- 명령 형식 1·2의 승인 지문 보존, 형식 3에서 Windows 바깥 셸의 변수 해석 방지·새 신뢰 필요
- 실행 파일 위치 결합 유지. 호스트 미로드·시간 초과의 차단 증명 제외
