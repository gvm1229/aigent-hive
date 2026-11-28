---
schema_version: 1
pair_id: human-cli-output
topic_slug: human-cli-output
language: ko
counterpart: ../en/human-cli-output.md
title: "CLI 사람용 출력"
summary: "사용자용 명령의 쉬운 기본 안내와 명시적 기계 프로토콜의 기존 계약 보존."
tags: [cli, compatibility, output]
aliases: ["human-readable CLI"]
sources:
  - "repo:.agents/directives/02-architecture.md#sha256:b2a6a7eb65b8382b59dd2a2c5599bdb0c1ceb1fd3f8ee1caaa1b2511a86c9db5"
  - "repo:crates/hive-cli/src/human_output.rs#sha256:b4ac3de0765769a0daa598d2c58440c83a14f8a138622b23c83c0a18f79be5fc"
  - "repo:crates/hive-cli/src/main.rs#sha256:db22fe13b2c2a5d0d342d002b791d335874346c6da3d059f0a5957718602b74d"
  - "repo:crates/hive-cli/tests/human_output.rs#sha256:ed87bc2cfbf8ccfac0a3e4a8aeb779903dfdca2f0728f7647244e6bec650f65b"
links: [inert-summon-contract, usage-threshold-provenance]
reviewed_revision: "git:43dad991f214a9afdc50524cf2b7f32c22ae46a1"
status: active
---

# CLI 사람용 출력

0.15.1 미발행 소스의 사용자용 명령 기본값: 설정한 한국어·영어의 결과·변경값·파일·복구·다음 행동 안내. 공통 출력기 사용, --output json의 기존 지원 스키마·값·종료 코드 보존. 기존 JSON 미지원 요청의 거부 유지, 단 0.15.3부터 doctor·check-target은 JSON 결과 지원. 내부 작업자 이름과 hook·policy hook·usage capture·Discord inbound 명령은 항상 기존 기계 프로토콜 유지. usage show·설정 요약의 JSON 인자 필수 조건 제거. 실제 설치·정식 공개 완료의 근거와 구분.
