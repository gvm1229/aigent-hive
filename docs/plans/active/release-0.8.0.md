# `0.8.0` 시험 배포 실행 계획

> Target: `0.8.0`
> Checklist owner: [`P7-*`](../phases/07-public-qualification.md)
> Decision: [`ADR-0013`](../../decisions/ADR-0013-0.8-release-scope.md)

## 배포 정의

- 목적: 실제 안정 릴리스 전 설치·업데이트·host onboarding 검증
- GitHub: Release와 prerelease를 만들지 않음
- npm: exact `0.8.0`과 `test` dist-tag만 publication, `latest` 이동 없음
- 직접 설치: GitHub Release asset이 아니라 npm registry의 같은 native package 사용
- Consumer runtime: Rust native binary, Node.js·PowerShell 7 dependency 없음
- 신뢰 기준: protected `main` exact commit, SHA-256, GitHub artifact attestation
- 공개 명칭: 안정 릴리스로 부르지 않고 `0.8.0 test distribution`으로 표시

실제 안정 릴리스는 사용자가 별도로 승인할 `0.8.x`에서 수행. 그때 npm `latest`,
GitHub normal release, 안정 설치 명령과 limitation 문구를 다시 확정.

## 지원 platform

| Platform | Rust target | 시험 배포 경로 |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | npm·`install.sh` |
| macOS Intel | `x86_64-apple-darwin` | npm·`install.sh` |
| Linux x86_64 | `x86_64-unknown-linux-musl` | npm·`install.sh` |
| Linux arm64 | `aarch64-unknown-linux-musl` | npm·`install.sh` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | npm·PowerShell·CMD |

Linux는 static musl artifact를 기본 배포 단위로 사용. 실제 build·install·runtime
증거 없는 platform은 지원 완료로 표시하지 않음.

## 단일 artifact 계보

```text
protected main exact commit
  → platform native build 1회
  → archive + SHA-256 + GitHub attestation
  → npm platform package
  → npm umbrella / digest-pinned direct installer
```

- GitHub Release asset·release tag 생성 없음
- Candidate artifact와 npm package의 native binary byte identity
- Installer·npm wrapper의 binary 재빌드·postinstall download 금지
- Platform allowlist, exact version, digest, archive member 검증
- 설치 receipt·package-manager ownership·atomic activation·failure recovery 유지

## npm 시험 설치

```console
npm install -g aigent-hive@0.8.0
```

또는 시험 channel을 명시:

```console
npm install -g aigent-hive@test
```

- Public umbrella package: `aigent-hive`
- Platform package: `@aigent-hive/darwin-arm64`, `darwin-x64`, `linux-arm64`,
  `linux-x64`, `win32-x64`
- Exact-version optional dependency와 `hive` executable shim
- Platform package 선행 publication, umbrella package 최종 publication
- 모든 package는 `--tag test --provenance`; `latest` 이동 금지
- Package install 이후 `hive --version`과 native architecture smoke
- Node.js/npm은 npm 설치 channel의 dependency일 뿐 `hive` runtime dependency 아님

안정 릴리스 승인 뒤에만 기본 명령 `npm install -g aigent-hive`가 최신 안정 버전을
설치하도록 `latest`를 이동.

## curl·Windows 직접 설치

