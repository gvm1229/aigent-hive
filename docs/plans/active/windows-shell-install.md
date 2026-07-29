# Windows shell 설치 경계 계획

> Checklist owner: `WSI-*`
> Target: `0.8.0`
> Decision: [`ADR-0013`](../../decisions/ADR-0013-preview-release-scope.md)
> Upstream: [Microsoft PowerShell Windows 설치](https://learn.microsoft.com/powershell/scripting/install/install-powershell-on-windows)
> 확인일: 2026-07-29

## 결정

| Surface | 계약 |
| --- | --- |
| Consumer `hive.exe` | PowerShell runtime dependency 없음 |
| Consumer direct install | Windows 기본 `powershell.exe` 5.1 지원 |
| Consumer `cmd.exe` | Exact-version PowerShell 5.1 bootstrap 호출 명령 지원 |
| Consumer PowerShell 7 | 설치 요구·탐지 경고·설치 제안 없음 |
| Source development | PowerShell 7 LTS를 Windows 개발·release dependency로 한정 |
| Source dependency setup | 명령·package·scope preview와 명시적 동의 뒤 Microsoft 지원 installer 위임 |
| Dependency ownership | Microsoft 또는 선택한 package manager 소유, Hive update·uninstall 없음 |

## 확인된 결과

- 실제 Windows 11 x86_64: Windows PowerShell `5.1.26100.8875`,
  PowerShell `7.6.4` LTS, Rust·Cargo `1.97.1`
- `scripts/install.ps1`: PowerShell 5.1·7 parser와 overwrite·receipt·pending
  recovery runtime PASS
- Atomic overwrite: 기존 destination의 `[IO.File]::Replace`와 신규 destination의
  two-argument `[IO.File]::Move` 분리
- UTF-8: Windows PowerShell 5.1·7 공통 BOM 없는 .NET UTF-8 byte contract
- Source dependency: `Microsoft.PowerShell` `7.6.4.0` user-scope exact WinGet
  설치와 재검증 PASS
- Consumer negative boundary: `pwsh`, `winget`, `Microsoft.PowerShell` 참조 0건

## 구현

- [x] [WSI-001] `scripts/install.ps1`의 Windows PowerShell 5.1 runtime 지원:
  overwrite·receipt·pending recovery의 atomicity, reparse·ownership·hash·signature
  경계와 PowerShell 7 결과 parity, shell-version 독립 UTF-8 byte contract
- [x] [WSI-002] `cmd.exe` paste/run용 exact-version direct-install 명령:
  built-in `powershell.exe -NoProfile -NonInteractive` 위임, 공백·quote·`%`·`!`
  안전성, child exit code 전달, temporary cleanup과 기존 release 검증 경로 단일화
- [x] [WSI-003] Source Windows dependency setup의 PowerShell 7 LTS detection·preview·
  explicit consent·Microsoft installer handoff와 재검증, 거절 시 external mutation 0건,
  consumer bundle·installer·harness의 `pwsh` dependency·설치 prompt 0건

완료 증거:

- `python -m unittest tests.conformance.test_phase6_update`: 21개 실행,
  platform 비대상 8개 expected skip, 나머지 PASS
- Windows PowerShell 5.1·PowerShell 7.6.4 동일 installer runtime PASS
- `cmd.exe /D /V:OFF` 공백·`%`·`!` prefix와 child exit code `23` 전달 PASS
- `scripts/setup-windows-dependencies.ps1` preview·동의·fake WinGet·재검증 PASS

## Qualification

- PowerShell 5.1: direct install, repeat install, update, pending receipt recovery,
  local modification·foreign path 보존
- `cmd.exe`: 공백 포함 prefix, exact version, success·failure exit code, cleanup parity
- PowerShell 7: source test·release workflow와 consumer installer compatibility 보조 검증
- Negative boundary: consumer setup·update·plugin install 중 PowerShell 7 detection,
  download, package-manager invocation 0회
- Actual Windows acceptance: `WSI-001–003` 완료 뒤 `P7-041` 실행
