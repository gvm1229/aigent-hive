# Aigent Hive

> Codex, Claude Code, Gemini Antigravity를 위한 provider-neutral 로컬 harness.

[![Version](https://img.shields.io/badge/version-0.7.0-4C1)](../../Cargo.toml)
[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust)](../../rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](../../LICENSE)

[English](../../README.md) · [한국어](./README.ko.md)

Hive: subscription 인증 agent host에 일관된 setup, Skill routing, project knowledge,
지속 가능한 role/run 상태, usage safeguard와 안전한 update 계약 제공.
Model-provider API key 요청·provider API 호출·host model runtime 대체 없음.

## 0.8.0 시험 배포

제품 후보 버전 `0.8.0`과 npm 시험판 버전 `0.8.0-test.N` 분리.
npm `test` 태그로만 배포하며 GitHub Release 생성·npm `latest` 이동 없음.

시험 배포가 완료된 뒤 설치:

```console
npm install -g aigent-hive@0.8.0-test.1
```

또는:

```console
npm install -g aigent-hive@test
```

npm 설치 dependency: Node.js·npm. 설치된 `hive` runtime: native Rust binary,
Node.js dependency 없음.

### macOS·Linux curl

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://unpkg.com/aigent-hive@0.8.0-test.1/install.sh | sh
```

### Windows PowerShell 5.1+

```powershell
irm https://unpkg.com/aigent-hive@0.8.0-test.1/install.ps1 | iex
```

### Windows 명령 프롬프트

```bat
curl.exe -fLo install-aigent-hive.cmd https://unpkg.com/aigent-hive@0.8.0-test.1/install.cmd && install-aigent-hive.cmd
```

직접 installer: npm의 동일 native package bytes 수신, embedded exact-version
SHA-256 검증, direct-install ownership receipt 기록. npm·Node.js·PowerShell 7
dependency 없음.

## 지원 target

| Platform | Native target | 0.8.0 gate |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | Candidate runtime 검증 |
| macOS Intel | `x86_64-apple-darwin` | Candidate runtime 검증 |
| Linux x86_64 | `x86_64-unknown-linux-musl` | Release qualification 진행 중 |
| Linux arm64 | `aarch64-unknown-linux-musl` | Release qualification 진행 중 |
| Windows x86_64 | `x86_64-pc-windows-msvc` | Candidate runtime 검증 |

Codex·Antigravity는 실제 host 증거가 있음. Claude Code package·projection은 fixture로
검증했지만 실제 subscription-backed session은 미검증. macOS notarization과 Windows
code signing은 후속 안정 릴리스로 deferred.

## 첫 설정

사용하는 host에 Hive 설치:

```console
hive install --scope user --host codex --apply --output json
```

필요하면 `codex`를 `claude` 또는 `antigravity`로 변경. 이후 host에 Aigent Hive
설정을 요청. 첫 선택은 `English` 또는 `한국어`이며, 나머지 모든 설정 질문과 global
Hive 지침은 선택 언어 사용.

Setup 질문: 일 1회 update 확인 허용 여부. 명시적 opt-in이며 자동 확인 범위는
새 version 알림, 설치 0건.

Project에서는 현재 repository에 Hive setup을 요청. Hive가 소유할 exact write set을
미리 보여 주고 foreign guidance bytes를 보존하며 canonical knowledge는 Markdown으로
유지.

## 업데이트

```console
hive update
```

즉시 version 확인. 새 version이 있으면 exact update 내용을 설명하고 authenticated
install owner를 실행하기 전에 질문. 거절·stdin 종료·noninteractive 실행에서는 설치
mutation 0건.

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
- [제품 결정](../decisions/product-release-decisions.md)

개발 dependency: Rust stable, conformance test용 Python 3.13, Windows 개발·release
workflow용 PowerShell 7. Consumer install dependency: Python·PowerShell 7 없음.

```console
python scripts/dev-check.py pre-push
```

## QA Contributors

| 이름 | GitHub | 검증 platform·영역 |
| --- | --- | --- |

## 라이선스

Apache-2.0. [LICENSE](../../LICENSE) 참고.