Unix:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://raw.githubusercontent.com/gvm1229/aigent-hive/main/scripts/install.sh | sh
```

Windows PowerShell 5.1+:

```powershell
irm https://raw.githubusercontent.com/gvm1229/aigent-hive/main/scripts/install.ps1 | iex
```

Windows CMD:

```bat
curl.exe -fLo install-aigent-hive.cmd https://raw.githubusercontent.com/gvm1229/aigent-hive/main/scripts/install.cmd && install-aigent-hive.cmd
```

Bootstrap은 embedded exact `0.8.0`과 platform package digest를 검증하고 npm registry
tarball에서 native binary를 취득. npm CLI나 Node.js는 직접 설치 경로에 필요하지 않음.
PowerShell 7 탐지·설치 요구·설치 제안도 없음.

## 업데이트 UX

초기 global setup의 첫 질문은 `English|한국어`. 이후 모든 질문과 전역 host 지침은
선택 언어 사용. 같은 setup에서 일 1회 자동 업데이트 확인을 별도로 opt-in.

- 자동 확인은 설치하지 않고 새 버전과 exact 명령만 알림
- 마지막 성공 확인으로부터 24시간 동안 재호출하지 않음
- Offline·timeout·registry 오류는 성공 시각을 기록하지 않음
- 다음 Codex·Claude Code·Antigravity session 첫 작업에서 즉시 재시도
- Raw registry 응답·credential·provider runtime state 저장 금지
- `hive update`는 즉시 확인하고 새 버전이 있으면 설치 전 명시적으로 질문
- 수락 뒤 현재 install owner의 exact-version adapter만 실행
- 거절·EOF·noninteractive invocation은 mutation 0건

## README

- Root `README.md`: 간결한 English canonical overview
- `docs/readme/README.ko.md`: 같은 핵심 구조의 한국어 문서
- 각 문서의 `Languages` 링크로 상호 이동
- 설치, 지원 범위, 안전 경계, 빠른 시작, update, contribution만 root README에 유지
- 상세 architecture·개발 계약은 기존 `docs/` 링크로 이동
- 두 문서에 빈 `QA Contributors` 표 유지

## Workflow 역할

| Workflow | 역할 |
| --- | --- |
| `release.yml` | 5개 target build, runtime 검증, digest, attestation, npm tarball staging |
| `release-publish.yml` | exact candidate와 attestation 재검증, npm `test` publication |

Publication workflow는 Git tag·GitHub Release를 생성하지 않음. Developer ID,
notarization, Authenticode, Azure signing, external TUF는 실제 안정 릴리스의
별도 hardened gate로 deferred.

## 실행 순서

1. `P7-046`: 영어·한국어 간결 README와 빈 QA Contributors 표
2. `P7-047`: language-first setup과 localized global harness
3. `P7-043`: Linux x86_64·arm64 musl qualification
4. `P7-044`: npm package family와 native smoke
5. `P7-045`: npm-backed Unix·PowerShell·CMD installer와 authenticated owner receipt
6. `P7-049`: authenticated install-owner adapter를 사용하는 대화형 `hive update`
7. `P7-020`: archive·npm tarball SHA-256·attestation·byte identity
8. `P7-018`: protected `main` exact `0.8.0` candidate qualification
9. `P7-037`: npm `test` publication과 npm·curl clean install 검증

`P7-049`는 `P7-044`와 `P7-045`가 exact-version package와 authenticated
install-owner adapter를 확정한 뒤 진행한다. 불확실한 owner를 추측하거나 Hive가
설치 관리자를 우회해 binary를 직접 덮어쓰지 않는다.

## 완료 기준

- 5개 target artifact와 expected architecture 일치
- Candidate archive와 npm-installed binary의 SHA-256 byte identity
- npm exact/test와 curl·PowerShell·CMD clean install·repeat update·recovery PASS
- GitHub Release·prerelease·release tag 0건, npm `latest` 이동 0건
- Auto check가 24시간 success throttle과 offline next-session retry를 만족
- Bare `hive update`가 확인·질문·동의·설치·재검증 순서를 만족
- English·Korean initial setup과 global guidance byte fixture PASS
- Consumer runtime의 Node.js·PowerShell 7 dependency 0건
- Exact `0.8.0` version parity와 clean-clone 전체 CI PASS
- Codex·Antigravity 실제 host 회귀 PASS
- Claude Code 미검증 상태와 signing deferred 범위 공개

## 외부 중지 경계

- npm package name·scope ownership과 Trusted Publishing 설정
- Credential·private signing material 접근 금지
- npm `latest`, GitHub Release, 안정 릴리스는 별도 사용자 승인 전 금지
- Exact `1.0.0` 별도 명시 전 major release 준비 금지

## 후속 안정 릴리스

- 사용자가 승인할 `0.8.x` exact version 결정
- npm `latest` 이동과 기본 install 명령 전환
- GitHub normal release 필요 여부 재승인
- macOS Developer ID signing·notarization
- Windows Authenticode 또는 Azure Artifact Signing
- External TUF 2-of-3 production authorization
- 실제 Claude Code subscription session qualification
