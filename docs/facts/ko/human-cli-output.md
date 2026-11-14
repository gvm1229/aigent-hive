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
  - "repo:.agents/directives/02-architecture.md#sha256:6f4b5504814c95be4ef7c7035bd090db945804356dafdaf19944af77eb9c3d3b"
  - "repo:crates/hive-cli/src/human_output.rs#sha256:ac6e4cc075655aa9ee8eabfc58fb6257dbd729861bb97c643084cd006139ebf6"
  - "repo:crates/hive-cli/src/main.rs#sha256:41ee2245710b7b06a878b7db926a069260c6cd816a93aa95a7eb7cdd8e0c9eac"
  - "repo:crates/hive-cli/tests/human_output.rs#sha256:717e0bef3a101dc5ae5d24c95320cfb234cd17306b35c6667834b2779745e8f0"
links: [inert-summon-contract, usage-threshold-provenance]
reviewed_revision: "git:43dad991f214a9afdc50524cf2b7f32c22ae46a1"
status: active
---

# CLI 사람용 출력

0.15.1 미발행 소스의 사용자용 명령 기본값: 설정한 한국어·영어의 결과·변경값·파일·복구·다음 행동 안내. 공통 출력기 사용, --output json의 기존 지원 스키마·값·종료 코드 보존. 기존 JSON 미지원 요청의 거부 유지. 내부 작업자 이름과 hook·policy hook·usage capture·Discord inbound 명령은 항상 기존 기계 프로토콜 유지. usage show·설정 요약의 JSON 인자 필수 조건 제거. 실제 설치·정식 공개 완료의 근거와 구분.
