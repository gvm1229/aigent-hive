# Aigent Hive

<p align="center">
  <img src="../assets/branding/hive-readme-banner-ko.png" alt="hive — 모든 프로젝트를 위한 지속적 맥락" width="100%">
</p>

> Codex, Claude Code, Gemini Antigravity를 위한 provider-neutral 로컬 harness.

[![Version](https://img.shields.io/badge/version-0.10.3-4C1)](../../Cargo.toml)
[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust)](../../rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](../../LICENSE)

[English](../../README.md) · [한국어](./README.ko.md)

<!-- AIGENT-HIVE:PUBLIC-STABLE version=0.10.3 release-date=2026-09-10 -->

Hive: subscription 인증 agent host에 일관된 setup, Skill routing, project knowledge,
지속 가능한 role/run 상태, usage safeguard와 안전한 update 계약 제공.
Model-provider API key 요청·provider API 호출·host model runtime 대체 없음.

현재 stable `0.10.3`: npm `latest`, normal GitHub Release, annotated Git tag 배포.

## 현재 stable 설치

npm `0.10.3|latest`, GitHub normal Release, annotated Git tag 배포.

기본 설치:

```console
npm install -g aigent-hive
```

또는 exact version 고정:

```console
npm install -g aigent-hive@0.10.3
```

npm 설치 dependency: Node.js·npm. 설치된 `hive` runtime: native Rust binary,
Node.js dependency 없음.

예상 stable version label:

```text
AIgent Hive v0.10.3 (released 2026-09-10)
```

### macOS·Linux curl

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://unpkg.com/aigent-hive@0.10.3/install.sh | sh
```

### Windows PowerShell 5.1+

```powershell
irm https://unpkg.com/aigent-hive@0.10.3/install.ps1 | iex
```

### Windows 명령 프롬프트

```bat
curl.exe -fLo install-aigent-hive.cmd https://unpkg.com/aigent-hive@0.10.3/install.cmd && install-aigent-hive.cmd
```

직접 installer: npm의 동일 native package bytes 수신, embedded exact-version
SHA-256 검증, direct-install ownership receipt 기록. npm·Node.js·PowerShell 7
dependency 없음.

## 선택형 one-prompt 설정

Codex, Claude Code 또는 Gemini Antigravity에게 user-level 설치 전체 진행을 맡기려면 아래
아래 안내문 사용. 수동 경로를 대신하는 선택 사항.

```text
I want the optional one-prompt Aigent Hive setup. Work only at user scope; do not inspect,
initialize, or change any project, repository, folder, or current working directory.

Install the current stable release 0.10.3. The stable install guidance is
https://github.com/gvm1229/aigent-hive#install-the-current-stable-release.
Detect my operating system and active host (Codex, Claude Code, or Gemini Antigravity), asking
me if either is unclear. Check whether Node.js and npm are available. If they are missing,
give me the official OS-specific Node.js installation command and request any approval the host
requires before installing it. Then install the exact Hive release I selected using the official
method in the linked guidance, verify `hive --version`, then run `hive update` once and select
the active host when prompted.

Then begin interactive global setup in this conversation. For a first setup, ask only whether I
want English or Korean first; continue one question at a time. For existing settings, first ask
whether I want to change one setting or review everything. Do not start project setup afterward:
offer the separate project-setup prompt instead. Never ask for provider API credentials or install
an optional third-party Skill.
```

이 선택지는 현재 stable release만 설치.

## 0.10.3 주요 변경

- 실행 파일이 이미 최신이어도 인증된 전역 사용자 설정과 선택 호스트 투영을 맞추는 `hive update`
- npm 최초 설치 뒤 `hive update`에서 하나 이상의 호스트 선택, npm 자체 범위는 실행 파일 package로 유지
- 새 전역 질문에 답할 때까지 일반 Hive 작업 대기, 마지막 답 뒤 같은 digest-bound 갱신 자동 재개
- `0.9.5`부터 `0.10.1`까지 바로 이관하면서 외부·수정·project 파일을 transaction에서 제외
- 프롬프트 작성과 실제 구현 요청을 분리하고, 범위 안 조사 중 이미 승인된 작업 지속

## 지원 target

| Platform | Native target | 0.10.3 근거 |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | 공개 안정판 설치 수용 |
| macOS Intel | `x86_64-apple-darwin` | Candidate runtime 검증 |
| Linux x86_64 | `x86_64-unknown-linux-musl` | native 후보 검증 |
| Linux arm64 | `aarch64-unknown-linux-musl` | native 후보 검증 |
| Windows x86_64 | `x86_64-pc-windows-msvc` | 공개 안정판 설치 수용 |

Codex·Antigravity는 실제 host 증거가 있음. Claude Code package·projection은 fixture로
검증했지만 실제 subscription-backed session은 미검증. Stable `0.10.3`: macOS ad-hoc signing,
SignPath Foundation 무료 승인 전 Windows unsigned 공개. 정확한 경계는
[code signing policy](../guides/code-signing-policy.md) 참고.

## 첫 설정

먼저 실행 파일을 설치한 뒤 전역 갱신 명령 하나를 실행. project 설정은 project마다 별도 명시 작업 유지.

### 1. Hive CLI 설치

위 [현재 stable 설치](#현재-stable-설치) 중 한 가지 명령 사용. npm 설치 범위: `hive` command 제공;
명령만 제공. 사용자 설정·host 투영·project harness 생성 없음.

### 2. 전역 Hive 설치 초기화 또는 갱신

대화형 terminal에서 실행:

```console
hive update
```

최초 npm 설치면 질문이 표시될 때 하나 이상의 host 선택. Hive가 소유한 최소 사용자 투영만 설치.
기존 설치: 인증된 저장 호스트만 갱신·검증. 프로젝트 검사·변경 제외.

### 3. 새 전역 질문 답변

선택한 호스트 열기. 새 버전의 전역 선택은 일반 작업 전에 질문. 의미 검색처럼 저장소·다운로드·향후 동작에 영향을 주는 선택의 답변 확인 목적. `yes`·`no` 모두 유효한 답. 취소·무응답은 일반 Hive 작업 대기 유지.

마지막 답 뒤 Hive가 전역 사용자 투영을 자동 재적용·검증. `hive install` 또는 별도 setup 명령 직접 실행 불필요.

### 4. Project 한 개 설정

Host에서 정확한 project를 열고 아래 별도 prompt 입력:

```text
Configure the local Aigent Hive harness for this project. Use my existing global Hive preferences, inspect only this project, show the exact write preview, and ask me only about choices that require my approval.
```

Project마다 한 번씩 사용. Global preference 상속, exact write preview 후 해당 project만 변경.
Host에서 project open 불가 시 absolute path 명시:

```text
Configure the local Aigent Hive harness for the project at /absolute/path/to/project. Use my existing global Hive preferences, inspect only that project, show the exact write preview, and ask me only about choices that require my approval.
```

Home directory에서 path 없는 project prompt 사용 금지. 두 scope 동시 요청: global setup 완료 후
project inspection·change 전 별도 확인.

두 prompt 모두 update, optional third-party Skill, provider credential 접근 권한 포함 없음.

## 업데이트

```console
hive update
```

즉시 version 확인과 Hive 소유 전역 사용자 파일 수렴 수행. 실행 파일이 최신이어도 불완전한 사용자
설치를 복구. 새 version이면 exact update 내용을 설명하고 authenticated install owner 실행 전 질문.
거절·stdin 종료·noninteractive 실행에서는 설치 mutation 0건. 최초 초기화에는 대화형 host 선택 필요.

Daily check: 마지막 성공 확인부터 24시간 throttle. Offline·failed check는 성공
기록 제외; 다음 Codex·Claude Code·Antigravity session에서 재시도.

Silent update: 금지.

## Automatic dispatch safeguard

Enabled 상태에서는 새 automatic dispatch 직전에 subscription usage 확인:

```console
hive usage enforce --target <project> --session-id <id> --process-id <pid> --output json
hive run resume --dispatch-intent automatic --target <project> --run <run-id> --capabilities <json> --output json
```

첫 command: preflight only, dispatch 단독 승인 authority 없음. External runtime의
cancellation 결과는 보조 evidence이며 durable goal/task 상태 대체 불가. 일반 응답과
manual 작업: automatic-dispatch gate 적용 제외.

## 대규모 지식 기반 부하 검증

`chunk`: 전체 문서가 아닌 검색에 알맞은 크기로 나눈 지식 조각. 규모 예시: 의도적으로 길게
만든 Wiki page 25개를 각 2,000 chunk로 분할한 검색 대상 50,000개. Portable bundle 시험에는
portable collection registry 100개도 함께 포함. 50,000개 문서 수와의 동일시 금지; 일반 사용자
지식량 산정이 아닌 대규모 조건 안정성 확인용 fixture.

| 검증 항목 | 가정한 규모 | 이번 p95 | 통과 기준 |
| --- | --- | ---: | ---: |
| 처음 local 검색 | 50,000 chunk | 170 ms | 500 ms 이하 |
| 같은 내용 재검색 | 50,000 chunk | 0.14 ms | 100 ms 이하 |
| portable bundle export | collection 100개, chunk 50,000개 | 1.04 s | 5 s 이하 |
| bundle import·index rebuild | collection 100개, chunk 50,000개 | 3.27 s | 15 s 이하 |

배포용 build·local SSD 기준 시험. 실제 시간: hardware·지식 구성에 따른 차이 가능. 방법과
기준: [부하 검증 기록](../facts/ko/knowledge-portability-scan.md).

## Hive 소유 범위

- Hive marker block과 manifest에 기록된 파일
- Provider-neutral Skill과 얇은 host projection
- Canonical Markdown·YAML·TOML state
- Canonical text에서 재생성하는 disposable SQLite index
- 검증된 direct-install receipt

Provider credential, model session, foreign guidance, OMX·OMC state, Homebrew·WinGet
installation과 미승인 optional third-party Skill은 Hive 소유가 아님.

## Architecture·maintainer 문서

- [문서 홈](../00-home.md)
- [전체 문서 색인](../01-index.md)
- [제품 개요](../overview/product.md)
- [개발·검증](../guides/development.md)
- [Active plan](../plans/PLAN.md)
- [현재 project 상태](../state/CURRENT.md)
- [Source layout](../architecture/source-layout.md)
- [Release·update trust boundary](../architecture/release-update-trust-boundary.md)
- [Code signing policy](../guides/code-signing-policy.md)
- [제품 결정](../decisions/product-release-decisions.md)

개발 dependency: Rust stable, conformance test용 Python 3.13, Windows 개발·release
workflow용 PowerShell 7. Consumer install dependency: Python·PowerShell 7 없음.

```console
python scripts/dev-check.py pre-push
```

## QA 기여자

| 이름 | GitHub | 검증 환경·영역 |
| --- | --- | --- |
| 안희준 | [No-Jyun](https://github.com/No-Jyun) | Windows x64 설치·설정 검증 |

## 라이선스

Apache-2.0. [LICENSE](../../LICENSE) 참고.
