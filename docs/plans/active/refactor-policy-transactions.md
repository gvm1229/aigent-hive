# 안전 규칙과 설치·갱신 처리 분리

> Plan version: 0.11.0
> Scope: product

> 상태: 구현·검증 진행
> 소유 항목: `RFP-001–004`, `RFT-001–004`
> 선행 조건: [공통 기준](refactor-foundations.md)의 `RFB-001–002`

## 안전 규칙

목표: 프로그램이 판정할 수 있는 안전 조건을 CLI에서 강제하고, 에이전트 지침에는 작업 선택과 설명에 필요한 규칙만 유지.

- [x] [RFP-001] 현재 규칙→정본→코드 검사→진입 명령→시험의 대응표와 작업별 지침 읽기 비용 기록
  - state: complete; depends: RFB-001,RFB-002; evidence: repo:tests/results/runs/20260919T054656-9b4165cadab7.md#sha256:3c781628869eb4c2ac89db00841305e4ccf4a9b7485cb65f62272ea3bd86c88c
- [x] [RFP-002] 이미 존재하는 판정 함수 재사용과 빠진 직접 CLI 진입 검사 보강; 동일 실패의 일관된 기존 결과 코드 유지
  - state: complete; depends: RFP-001; evidence: repo:tests/results/runs/20260919T083543-4aa35fb79366.md#sha256:8281963d66ab25b55bd9c19d5f23a6307d98ce927efc41d81bb94c16f1606a12
- [x] [RFP-003] 코드로 검증되는 상세 절차의 지침 중복 축소, 정본과 현재 세 호스트 투영 동기화
  - state: complete; depends: RFP-002; evidence: repo:tests/results/runs/20260919T084503-04af8f3aa6f4.md#sha256:15de7117022ca333d10dad24d98358aa8f66b1c68a81dbf543920164edf18363
- [x] [RFP-004] 허용·거부·취소·재개·일반 질문의 전후 행동 평가와 정보량·호출 수 비교
  - state: complete; depends: RFP-003; evidence: repo:tests/results/runs/20260919T094252-cd5de58152a7.md#sha256:aba1ba10658443991cc4fcf814f66e578bf0f6d965ddb244348a047e48886d90

코드 대상: `crates/hive-core/src/{lib,usage_guard,run}.rs`, `crates/hive-cli/src/{knowledge,run,usage_control,user_setup}.rs`, 해당 JSON 스키마.

지침 대상: `harness/directives/`, `harness/skills/`, `harness/template/AGENTS.md.jinja`, `crates/hive-cli/src/user_directives.rs`, `crates/hive-projection/src/lib.rs`. 소스 개발 지침은 소비자 지침과 별도 소유권 유지.

분류와 합격 기준:

- 코드 책임: 경로·소유권·대상·지문·기밀 접근·사용량 판정·실행 결과 검증
- 지침 책임: 사용자 의도·설명 언어·추가 질문의 필요성·모델 행동. 자연어 승인을 코드가 완전히 해석한다는 가정 제외
- 승인 자료 위조·누락·대상 변경·재사용은 거부; 기존 실패 경로의 쓰기 0건 확인
- 지침을 직접 거치지 않은 CLI 호출에도 동일 경계 적용
- 2026-09-19 선택 반영: 자동 `run resume`의 세션 정보 누락 거부. 공통 사용량 검사와 중단 상태를 재사용하고, 허가 기록 직전 정책·세션 상태 재확인. 같은 호출 안의 센서 중복 조회 제외
- 일반 질문·수동 작업·자동 실행·소스 개발의 현재 사용량 검사 적용 범위 보존
- `RFP-001`에서 고정한 일반 질문·지식 작업 지침 경로의 전체 읽기 바이트 20% 이상 축소 목표; 필수 행동 평가 전부 유지
- 규칙 삭제 전 유효 주장 전수 목록과 새 정본 연결; 외부 지침과 과거 기준본 변경 0건
- 시험: `scripts/check-agent-directives.py`, 투영·정적 계약·사용량·실행·설정 보안 시험과 행동 시나리오

## 설치·갱신

현재 의존 방향: `hive-update → hive-render → hive-core`. 사용자 설치의 파일 적용은 `hive-cli/src/user_install.rs`에도 존재.

목표: 도메인별 변경 계산과 공통 파일 처리를 분리하면서 사용자·프로젝트의 서로 다른 승인 범위 보존.

