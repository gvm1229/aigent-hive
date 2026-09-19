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
  - "repo:crates/hive-cli/src/policy/configure.rs#sha256:c85cea280be13c31da88fd5463eb1504572dd86c7cfb0d2e10ab825a499a0612"
  - "repo:schemas/host-policy-intent.schema.json#sha256:09cbc4e7c9bb5065bb6035356b9a5313708af23baa0f4d2778fbe7920319f495"
  - "repo:tests/conformance/contracts/test_native_policy_hooks.py#sha256:b9471386e86affb501ce3c2b6c7f621c838fe56bef0ac3c32a4b561c36d95082"
links: [foundation-refactor, project-policy-enforcement]
reviewed_revision: "git:80779cb0da9104108c3e5349d6493cd726e9077d"
status: active
---

# 호스트 훅 검사기 실행 실패의 차단 경계

- 0.11.0 실제 훅 검증에서 검사기 부재 시 보호 편집 허용 발견
- 생성한 `PreToolUse` 명령: 검사기 부재·실패 종료·빈 출력의 명시적 거부, 부분 출력·원본 오류의 전달 제외
- 정상 호스트 권한 응답 유지
- 명령 형식 1의 승인 지문 보존, 형식 2 적용의 새 미리 보기·신뢰 필요
- 실행 파일 위치 결합 유지. 호스트 미로드·시간 초과의 차단 증명 제외
