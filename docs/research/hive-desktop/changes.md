# 현재 코드와 변경량

## 실제 관찰

- [Cargo.toml](../../../Cargo.toml): 여섯 Rust 패키지, 제품 `0.12.0`, Apache-2.0
- [hive-cli/Cargo.toml](../../../crates/hive-cli/Cargo.toml): `hive` 실행파일, 별도 공개 라이브러리 대상 부재
- [main.rs](../../../crates/hive-cli/src/main.rs): 인자 해석·작업 연결·`ActionResult`·표준 출력 처리
- [run.rs](../../../crates/hive-cli/src/run.rs)의 `continuation_envelope`: `task_launch: host-owned`, `spawned: false`
- [codex_native.rs](../../../crates/hive-cli/src/usage/codex_native.rs)의 `exchange`: Codex 보조 프로세스의 `initialize`, `account/read`, `account/rateLimits/read` 호출
- 위 계정·사용량 감지는 **실제 존재하는 연결 코드**. 대화·도구 실행·장기 연결·중단·복구 구현의 증명과의 구분
- 제품 화면 구현 검색: Tauri·Electron·React·Flutter 앱 코드 미발견. 설치용 문서·npm 배포 파일과 제품 화면 코드의 구분

## 코드의 크기

Windows PowerShell에서 각 `crates/*/src/**/*.rs`의 파일 수와 물리적 줄 수 집계. 빈 줄·주석·파일 안 시험 포함, 별도 `tests/`·`vendor/` 제외. 실질 기능량·수정량·유지 가능 비율의 측정과의 구분.

| 패키지 | Rust 파일 | 물리적 줄 |
| --- | ---: | ---: |
| `hive-cli` | 43 | 71,731 |
| `hive-core` | 14 | 14,131 |
| `hive-projection` | 2 | 4,305 |
| `hive-render` | 3 | 12,174 |
| `hive-update` | 7 | 6,718 |
| `hive-wiki` | 16 | 37,146 |
| 합계 | 85 | 146,205 |

CLI 안의 코드가 전체 집계의 약 49.1%. 업무 처리의 독립성이 아직 부족하다는 조사 근거. 약 절반의 재작성 필요라는 의미 제외. 한꺼번에 71,731줄 이동보다 첫 앱 흐름에 필요한 기능의 순차 분리 추천.

## 유지·분리·추가

| 현재 위치·식별자 | 살릴 부분 | 필요한 변화 | 연결형 앱의 추정 규모 |
| --- | --- | --- | --- |
| `hive-core/src/orchestration.rs`: `ReducerState`, `HostReceipt`, `select_candidate`, `Lease` | 이벤트 검증·예산·중복 방지·작업 상태 | 실제 실행 엔진의 요청·결과를 검증된 Hive 상태에 연결 | 중규모 |
| `hive-core/src/run.rs`: `Host`, `HostSurface`, 작업 소유자·완료 조건 | 작업·역할·판단·소유권 고정 | 앱과 엔진의 식별 분리, 새 능력 증거와 호환성 | 중규모 |
| `hive-core/src/native_workflow.rs`: `Mailbox`, `Barrier`, `Goal`, `GoalBudget` | 작업 분담·대기·목표·예산 | 실행 관리의 실제 생명주기에 연결 | 중규모 |
| `hive-wiki/src/lib.rs`, `rag.rs`, `store.rs`, `shared.rs` | 정본 지식·검색·재생성·범위 보호 | 화면용 서비스, 긴 작업 진행·중단, 문서 수정 충돌 안내 | 소규모–중규모 |
| `hive-update/src/transaction.rs`, `release.rs`, `merge.rs` | 지침 갱신·무결성·복구·병합 | 앱 실행파일 갱신과 지침 갱신의 별도 책임·호환 계약 | 중규모 |
| `hive-projection/src/lib.rs`: `Host`, `compile_projection` | 기존 호스트별 스킬·지침 생성 | 기존 호스트 지원 보존, 자체 앱 설정의 새 출력 대상 | 소규모–중규모 |
| `hive-render/src/lib.rs`, `activation.rs` | 임시 저장·경로 소유권·안전 적용 | 화면에서 미리 보기·승인 후 동일 처리 호출 | 소규모–중규모 |
| `hive-cli/src/main.rs`: `ActionResult`, `emit_action_result` | 기존 JSON 결과·오류 코드 | 결과형·업무 함수와 인자·출력 처리 분리 | 중규모–대규모 |
| `hive-cli/src/run.rs`, `session.rs`, `user_setup.rs`, `knowledge/remember.rs` | 파일 검증·설정·승인·기록 흐름 | 새 `hive-app`으로 필요한 업무만 이동, 명시적 작업 경로·입출력·반환값 | 중규모–대규모 |
| `hive-cli/src/usage/codex_native.rs` | 실행파일 검증·제한된 JSON 입출력·시간 제한의 방식 | 별도 지속 연결, 답변 조각·승인 요청·중단·재연결 추가 | 새 연결 계층의 대규모 작업 |
| `harness/`, `schemas/` | 소비자 스킬·현재 계약·과거 기준본 | 데스크톱 선택·실행 엔진 계약 추가, 출시·마이그레이션 검증 | 중규모 |
| 새 `apps/desktop/`, `crates/hive-desktop/` | 해당 없음 | 모든 화면·창 관리·접근성·설치 구성 | 대규모 |
| 새 `crates/hive-runtime/` | 기존 상태 판단의 사용 | 지속 입출력·취소·작업 추적·복구·실행 엔진 연결 | 대규모 |
| 자체 실행 엔진 후보 | 기존 Hive 정책·지식·계획 | 모델 통신·문맥 관리·도구·안전한 실행 환경·비용 제어 | 매우 큰 규모 |

