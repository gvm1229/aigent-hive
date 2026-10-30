---
schema_version: 1
pair_id: windows-native-version-output
topic_slug: windows-native-version-output
language: ko
counterpart: ../en/windows-native-version-output.md
title: "Windows 실행 파일의 버전 문자 해석"
summary: "정확한 버전 출력의 UTF-8 해석과 호출자 문자 설정 복원"
tags: [encoding, installation, windows]
aliases: []
sources:
  - "repo:scripts/install.ps1#sha256:25e0cad4ec3265eb838989036adddde01fffadbec498e0dc12302965c29d5951"
  - "repo:tests/results/runs/20261029T223136-9775bcc685b1.md#sha256:0026fabac51fcd69914724f52efcbe3dc2b84aa9370f5f0ca8c389b657914806"
  - "repo:tests/results/runs/20261029T223512-967275e97cca.md#sha256:edb5ca20375ade65d829c7df22a353456f0d4e7f1c5b517ef848ba42597d93a4"
links: [windows-direct-replacement, windows-powershell-module-isolation]
reviewed_revision: "git:a03c6a2e91d14f04e159aa2009a72ad8fa799f15"
status: active
---

# Windows 실행 파일의 버전 문자 해석

Windows 설치기의 실제 --version 출력은 UTF-8 해석 후 정확한 버전 대조. PowerShell5.1의 CP437 해석에서 가운데 점이 두 글자로 변해 정상 공개0.14.0-test.2 거부 재현. 해당 호출 동안만 Console.OutputEncoding 변경, finally에서 원래 설정 복원. 다른 버전·종료 실패 거부 유지. 공개 바이너리 조회와 PowerShell5.1/7 자식 출력으로 문자 해석·복원 검증, 전체 설치 수용과 구분. 출처:0.14.0 첫 갱신 완료 요청.
