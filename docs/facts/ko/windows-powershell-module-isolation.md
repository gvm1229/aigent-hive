---
schema_version: 1
pair_id: windows-powershell-module-isolation
topic_slug: windows-powershell-module-isolation
language: ko
counterpart: ../en/windows-powershell-module-isolation.md
title: "Windows PowerShell 모듈 격리"
summary: "Windows 설치기는 모듈 자동 불러오기 없이 SHA-256을 계산하여 CMD가 물려준 PowerShell 7 모듈 경로와 PowerShell 5.1의 충돌을 차단."
tags: [installation, powershell, security, windows]
aliases: ["PSModulePath 격리"]
sources:
  - "repo:docs/archive/plans/foundations/windows-shell-install.md#sha256:1a1f2b2e0657be2b2aa03bb5ea258cac5a8e9f8b81630b7105a7470d8b298654"
links: [test-distribution]
reviewed_revision: "git:cdde668bed5f3b35e08a35f64e7e25594ce9c3a2"
status: active
---

# Windows PowerShell 모듈 격리

CMD 설치 시작 파일은 Windows PowerShell 5.1 실행. 일부 컴퓨터의 상속된
`PSModulePath`는 PowerShell 7 모듈 폴더를 Windows PowerShell 폴더보다 먼저 배치.
모듈 자동 불러오기가 호환되지 않는 `Microsoft.PowerShell.Utility` 선택. 그 결과
`Get-FileHash` 명령 부재. 설치기는 모듈 검색 대신
`System.Security.Cryptography.SHA256`으로 일반 파일을 직접 해시하여 경로 우선순위
영향 제거. 소비자 PowerShell 7 의존 없음. 완료 기준: PowerShell 5.1 최초 설치·반복
설치·대기 영수증 복구·오염된 CMD 환경 실행에서 버전·digest 검증 유지. 요청 배경:
정확한 원인 설명과 clean clone에서도 안전한 Windows 수정.
