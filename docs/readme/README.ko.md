# Aigent Hive

<p align="center">
  <img src="../assets/branding/hive-readme-banner-ko.png" alt="hive — 모든 프로젝트를 위한 지속적 맥락" width="100%">
</p>

> Codex, Claude Code(검증 전), Gemini Antigravity를 위한 provider-neutral 로컬 harness.

[![Version](https://img.shields.io/badge/version-0.16.0-4C1)](../../Cargo.toml)
[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust)](../../rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](../../LICENSE)

[English](../../README.md) · [한국어](./README.ko.md)

<!-- AIGENT-HIVE:PUBLIC-STABLE version=0.16.0 release-date=2026-11-23 -->

Hive: subscription 인증 agent host에 일관된 setup, Skill routing, project knowledge,
지속 가능한 role/run 상태, usage safeguard와 안전한 update 계약 제공.
Model-provider API key 요청·provider API 호출·host model runtime 대체 없음.

현재 stable `0.16.0`: npm `latest`, normal GitHub Release, annotated Git tag 배포.

## 현재 stable 설치

npm `0.16.0|latest`, GitHub normal Release, annotated Git tag 배포.

기본 설치:

```console
npm install -g aigent-hive
```

또는 exact version 고정:

```console
npm install -g aigent-hive@0.16.0
```

npm 설치 dependency: Node.js·npm. 설치된 `hive` runtime: native Rust binary,
Node.js dependency 없음.

예상 stable version label:

```text
AIgent Hive v0.16.0 (released 2026-11-23)
```

### macOS·Linux curl

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://unpkg.com/aigent-hive@0.16.0/install.sh | sh
```

### Windows PowerShell 5.1+

```powershell
irm https://unpkg.com/aigent-hive@0.16.0/install.ps1 | iex
```

### Windows 명령 프롬프트

```bat
curl.exe -fLo install-aigent-hive.cmd https://unpkg.com/aigent-hive@0.16.0/install.cmd && install-aigent-hive.cmd
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

Install the current stable release 0.16.0. The stable install guidance is
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

## 0.16.0 주요 변경

- 공개 0.15.x의 Claude Code·Antigravity 설치를 `hive update`가 그대로 인식. 재설치가 필요해도 프로젝트 목록·실행 상태·백업·claims 보존. 사용자 답변·지식·프로젝트 파일 보존
- 설치·갱신·제거 전에 설정·프로젝트 목록·바뀔 파일을 `~/hive-backups`에 복사(`~/.claude/settings.json` 포함). `hive storage backup list`로 확인, `hive storage backup restore --id <id>`로 복원 미리보기. 자동 삭제 없음. 바뀌지 않는 지식·기록과 `hive setup` 변경은 백업 범위 밖
- Claude는 사용한 %, Codex·Antigravity는 남은 %로 표시와 입력. 예: `hive usage threshold --user-root <user-root> --host claude --window weekly --stop-used-percent 96`은 96% 사용에서 중단
- 사용량을 읽지 못하면 빠진 한도·원인·해결 방법 경고 뒤 작업 계속. 측정 실패의 잘못된 Discord 회복 안내 제거
- `hive setup --scope user --section usage --claude-statusline preview`로 Claude Code 상태 표시줄 연결을 미리 보고 승인. 기존 상태 표시줄 유지, 되돌리기 가능. 데스크톱 앱 Code 탭은 미확인, 사용량이 보이지 않으면 터미널의 Claude Code 사용
- 실험 기능 `hive msg`: 프로젝트 파일 수신 상자로 작성·검토·자문 역할의 일과 확인증 전달, 프로젝트 훅·결정 대기열·상태 화면. `--reviewer-provider`로 검토 공급자 선택. Hive의 작업자 실행 없음. 훅 상태와 호스트의 실제 실행 증거는 별개
- 이 버전의 Codex 훅·작업자·교차 검토는 미검증. 적용 뒤 Codex의 프로젝트 훅 신뢰 승인 필요. Claude 자동 모드의 `hive msg` 차단 가능성, 프로젝트 권한 규칙은 사용자 선택. Gemini 훅은 미리보기만
- `hive stop`으로 새 메시지 중지. 작업자 한도는 `max_workers`, 옛 `max_panes` 값 이전, 값 충돌 시 변경 중지. `hive route`·`hive prompt`는 `hive advanced` 아래로 이동
- 내장 Skill을 `knowledge-manage`·`run`·`judge`로 통합. 설명에 옛 이름 표기, Claude Code의 옛 `/aigent-hive:...` 명령 실행은 미지원. 사용자가 바꾼 파일 보존
- Claude Code에서 플러그인 훅이 요청마다 Hive 지식을 찾아 상위 결과를 함께 전달. Agent가 조회를 빠뜨려도 저장된 선호가 전달되며, 아무것도 쓰지 않고 결과가 없으면 추가 없음. 사용자 지침의 조회 조건을 잡담 외 모든 작업으로 명확화, 읽기 전용 작업도 포함
- Windows의 `--target .` 지원
- Antigravity 사용량 콜백은 1.1.18 이상 2.0 미만의 같은 자료 구조만 수용. 1.2 실제 호스트 미검증
- 0.15.x로 내리기 전 상태 표시줄 연결 되돌리기 필요. 0.15.x 재설치의 `~/.hive/backups`·프로젝트 목록 삭제

## 이전 0.15.3 변경

- AI에게 프로젝트의 Hive 설정을 맡기기 지원. project-setup Skill을 부르면 AI가 직접 설정: 빈 폴더는 프로젝트 내용을 한 번에 하나씩 질문, 기존 프로젝트는 파일을 먼저 살펴본 뒤 한 번의 제안 확인 후 적용
- AI의 프로젝트 설정 미리보기·적용·검증에 능력 파일 불필요. `hive setup --answers`가 터미널 설정과 같은 저장·감지 능력 사용, 감지한 능력은 미검증 표시·Hook 미허용
- 설정이 대기 상태로 멈추면 AI가 이어 갈 명령(`--show --summary`, 이어서 에이전트 `--dry-run`) 안내. 사람용 터미널 재개 명령도 함께 표시
- 터미널 설정 화면 가독성 개선: 질문마다 `[2/4] 프로젝트 이름` 같은 번호 붙은 제목, 미리보기는 구역별 표. 색은 `NO_COLOR` 없는 대화형 터미널에서만
- 새로 설정한 프로젝트에서 `hive project upgrade --validate` 통과. 실제로 바뀌는 파일은 경로마다 사유 표시
- `hive knowledge remember --batch <facts.json>`으로 한 턴의 여러 사실을 한 번에 기록. 하나라도 거부되면 전체 취소
- 사실마다 `design/visual-style` 같은 주제 지정, 맞은 사실과 같은 주제의 사실 최대 16개를 함께 반환. `hive knowledge subjects list|preview|apply|restore`로 주제 관리
- `남색`으로 `남색과`를 찾는 것처럼 조사가 붙은 저장 문장도 짧은 한국어로 검색. 회수 Skill이 `--expansions`로 한·영 바꿔 쓴 말 함께 전달
- 주제가 붙은 사실이 생기면 Hive 0.15.2 이하가 그 지식 저장소를 거부, 하위 버전 되돌림 미지원
- 사용량 보호가 고른 앱마다 구독의 5시간 한도 여부를 묻고, 5시간만·주간만·둘 다(기준 각각) 중 보호할 한도 선택. Antigravity는 기준 하나, 새 앱별 기준은 전역 기준에서 시작. Codex의 30일 사용량 창은 주간 한도로 측정
- 잘못 입력한 명령·옵션에 AI 없이 가까운 추천 최대 3개(예 `hive instal` → `install`). `hive --help`는 짧은 묶음별 목록
- `npm install -g`로 새 버전을 이전 공개 버전 위에 설치한 뒤 `hive update` 정상 진행. 재설치 실패 시 Hive 파일 보존과 복구 명령 안내
- 설정 마지막 확인의 `아니요`는 실패 대신 변경 없음과 이어가기 명령 표시. JSON은 같은 코드·종료 코드 2에 `data.declined` 추가. 긴 설정 문장은 80열 안에서 줄바꿈, 느린 앱 확인 앞 안내
- Hive 사용자 지침에 지속 선호를 같은 주제의 원자 사실로 나눠 저장하는 규칙. `hive setup --describe --question`이 `project_kind`를 `project-kind`로 해석하고 가까운 질문 이름 추천

## 이전 0.15.2 변경

- 한 번의 설정으로 Codex·Claude Code·Antigravity에 함께 설치. 앱 질문에 여러 개 선택 안내 표시, `1,2`처럼 번호나 이름을 쉼표나 공백으로 구분해 입력
- 지워진 폴더를 가리키던 `aigent-hive` 등록을 정리하고 Claude Code CLI·앱에 Hive 플러그인과 Skill 설치
- Antigravity CLI 1.2.x 지원
- `hive update` 재개 시 "monitor choice changed since setup preview" 반복 해결. 0.15.1에서 이미 반복 중인 PC는 `npm install -g aigent-hive@0.15.2` 뒤 `hive update`
- 첫 설치 실패 시 파일 복원 전 Codex 등록 정리, Codex 플러그인 목록 유지. 이전 실패로 깨진 등록은 갱신 시 복구
- 연결한 지식 폴더의 색인 동의가 필요하면 설치·갱신 완료 뒤 다음 단계로 `hive index rebuild --user-root <홈>` 안내

## 이전 0.15.1 변경

- 명령 결과를 기본값으로 쉬운 한국어·영어 문장과 원인·다음 행동으로 표시. 프로그램이 읽을 때만 `--output json` 추가
- 프로젝트 설정에서 모델 호출 없이 언어·프레임워크·패키지 관리자·명령·구조를 로컬 감지, 한 번의 확인으로 제안 적용
- 프로젝트 사실과 짧은 Hive 입구 분리, Hive 표시 밖의 사용자 규칙 보존
- 압축 배치를 고른 프로젝트만 검증된 Hive 플러그인의 공용 Skill 재사용, 그 밖의 프로젝트는 로컬 Skill 사본 유지
- AI가 자체 기억에 남길 사실을 Hive Markdown 지식에 기록. 프로젝트 우선, 여러 프로젝트에 적용되는 사실만 전역 지식
- 신 입구 9개를 포함한 모든 Skill 설명에 하는 일 표시
- 공개 0.15.0·0.14.x에서 인증된 과거 원본으로 보존 갱신. 지원 JSON 스키마·값·종료 코드 유지

## 이전 0.15.0 변경

- 계획·조사·구현·글쓰기·설정·조율·지식·사용량·정리의 신 입구 9개와 한글 이름·별칭 추가, 기존 명령·Skill 경로 보존
- 전역 기본값과 프로젝트별 역할의 모델·작업 수준 설정 추가, `hive roster`로 합친 구성 확인·기존 고정 모델 보존
- `hive summon`의 작업 요청 미리보기·서명된 수신 상자 준비 추가, Hive의 작업자 실행 없음. 이번 버전의 조율 비활성·활성화 경로 없음
- 공개 0.14.2의 프로젝트·사용자 원문을 이용한 보존 갱신

## 이전 0.14.2 변경

- 갱신 뒤와 `hive prune --confirm`에서 1 MiB 초과·128 MiB 이하인 오래된 감시 프로그램 사본 정리 복구
- 현재·직전 사본 보관, Windows 잠금 파일과 정확한 바이트 검증 실패 파일 보존
- 감시 프로그램 사본의 기존 128 MiB 입력 한도 유지


## 이전 0.14.1 변경

- 전역 사용량 하한 변경 때 저장된 선택 기록의 유효성 유지와 변경 성공
- 직접 해제한 지식 연결의 해제 유지, 동기화·재구성의 재연결 없음. 사용 전 명시적 재연결 필요
- 갱신 뒤 현재와 직전 감시 프로그램만 보관, 오래된 사본 정리
- `hive storage`의 저장 공간 보고, `hive prune --dry-run`의 미사용 감시 프로그램 미리보기. 삭제는 `hive prune --confirm`
- 공개 0.14.0에서의 갱신도 전체 파일 소유 검증 유지. 연결 해제 뒤 0.14.0의 연결 목록 읽기 불가, 이전 버전 복귀 전 재연결 필요

## 이전 0.14.0 변경

- 모델 호출 없이 한국어·영어의 터미널 질문으로 전역 환경과 프로젝트 설정
- 저장한 선택 재사용·한 항목 변경·중단한 질문 재개, 사용자 글·역할 보존
- 새 설치의 사용량 자동 중단 기본 비활성, 기존 보호 변경 전 선택 확인
- `hive uninstall --dry-run`으로 제거·보존 대상 확인, 프로그램 제거 뒤에도 설정·지식·기록·프로젝트 자료 보존과 재설치 지원
- 보호 활성 시 남은 사용량0~99% 하한 선택. 초기화·할당량 소진·실행 중 감시의 개별 제어.0%에서도 소진 보호 유지, 크레딧 차감0 보장 제외
- 0.13.1 갱신에서 새 터미널 질문까지 연결. 질문할 수 없는 환경의 승인된 보존 갱신은 기존 선택 유지·새 선택형 기능 비활성·남은 질문 안내. 과거 설치도 전체 파일 소유 검증 유지
- 내려받기·후크·외부 알림·Windows Codex 실행 중 중단의 별도 동의 유지. 앱의 필요한 통신 기능 확인, 앱 유지·직접 재개 지원. 다른 운영체제·원격·클라우드 직접 중단 미지원
- 기존 답안 파일 지원 유지, 직접 선택·기본값·상속·과거 출처 불명의 구분

이전 개선 사항은 [0.13.1 출시 안내](../releases/0.13.1.md)·[0.13.0 출시 안내](../releases/0.13.0.md)·[0.12.0 출시 안내](../releases/0.12.0.md)에 보존.

## 지원 target

| Platform | Native target | 0.16.0 근거 |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | 공개 번호 시험판 수용 |
| macOS Intel | `x86_64-apple-darwin` | Candidate runtime 검증 |
| Linux x86_64 | `x86_64-unknown-linux-musl` | 공개 번호 시험판 수용 |
| Linux arm64 | `aarch64-unknown-linux-musl` | native 후보 검증 |
| Windows x86_64 | `x86_64-pc-windows-msvc` | 공개 번호 시험판 수용 |

공개 시험의 Windows x64·Linux x64·macOS arm64에서 0.15.3·0.13.1 갱신, 제거, 재설치 수용(대역 Antigravity 호스트).
실제 Codex·Claude Code·Antigravity 설치 갱신은 Windows 11 PC 1대에서 확인. macOS·Linux 실제 호스트 갱신과
모든 호스트의 데스크톱 모델·할당량 조회 증명 제외. Claude Code 사용량 기록은 Windows 터미널 세션에서 확인,
그 밖 구독 세션 동작 미검증. Stable `0.16.0`: macOS ad-hoc signing,
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

`hive update`의 터미널 질문에 답한 뒤 적용 전 변경 내용 확인. 의미 검색처럼 저장소·다운로드·향후 동작에 영향을 주는 선택은 별도 확인. 취소는 동의로 처리하지 않고, 질문할 수 없는 보존 갱신은 기존 선택 유지와 남은 질문 안내.

확인 뒤 전역 사용자 파일 적용·검증, 프로젝트 설정은 별도 작업 유지.

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
명시한 갱신 승인 없는 거절·입력 종료·비대화형 실행의 설치 변경0건. 최초 초기화에는 대화형 host 선택 필요.

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

## 명령 결과 표시

- 한국어·영어 설정에 맞춘 결과·변경 전후·다음 행동 안내
- 사람용 명령에서 JSON 인자 생략, 프로그램·스킬·훅의 기계 판독에는 `--output json` 선택
