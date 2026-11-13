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
  - "repo:crates/hive-cli/src/human_output.rs#sha256:61c71804508ee28e92b23508f47f3c002190dd6e500e478b28d946a56122c030"
  - "repo:crates/hive-cli/src/main.rs#sha256:a68944d85c5808e5090bba2b54977d6e484f7acdccfbbbd5acc2ab20ba75f7cd"
  - "repo:crates/hive-cli/tests/human_output.rs#sha256:6814fec5d42b5c7358bfca51cbbea1263a204673d572d7e759da285948f00111"
links: [inert-summon-contract, usage-threshold-provenance]
reviewed_revision: "git:43dad991f214a9afdc50524cf2b7f32c22ae46a1"
status: active
---

# CLI 사람용 출력

0.15.1 미발행 소스의 사용자용 명령 기본값: 설정한 한국어·영어의 결과·변경값·파일·복구·다음 행동 안내. 공통 출력기 사용, --output json의 기존 지원 스키마·값·종료 코드 보존. 기존 JSON 미지원 요청의 거부 유지. 내부 작업자 이름과 네이티브 stdin-JSON 훅은 기존 기계 프로토콜의 명시 선택. usage show·설정 요약의 JSON 인자 필수 조건 제거. 실제 설치·정식 공개 완료의 근거와 구분.
