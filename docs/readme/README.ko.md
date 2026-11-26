# Aigent Hive

<p align="center">
  <img src="../assets/branding/hive-readme-banner-ko.png" alt="hive — 모든 프로젝트를 위한 지속적 맥락" width="100%">
</p>

> Codex, Claude Code(검증 전), Gemini Antigravity를 위한 provider-neutral 로컬 harness.

[![Version](https://img.shields.io/badge/version-0.16.1-4C1)](../../Cargo.toml)
[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust)](../../rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](../../LICENSE)

[English](../../README.md) · [한국어](./README.ko.md)

<!-- AIGENT-HIVE:PUBLIC-STABLE version=0.16.1 release-date=2026-11-26 -->

Hive: 지금 쓰는 AI 코딩 앱에 같은 설정, 프로젝트 지식, Skill 연결, 사용량 보호, 안전한 업데이트 제공.
모델 회사 API 키 요청·API 호출·앱의 모델 실행 대체 없음.

현재 stable `0.16.1`. 버전별 변경 내용은 [GitHub Releases](https://github.com/gvm1229/aigent-hive/releases) 참고.

## 현재 stable 설치

```console
npm install -g aigent-hive@0.16.1
```

npm 설치에만 Node.js·npm 필요, 설치된 `hive`는 독립 실행 파일. npm 없이 설치할 때는 셸별 직접 설치:

```sh
# macOS·Linux
curl --proto '=https' --tlsv1.2 -LsSf https://unpkg.com/aigent-hive@0.16.1/install.sh | sh
```

```powershell
# Windows PowerShell 5.1 이상
irm https://unpkg.com/aigent-hive@0.16.1/install.ps1 | iex
```

```bat
:: Windows 명령 프롬프트
curl.exe -fLo install-aigent-hive.cmd https://unpkg.com/aigent-hive@0.16.1/install.cmd && install-aigent-hive.cmd
```

직접 설치는 해당 버전의 SHA-256 확인, npm·Node.js·PowerShell 7 불필요. 설치 뒤 `hive --version` 결과:

```text
AIgent Hive v0.16.1 (released 2026-11-26)
```

## 첫 설정

1. 대화형 터미널에서 `hive update` 실행, 쓰는 AI 앱 선택과 설정 질문 답변. 적용 전 변경 미리보기, 프로젝트 변경 없음
2. AI 앱에서 프로젝트를 열고 프로젝트마다 한 번 아래 요청 붙여 넣기:

   ```text
   Configure the local Aigent Hive harness for this project. Use my existing global Hive preferences, inspect only this project, show the exact write preview, and ask me only about choices that require my approval.
   ```

   앱이 그 프로젝트에서 열려 있지 않으면 "this project" 자리에 프로젝트 절대 경로 사용

<details>
<summary>선택: 사용자 범위 설치 전체를 AI 앱에 맡기기</summary>

```text
I want the optional one-prompt Aigent Hive setup. Work only at user scope; do not inspect,
initialize, or change any project, repository, folder, or current working directory.

Install the current stable release 0.16.1. The stable install guidance is
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

</details>

## 업데이트

대화형 터미널에서 `hive update` 실행. 새 버전 확인, 정확한 갱신 내용 표시, 승인 뒤에만 설치.
거절·입력 종료·비대화형 실행은 설치 변경 없음. 몰래 업데이트 없음, 바꾸기 전 설정과 바뀔 파일을
`~/hive-backups`에 복사.

명령 결과는 읽기 쉬운 한국어·영어. 프로그램이 결과를 읽을 때만 `--output json` 추가.

## 다른 컴퓨터로 전역 지식 옮기기

`--user-root`는 `.hive` 폴더가 아닌 홈 폴더. 묶음 파일에는 옮길 수 있는 Markdown 지식만 포함하고
검색 색인·실행 상태·프로젝트 전용 지식·자격 증명·절대 경로는 제외. `--apply` 전에 SHA-256과
`--dry-run` 결과 확인.

```sh
# macOS·Linux: 내보낸 뒤 다른 컴퓨터에서 가져오기
hive knowledge export --user-root "$HOME" --scope global --bundle "$HOME/Downloads/hive.hivekb"
shasum -a 256 "$HOME/Downloads/hive.hivekb"
hive knowledge import --user-root "$HOME" --bundle "$HOME/Downloads/hive.hivekb" --dry-run
hive knowledge import --user-root "$HOME" --bundle "$HOME/Downloads/hive.hivekb" --apply
```

```powershell
# Windows PowerShell: 내보낸 뒤 다른 컴퓨터에서 가져오기
hive knowledge export --user-root $env:USERPROFILE --scope global --bundle "$env:USERPROFILE\Downloads\hive.hivekb"
Get-FileHash -Algorithm SHA256 "$env:USERPROFILE\Downloads\hive.hivekb"
hive knowledge import --user-root $env:USERPROFILE --bundle "$env:USERPROFILE\Downloads\hive.hivekb" --dry-run
hive knowledge import --user-root $env:USERPROFILE --bundle "$env:USERPROFILE\Downloads\hive.hivekb" --apply
```

## Automatic dispatch safeguard

자동 작업자 실행은 꺼져 있고 이 버전에는 켜는 방법 없음. 사용량 보호를 켜면 새 자동 실행 직전에
아래 사전 확인 실행. 이 확인만으로 실행 승인 없음:

```console
hive usage enforce --target <project> --session-id <id> --process-id <pid> --output json
```

## 지원 플랫폼

macOS(Apple Silicon·Intel), Linux(x86_64·arm64), Windows x86_64. macOS는 ad-hoc 서명, Windows는
서명 없음. 자세한 내용은 [코드 서명 정책](../guides/code-signing-policy.md), 버전별 검증 범위는
[릴리스 노트](https://github.com/gvm1229/aigent-hive/releases) 참고.

## Hive 소유 범위

Hive는 자기 표시 구간과 목록에 있는 파일만 변경, 지식은 Markdown·YAML·TOML 텍스트로 보관,
SQLite 색인은 다시 만들 수 있는 사본으로 취급. 소유 범위 밖: 모델 회사 자격 증명·모델 세션·다른 도구의 지침·
승인 없는 선택형 외부 Skill.

## 문서

- [문서 홈](../00-home.md) · [문서 색인](../01-index.md)
- [제품 개요](../overview/product.md) · [개발](../guides/development.md)

개발에는 Rust stable과 Python 3.13 필요. push 전 `python scripts/dev-check.py pre-push` 실행.

## QA 기여자

| 이름 | GitHub | 확인한 플랫폼·영역 |
| --- | --- | --- |
| 안희준 | [No-Jyun](https://github.com/No-Jyun) | Windows x64 설치·설정 검증 |

## 라이선스

Apache-2.0. [LICENSE](../../LICENSE) 참고.
