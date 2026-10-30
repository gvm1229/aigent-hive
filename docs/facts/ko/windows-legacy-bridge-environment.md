---
schema_version: 1
pair_id: windows-legacy-bridge-environment
topic_slug: windows-legacy-bridge-environment
language: ko
counterpart: ../en/windows-legacy-bridge-environment.md
title: "Windows 구버전 연결의 환경 전달"
summary: "컴파일러 환경 한도와 전달 변수의 수명"
tags: [compatibility, isolation, update, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/update_bridge/windows.rs#sha256:6300accf43181a9f1e1746029db78a33ed23fa870c0b6987dcbeff05fe0f9b76"
  - "repo:tests/results/runs/20261028T055203-acfe0611bc2e.md#sha256:7f713d57b290777472bf108acb3ff91832c2beb03e741278d01741816df11d1f"
  - "repo:tests/results/runs/20261028T055306-db9e1b405451.md#sha256:7e94b4a0014c46552089127c875c0681acd4445759ad5d151cf186ea581fc06f"
links: []
reviewed_revision: "git:2b9e88e113352ec50c743d69b83b19d7798f2be8"
status: active
---

# Windows 구버전 연결의 환경 전달

0.14.0 첫 갱신 작업에서 CodeDOM 컴파일러의 상속 환경65,535바이트 한도 초과 확인. HIVE_UPDATE_BRIDGE_n을 메모리로 읽은 뒤 연결 프로그램 안의 해당 변수만 제거하고 Add-Type 실행. 다른 환경과 부모 인증 유지.

격리 실패·Windows 연결 검사의 해당 경계만 증명. 공개 출시·실제 앱/감시 제어는 수용 범위 제외.
