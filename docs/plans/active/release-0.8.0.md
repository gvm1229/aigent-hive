# `0.8.0` 시험 배포 실행 계획

> Target: `0.8.0`
> Checklist owner: [`P7-*`](../phases/07-public-qualification.md)
> Decision: [`ADR-0013`](../../decisions/ADR-0013-0.8-release-scope.md)

## 배포 정의

- 목적: 실제 안정 릴리스 전 설치·업데이트·host onboarding 검증
- GitHub: Release·prerelease 생성 없음
- npm: `0.8.0-test.N`과 `test` dist-tag만 publication, `latest` 이동 없음
- 첫 npm 시험판: `0.8.0-test.1`; 반복 시험: `test.2`, `test.3` 순차 증가
- Product candidate: `0.8.0`; npm package version: `0.8.0-test.N`
- 직접 설치: GitHub Release asset이 아니라 npm registry의 같은 native package 사용
- Consumer runtime: Rust native binary, Node.js·PowerShell 7 dependency 없음
- 신뢰 기준: 명시적으로 선택된 protected branch의 exact commit, SHA-256,
  GitHub artifact attestation
- 공개 명칭: 안정 릴리스로 부르지 않고 `0.8.0-test.N test distribution`으로 표시

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
증거 없는 platform: 지원 완료 표시 금지.

## 단일 artifact 계보

```text
selected protected branch exact commit
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
npm install -g aigent-hive@0.8.0-test.1
```

또는 시험 channel을 명시:

```console
npm install -g aigent-hive@test
```

- Public umbrella package: `aigent-hive`
- Platform package: `@aigent-hive/darwin-arm64`, `darwin-x64`, `linux-arm64`,
  `linux-x64`, `win32-x64`
- Exact package-version optional dependency와 `hive` executable shim
- Platform package 선행 publication, umbrella package 최종 publication
- 모든 package는 `--tag test --provenance`; `latest` 이동 금지
- Package install 이후 `hive --version`과 native architecture smoke
- Node.js/npm은 npm 설치 channel의 dependency일 뿐 `hive` runtime dependency 아님

최초 등록만 `release-publication`의 임시 `NPM_TOKEN`과
`bootstrap_with_token=true` 사용. 등록 직후 secret 삭제, 6개 package에
`release-publish.yml` Trusted Publisher 연결, token 차단. 이후 OIDC 전용.

## curl·Windows 직접 설치

Unix:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://unpkg.com/aigent-hive@0.8.0-test.1/install.sh | sh
```

Windows PowerShell 5.1+:

```powershell
irm https://unpkg.com/aigent-hive@0.8.0-test.1/install.ps1 | iex
```

Windows CMD:

```bat
curl.exe -fLo install-aigent-hive.cmd https://unpkg.com/aigent-hive@0.8.0-test.1/install.cmd && install-aigent-hive.cmd
```

Bootstrap은 embedded product `0.8.0`, npm package `0.8.0-test.1`과 platform package
digest를 검증하고 npm registry tarball에서 native binary 취득. 직접 설치 경로의 npm
CLI·Node.js dependency 없음.
PowerShell 7 탐지·설치 요구·설치 제안도 없음.

## 업데이트 UX

초기 global setup의 첫 질문은 `English|한국어`. 이후 모든 질문과 전역 host 지침은
선택 언어 사용. 같은 setup에서 일 1회 자동 업데이트 확인을 별도로 opt-in.

- 자동 확인은 설치하지 않고 새 버전과 exact 명령만 알림
- 마지막 성공 확인부터 24시간 재호출 억제
- Offline·timeout·registry 오류: 성공 시각 기록 제외
- 다음 Codex·Claude Code·Antigravity session 첫 작업에서 즉시 재시도
- Raw registry 응답·credential·provider runtime state 저장 금지
- `hive update`는 즉시 확인하고 새 버전이 있으면 설치 전 명시적으로 질문
- 수락 뒤 현재 install owner의 exact-version adapter만 실행
- 거절·EOF·noninteractive invocation은 mutation 0건

## Workflow 역할

| Workflow | 역할 |
| --- | --- |
| `release.yml` | 5개 target build, runtime 검증, digest, attestation, npm tarball staging |
| `release-publish.yml` | exact candidate와 attestation 재검증, npm `test` publication |

Publication workflow의 Git tag·GitHub Release 생성 0건. Developer ID,
notarization, Authenticode, Azure signing, external TUF는 실제 안정 릴리스의
별도 hardened gate로 deferred.

## 실행 순서

완료: `P7-046` 영·한 README, `P7-047` bilingual setup, `P7-043` Linux musl
x86_64·arm64 qualification, `P7-049` 설치 소유자 기반 대화형 `hive update`.

0. `release-publication` 필수 검토자 설정 확인
1. `P7-044`: npm package family와 native smoke
2. `P7-045`: npm-backed Unix·PowerShell·CMD installer와 authenticated owner receipt
3. `P7-020`: archive·npm tarball SHA-256·attestation·byte identity
4. `P7-018`: protected `develop` exact `0.8.0` product candidate qualification
5. `P7-037`: npm `0.8.0-test.1`의 `test` publication과 npm·curl clean install 검증
6. 시험 배포 성공 commit의 `develop` → `main` PR 병합

`P7-049` 선행 조건: `P7-044`·`P7-045`의 exact-version package와 authenticated
install-owner adapter 확정. 불확실한 owner 추측과 설치 관리자 우회 binary
직접 overwrite 금지.

## Candidate branch authority 경계

- 사용자 순서: 시험 배포 성공 뒤 `develop` → `main` 병합
- Workflow authority: protected `develop` candidate 한정
- Current ruleset: `develop`에 PR·필수 검사·삭제·강제 push 차단 적용
- Current publication environment gap: `release-publication` 필수 검토자 없음
- 사용자 작업: `release-publication`에 필수 검토자 추가
- 자동화 작업: workflow의 exact `develop` ref·commit·run 검증
- 환경 검토자 확인 전 P7-037 publication 중단

## 완료 기준

- 5개 target artifact와 expected architecture 일치
- Candidate archive와 npm-installed binary의 SHA-256 byte identity
- npm exact/test와 curl·PowerShell·CMD clean install·repeat update·recovery PASS
- GitHub Release·prerelease·release tag 0건, npm `latest` 이동 0건
- Auto check가 24시간 success throttle과 offline next-session retry를 만족
- Bare `hive update`가 확인·질문·동의·설치·재검증 순서를 만족
- English·Korean initial setup과 global guidance byte fixture PASS
- Consumer runtime의 Node.js·PowerShell 7 dependency 0건
- Exact product `0.8.0`과 npm package `0.8.0-test.N` 결합 검증,
  clean-clone 전체 CI PASS
- Codex·Antigravity 실제 host 회귀 PASS
- Claude Code 미검증 상태와 signing deferred 범위 공개

## 외부 중지 경계

- npm 로그인·2FA·organization, 최초 등록용 임시 token과 6개 Trusted Publisher
- `release-publication` 필수 검토자 설정과 publication 승인
- Credential·private signing material 접근 금지
- npm `latest`, GitHub Release, 안정 릴리스는 별도 사용자 승인 전 금지
- Exact `1.0.0` 별도 명시 전 major release 준비 금지

## 후속 안정 릴리스

- 사용자가 승인할 안정 `0.8.x` exact version 결정
- npm `latest` 이동과 기본 install 명령 전환
- GitHub normal release 필요 여부 재승인
- macOS Developer ID signing·notarization
- Windows Authenticode 또는 Azure Artifact Signing
- External TUF 2-of-3 production authorization
- 실제 Claude Code subscription session qualification
