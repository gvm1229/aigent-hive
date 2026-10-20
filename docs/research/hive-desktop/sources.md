# 공식 근거와 적용 범위

- 공통 접근일: **2026-10-14**, 한국 시간
- 자료 선택: 제품·규약 제공자의 공식 문서와 유지관리자의 공개 코드
- 날짜 미확인 표기: 이번에 읽은 자료에서 개정 날짜의 확정 없음. 오래된 날짜로 현재 지원을 추정하는 처리 제외
- `latest` 표기: 공식 문서의 주소·대상 명칭. 정확한 패키지 번호·설치 상태의 별도 확인 필요
- 근거의 한계: 제공자의 문서상 기능 확인; Hive 통합·실제 인증·성능·운영 지원 자격의 시험 증명 없음
- 상충 자료: 현재 읽은 공식 자료 우선. 오래된 구독 연결 허용 사례·WebDriver 제한 설명을 현재의 보편적 사실로 사용하지 않는 기준

## 화면·실행 환경

| 자료·직접 위치 | 게시·개정일 | 적용 버전·범위 | 확인한 사실·제안과의 구분 |
| --- | --- | --- | --- |
| [Tauri Architecture](https://v2.tauri.app/concept/architecture/) | 미확인 | Tauri 2 | Rust·WebView 구조; Hive 기본 후보 선택은 조사 권고 |
| [Tauri Process Model](https://v2.tauri.app/concept/process-model/) | 2025-02-22 | Tauri 2, Windows·macOS·Linux | WebView2·WKWebView·WebKitGTK와 플랫폼 차이 |
| [Announcing Tauri 2.12](https://v2.tauri.app/blog/tauri-2.12/) | 2026-09-26 | Tauri 2.12 | 최소 Rust 1.90, 공식 Windows 7 지원 종료 |
| [Tauri Capabilities](https://v2.tauri.app/security/capabilities/) | 2026-09-03 | Tauri 2 | 화면의 시스템 접근 제한, Rust 내부 안전의 별도 책임 |
| [Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-frontend/) | 미확인 | Tauri 2 | Channels의 순서 있는 데이터 전달; Hive 진행 표시 제안 |
| [Embedding External Binaries](https://v2.tauri.app/develop/sidecar/) | 미확인 | Tauri 2 | 별도 실행파일 포함·플랫폼별 배포 방식 |
| [Tauri Updater](https://v2.tauri.app/plugin/updater/) | 미확인 | Tauri 2 갱신 플러그인 | 서명된 앱 갱신, Hive 지침 갱신과 별도 통합 필요 |
| [Tauri WebDriver](https://v2.tauri.app/develop/tests/webdriver/) | 미확인 | 조회 당시 문서 | 내장 WebDriver의 세 OS 지원, 직접 tauri-driver의 Windows·Linux 제한 |
| [Electron Process Model](https://www.electronjs.org/docs/latest/tutorial/process-model) | 미확인 | `latest` 문서 | Chromium 기반 여러 프로세스·메인과 화면의 책임 |
| [Electron Security](https://www.electronjs.org/docs/latest/tutorial/security) | 미확인 | `latest` 문서 | 화면의 권한·프로세스 격리; Rust 보조 실행파일 연결은 Hive 설계 제안 |
| [Flutter Desktop](https://docs.flutter.dev/platform-integration/desktop) | 미확인 | 조회 당시 문서 | Windows·macOS·Linux 기본 앱 지원; Hive 연결 비용은 조사 판단 |

## 실행 엔진·연결 규약

| 자료·직접 위치 | 게시·개정일 | 적용 버전·범위 | 확인한 사실·제약 |
| --- | --- | --- | --- |
| [Codex App Server](https://learn.chatgpt.com/docs/app-server) | 미확인 | 조회 당시 App Server 문서 | 대화·승인·중단·계정의 JSON 통신, 실험 단계·운영 지원 제한, 인증 적용 범위 |
| [Codex Authentication](https://learn.chatgpt.com/docs/auth) | 미확인 | Codex 로컬·클라우드 제품 | ChatGPT 구독과 API 인증의 구분; 제삼자 앱의 허용 범위 일반화 금지 |
| [Sign in with ChatGPT](https://developers.openai.com/siwc/token-sharing-open-source) | 미확인 | ChatGPT 계획 사용 경로 | 앱별 인증 연결의 대안 후보; Hive 현행 정책과 별도 검토 |
| [Codex App Server와 계획 사용](https://developers.openai.com/siwc/token-sharing-open-source/codex-app-server) | 미확인 | 해당 토큰 공유 경로 | 앱의 토큰 갱신·재시작·대화 재개 책임 |
| [Preview limitations](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations) | 미확인 | 해당 미리 보기 경로 | 도구·입력·통신 제한; Codex 앱 전체 기능 이전 보장 없음 |
| [Claude Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview) | 미확인 | Python·TypeScript Agent SDK | 도구·실행 반복·문맥 관리 제공 |
| [Claude Agent SDK quickstart](https://code.claude.com/docs/en/agent-sdk/quickstart) | 미확인 | 조회 당시 SDK 인증 조건 | 사전 승인 없는 제삼자 제품의 claude.ai 로그인·사용량 한도 제공 불허 |
| [API credits for Max and Team plans](https://platform.claude.com/docs/en/about-claude/api-credits-for-subscribers) | 미확인 | 해당 Max·Team 계획 | API 크레딧과 Claude Code 구독 사용량의 별도 적용 |
| [Hermes Desktop](https://hermes-agent.nousresearch.com/docs/user-guide/desktop) | 미확인 | 조회 당시 데스크톱 문서 | Electron·React·별도 hermes serve 실행 엔진 구조 |
| [Hermes Architecture](https://hermes-agent.nousresearch.com/docs/developer-guide/architecture) | 미확인 | 조회 당시 엔진 구조 | Python 실행 반복·도구·문맥·SQLite·ACP의 역할 |
| [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent) | 조회 당시 기본 브랜치, 고정 커밋 미확인 | 공개 저장소 | MIT 표시; Hive 코드 이식·라이선스 전체 검토 미실행 |
| [MCP 소개](https://modelcontextprotocol.io/docs/2026-07-28/getting-started/intro) | 규약 경로 2026-07-28 | 해당 MCP 문서 | AI와 도구·자료의 연결 규약 |
| [ACP Introduction](https://agentclientprotocol.com/get-started/introduction) | 미확인 | 조회 당시 규약 | 편집기·에이전트 연결, 로컬 표준 입출력·JSON-RPC, 원격 지원 개발 중 |

## 화면 구성 요소

| 자료·직접 위치 | 개정일·버전 | 적용 |
| --- | --- | --- |
| [React 앱 생성](https://react.dev/learn/creating-a-react-app) | 날짜·정확한 패키지 번호 미확인 | 화면 작성 방식 참고; Tauri 내부 화면 조합은 조사 제안 |
| [Vite 시작](https://vite.dev/guide/) | 날짜·정확한 패키지 번호 미확인 | 개발·화면 묶음 도구 후보 |
| [Monaco Editor](https://github.com/microsoft/monaco-editor) | 기본 브랜치, 고정 커밋 미확인 | 코드 편집 후보, VS Code 확장 직접 실행 불가 |
| [xterm.js](https://xtermjs.org/) | 날짜·정확한 패키지 번호 미확인 | 터미널 표시 후보; 실제 셸 연결 별도 |
| [portable-pty](https://docs.rs/portable-pty/latest/portable_pty/) | `latest`, 정확한 고정 버전 미확인 | Rust 터미널 연결 후보 |
| [Tokio CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html) | `latest`, 정확한 고정 버전 미확인 | 비동기 작업의 취소 전달 후보; 외부 프로세스 중단 확인 별도 |

## 저장소 관찰의 출처

- [코드 관찰·변경표](changes.md)의 정확한 파일·식별자
- [현재 제품 계획](../../plans/PLAN.md): 완료된 `0.12.0`, 다음 버전·공개 시험 미지정
- [제품 목적](../../overview/product.md): 호스트 실행과 Hive의 결정적 관리 책임
- [실행 소유권](../../../.agents/directives/02-architecture.md): 모델·프로세스·인증 경계
- 줄 수: Windows PowerShell의 현재 소스 집계; 추정 기간의 근거 자료 중 하나, 수정 비율의 계산 근거로 사용 제외

제공자별 기능·인증은 각 제공자의 공식 설명이라는 단일 권위 근거. 서로 다른 엔진 문서의 일치는 Hive에서의 실제 상호 교환 가능성 증명과의 구분.
