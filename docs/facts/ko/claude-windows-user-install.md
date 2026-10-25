---
schema_version: 1
pair_id: claude-windows-user-install
topic_slug: claude-windows-user-install
language: ko
counterpart: ../en/claude-windows-user-install.md
title: "Claude Windows 사용자 설치"
summary: "Claude 명령의 일반 Windows 경로 전달과 제한된 안전 오류 진단."
tags: [claude, installation, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage.rs#sha256:ce45a82f02376c71b9bd25a7daf377c11fa1c9b55a4f08098098f97cd7ae920f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2fade132e2fe4e60b3f7b70e089d5deaa145a25e2976bdfdb62a58a82fa86192"
  - "repo:crates/hive-cli/src/user_install/host_state.rs#sha256:8c2ada111bbfd27361a56a75a38c2c379c3517110233590717cd97caa6f5c8be"
  - "repo:scripts/qualify-claude-user-install.py#sha256:29efd013c8f258c979e14095297db8e277737bfdbb63620ec776dd315fe2de52"
  - "repo:tests/results/legacy/af6a478d5074c2d3a277.md#sha256:ff1b2012dc26150493fa439cf4eab9f546d64549640d94622d9e1ee9704ff9bd"
  - "repo:tests/results/runs/20261001T234717-11673280f355.md#sha256:72fa345ddb6e4feff83592041055cf32edb4a3bd3347cee6a5af5423328ea692"
links: [multi-host-user-install, supported-hosts]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Claude Windows 사용자 설치

Windows 확장 경로의 파일 검사 통과와 Claude 마켓플레이스 등록 거절 차이. 내부 파일 접근의 정규 경로 유지, Claude 명령 인자만 일반 경로로 변환. 등록 전 파일 기록, 실패 뒤 복구에 따른 파일 부재 가능성.

명령 실행기의 제한된 오류 출력·종료 코드 수집. 고정 오류 분류와 출력 길이·지문만 표시, 원문 비노출. 소유권 불명확 중단의 외부 상태·복구 기록 보존.

반복 가능한 실제 CLI 격리 시험: 일반·공백·한글 경로, 설치·검증·재설치·갱신과 외부 자료 보존. 실제 대화의 스킬 발견·모델 질문은 CLI 시험의 증명 범위 밖.

공개 `0.11.1-test.6`·Claude `2.1.163` 통과.