- [x] [RFT-001] `user_install.rs`와 `hive-render/src/lib.rs`의 책임별 내부 모듈 분리, 기존 진입 함수·출력·테스트 호환성 유지
  - state: complete; depends: RFB-001,RFB-002; evidence: repo:tests/results/runs/20260919T053227-5fb662be3b1e.md#sha256:5ad138d4bd7be03a54861f8bbb5677f20c28905f202c8d1347ff8d2fec600769
- [x] [RFT-002] 두 호출 경로의 공통 파일 교체·복구 원시 기능 추출, 순환 의존 없는 공유 경계와 오류 계약 검증
  - state: complete; depends: RFT-001; evidence: repo:tests/results/runs/20260919T053227-5fb662be3b1e.md#sha256:5ad138d4bd7be03a54861f8bbb5677f20c28905f202c8d1347ff8d2fec600769
- [x] [RFT-003] 프로젝트 생성과 사용자 설치의 순차 전환, 기존 변경 계획·지문·거부 결과·중단 후 복구의 동등성 비교
  - state: complete; depends: RFT-002; evidence: repo:tests/results/runs/20260919T053227-5fb662be3b1e.md#sha256:5ad138d4bd7be03a54861f8bbb5677f20c28905f202c8d1347ff8d2fec600769
- [ ] [RFT-004] 프로젝트 갱신까지 공통 처리 연결, 장애 삽입·동시 변경·세 운영체제 수용과 기존 이관 검증
  - state: agent-owned; depends: RFT-003

권장 모듈 경계:

| 책임 | 위치·경계 |
| --- | --- |
| 인자·결과 출력 | 기존 CLI 진입점 |
| 사용자 변경 계산·과거 설치 판독·호스트 활성화 | `user_install/` 내부 분리; 호스트별 활성화는 일반 파일 교체와 분리 |
| 프로젝트 출력 계산·설정·소유권 | `hive-render` 내부 모듈 |
| 검증된 대상 핸들·안전 교체·복구 | 두 경로의 실제 공통 기능에 한해 `hive-core`의 좁은 내부 모듈 우선 검토 |
| 출시 출처·백업·기록·이관 순서 | 기존 `hive-update/src/transaction.rs` |

`hive-core`가 출시·설정 정책을 흡수하거나 위쪽 모듈을 참조해야 한다면 독립 하위 라이브러리로 분리. 새 라이브러리는 두 호출 경로의 실제 구현과 함께 추가; 빈 껍데기·일반 실행 엔진 사전 생성 제외.

공통 파일 처리의 최소 입력: 열린 대상 핸들, 검증된 상대 경로, 예상 기존 파일 지문·존재 상태, 교체 내용·권한. 출력: 실제 적용 목록과 복구에 필요한 변경 전 상태. 사용자 동의·설치 범위 확대 권한은 입력 자료에서 자동 추론 금지.

파일 처리 합격 기준:

- 변경 계산·검증 → 임시 준비 → 적용 기록 → 파일 교체 → 사후 검증 → 완료 기록의 기존 안전 순서 보존
- 전체 파일 묶음의 단일 원자성 주장 금지; 파일별 원자적 교체와 중단 후 복구의 실제 보장 구분
- 미리 보기의 대상 쓰기 0건, 두 번 실행 결과 수렴, 외부 바이트와 사용자 설정 보존
- 변경 직전 대상 교체·심볼릭 링크·동시 사용자 편집 거부; 충돌한 사용자 파일의 무조건 복원 금지
- 각 쓰기 경계의 강제 실패·프로세스 종료 이후 재실행, 복구 실패의 별도 진단
- 과거 설치 기준본 수정 없이 지원 버전의 실제 자료로 이관 검증
- Windows 파일 잠금·권한과 POSIX 권한·동기화의 운영체제별 실제 증거 확보

직접 관련 시험: `hive-render`·`hive-update`·CLI 설치 단위 시험, `crates/hive-cli/tests/historical_project_upgrade.rs`, `tests/conformance/security/test_setup_security.py`, `test_ownership_hostile.py`, `tests/conformance/release/test_update_migration.py`.

되돌리기: 내부 분리→공통 기능→프로젝트→사용자→갱신의 독립 커밋. 각 단계의 기존 동작 비교 후 다음 전환; 저장 형식 변경 없는 상태에서 이전 구현으로 복원 가능.
