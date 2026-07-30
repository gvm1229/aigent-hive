# `0.8.0` 공개 릴리스 실행 계획

> Target: `0.8.0`
> Checklist owner: [`P7-*`](../phases/07-public-qualification.md)
> Decision: [`ADR-0013`](../../decisions/ADR-0013-0.8-release-scope.md)

## 릴리스 정의

- 공개 명칭: `Aigent Hive 0.8.0`
- 성숙도 표기: 별도 `preview` label·npm dist-tag 없음
- 배포 channel: npm registry, GitHub Release, Unix·Windows 직접 설치
- Consumer runtime: Rust native binary, Node.js·PowerShell 7 dependency 없음
- 신뢰 기준: exact tag·source commit, SHA-256, GitHub artifact attestation
- 업데이트 방식: package manager 또는 검증된 직접 설치
- Network self-update: 비활성
- 미검증 공개: Claude Code subscription-backed 실제 E2E와 usage parity

`0.x` SemVer 자체를 pre-1.0 성숙도 신호로 사용. Stable·production-ready 보증 대신
검증 host·version, known limitation, deferred signing 범위 공개.

## 지원 platform

| Platform | Rust target | 배포 경로 |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | npm·GitHub·`install.sh` |
| macOS Intel | `x86_64-apple-darwin` | npm·GitHub·`install.sh` |
| Linux x86_64 | `x86_64-unknown-linux-musl` | npm·GitHub·`install.sh` |
| Linux arm64 | `aarch64-unknown-linux-musl` | npm·GitHub·`install.sh` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | npm·GitHub·PowerShell·CMD |

Linux의 static-friendly musl artifact를 기본 배포 단위로 채택. 두 Linux target의
release build·install·runtime·upgrade는 `P7-043`에서 실제 검증 전 지원 완료 주장 금지.

## 단일 artifact 계보

```text
protected exact source
  → platform native build 1회
  → archive + SHA-256 + GitHub attestation
  → GitHub Release
  → npm platform package / 직접 설치 adapter
```

- GitHub Release archive와 npm package의 native binary byte identity
- Installer·npm wrapper의 binary 재빌드·postinstall download 금지
- Platform allowlist, exact version, digest, archive member 검증
- 설치 receipt·package-manager ownership·atomic activation·failure recovery 유지

## npm 전역 설치

사용자 명령:

```console
npm install -g aigent-hive
```

Package 구조:

- Unscoped public umbrella package: `aigent-hive`
- Platform package: `@aigent-hive/darwin-arm64`, `darwin-x64`, `linux-arm64`,
  `linux-x64`, `win32-x64`
- Umbrella의 exact-version optional dependency와 `hive` executable shim
- Platform package 선행 publication, umbrella package 최종 publication
- `0.8.0`을 npm `latest`로 publication, `preview` dist-tag 없음
- Package install 이후 `hive --version`과 native architecture smoke
- Package name·scope 확보와 최초 public publication은 사용자 소유 외부 경계
- Node.js/npm은 npm channel의 installer dependency일 뿐 `hive` runtime dependency 아님

## 직접 설치

Unix 사용자 명령:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/gvm1229/aigent-hive/releases/latest/download/install.sh | sh
```

Windows PowerShell 사용자 명령:

```powershell
irm https://github.com/gvm1229/aigent-hive/releases/latest/download/install.ps1 | iex
```

Windows CMD:

- GitHub Release의 versioned `install.cmd` 다운로드·실행
- Windows 기본 `powershell.exe` 5.1 위임
- PowerShell 7 탐지·설치 요구·설치 제안 없음

직접 installer 계약:

- Release publication 시 삽입된 exact artifact name·SHA-256
- OS·architecture allowlist와 TLS 제한
- Temporary staging, ownership receipt, atomic replace, pending recovery
- Existing local modification·foreign path 보존
- Unsigned pre-1.0 limitation 안내, Gatekeeper·SmartScreen 전역 완화 금지

## Workflow 역할

| Workflow | 역할 |
| --- | --- |
| `release.yml` | 5개 target build, test, archive, digest, attestation, npm tarball staging |
| `release-publish.yml` | exact candidate 검증, GitHub Release와 npm 순차 publication |

Developer ID·notarization·Authenticode·Azure signing·external TUF는 `0.8.0` 필수
secret에서 제외. 관련 input 부재를 실패로 취급하지 않고 future hardened channel의
별도 opt-in gate로 유지.

## 남은 작업

1. `P7-043`: Linux x86_64·arm64 musl build, archive, install, runtime qualification
2. `P7-044`: npm package family와 exact-version platform selection·native smoke
3. `P7-045`: `install.sh`·`install.ps1`·`install.cmd` 단순 명령과 digest 검증
4. `P7-020`: 5개 archive·npm tarball의 SHA-256·attestation·source provenance
5. `P7-018`: protected `main` exact `0.8.0` 전체 candidate qualification
6. `P7-037`: GitHub normal release와 npm `latest` publication

## 완료 기준

- 5개 target artifact와 expected architecture 일치
- GitHub archive와 npm-installed binary의 SHA-256 byte identity
- npm·curl·PowerShell·CMD의 clean install·repeat update·recovery PASS
- Consumer runtime의 Node.js·PowerShell 7 dependency 0건
- Exact `0.8.0` version parity와 clean-clone 전체 CI PASS
- Codex·Antigravity 실제 host 회귀 PASS
- Claude Code 미검증 상태와 signing deferred 범위 공개
- Package name·scope·registry publication 사용자 승인

## 외부 중지 경계

- Protected GitHub Release와 npm 최초 publication 직전 사용자 최종 확인
- npm package name·scope ownership과 Trusted Publishing 설정
- Credential·private signing material 접근 금지
- Exact `1.0.0` 별도 명시 전 major release 준비 금지

## 후속 hardening

- macOS Developer ID signing·notarization
- Windows Authenticode 또는 Azure Artifact Signing
- External TUF 2-of-3 production authorization
- 실제 Claude Code subscription session qualification
- `cargo-dist`의 ownership·recovery·artifact 계보 fit-gap 재평가