규모는 설계 판단이며 수정 줄 수의 확정치와의 구분. 다섯 라이브러리는 활용 가능한 기반, 전부 수정 없이 앱에 붙일 수 있다는 주장 제외.

## 가장 먼저 분리할 경계

1. CLI 출력과 무관한 결과형·오류형의 공통화. 현재 JSON 필드·오류 코드 보존
2. 첫 화면에 필요한 프로젝트 읽기·지식 검색·작업 상태 읽기의 서비스화
3. 함수 안의 현재 작업 폴더·환경 변수 접근을 명시적 요청 문맥으로 이동
4. 설정·지식·갱신의 미리 보기→사용자 승인→검증→적용 흐름 연결
5. 오래 걸리는 작업의 화면 스레드 분리와 진행·중단 처리
6. 선택한 실제 실행 엔진의 결과를 `HostReceipt` 검증에 연결

첫 분리 대상의 예: `run_run`·`run_knowledge` 같은 CLI 진입 함수를 앱에서 직접 호출하는 방식의 회피. 공개 결과를 반환하는 업무 함수에 CLI와 앱의 얇은 연결 계층 배치. 기존 CLI의 자동화·시험·플러그인 사용 보존.

## 실제로 필요한 정책 변화

- 검토 대상: [ADR-0002](../../decisions/ADR-0002-subscription-host-only.md), [ADR-0004](../../decisions/ADR-0004-orchestration-ownership.md), [실행 소유권 지침](../../../.agents/directives/02-architecture.md), [인증·안전 지침](../../../.agents/directives/05-security-safety.md)
- 연결형 앱: Hive가 선택한 실행 엔진 프로세스의 시작·중단·복구를 맡는 범위와 엔진의 모델·도구 실행 책임 분리
- 자체 엔진: 모델 제공자 연결·인증 수단·요금·도구 실행의 새로운 소유권 정의
- `Host`에 단순히 `Hive` 값 추가보다 화면·작업 소유자·실행 엔진·모델 제공자 식별의 분리 필요
- 기존 실행의 소유자 자동 변경 금지. 새 실행이나 명시적 마이그레이션으로만 전환
- 조사 요청에 따른 정책 문서 자동 개정·SDK 설치·모델 요청 실행 없음

검토 깊이: 관련 패키지·주요 함수와 계약의 범위 제한 조사. 모든 내부 함수·실행 경로·의존성 호환성의 전수 감사 제외. 새 경로·함수명은 구현 후보이며 현재 존재 API와의 구분.
