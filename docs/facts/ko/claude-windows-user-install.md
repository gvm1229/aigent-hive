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
  - "repo:crates/hive-cli/src/usage.rs#sha256:08df602b839ca6ced6cd1571c37111b300853def9bba4e25c9c24774453535f6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:07c53684b15ca76c0912c67ab9d2dac2464a98580fa186956492a95fa238af63"
  - "repo:crates/hive-cli/src/user_install/host_state.rs#sha256:f7f2d78da2e843f3a75267a106251eb26af09f22773bf67ec8f7bc2cca7f8c60"
  - "repo:scripts/qualify-claude-user-install.py#sha256:29efd013c8f258c979e14095297db8e277737bfdbb63620ec776dd315fe2de52"
  - "repo:tests/results/legacy/af6a478d5074c2d3a277.md#sha256:ff1b2012dc26150493fa439cf4eab9f546d64549640d94622d9e1ee9704ff9bd"
  - "repo:tests/results/runs/20261001T234717-11673280f355.md#sha256:72fa345ddb6e4feff83592041055cf32edb4a3bd3347cee6a5af5423328ea692"
links: [multi-host-user-install, supported-hosts]
reviewed_revision: "git:ad412a7b2aa6bba927f5f841e0e5161f4fb60488"
status: active
---

# Claude Windows 사용자 설치

Windows 확장 경로의 파일 검사 통과와 Claude 마켓플레이스 등록 거절 차이. 내부 파일 접근의 정규 경로 유지, Claude 명령 인자만 일반 경로로 변환. 등록 전 파일 기록, 실패 뒤 복구에 따른 파일 부재 가능성.

명령 실행기의 제한된 오류 출력·종료 코드 수집. 고정 오류 분류와 출력 길이·지문만 표시, 원문 비노출. 소유권 불명확 중단의 외부 상태·복구 기록 보존.

반복 가능한 실제 CLI 격리 시험: 일반·공백·한글 경로, 설치·검증·재설치·갱신과 외부 자료 보존. 실제 대화의 스킬 발견·모델 질문은 CLI 시험의 증명 범위 밖.

공개 `0.11.1-test.6`·Claude `2.1.163` 통과.
