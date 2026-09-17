# 지식 흐름과 실제 호스트 검증

> 상태: 구현 전 계획
> 소유 항목: `RF-K01–K03`, `RF-H01–H03`
> 선행 조건: [공통 기준](refactor-foundations.md)의 `RF-B01–B02`

## 지식 흐름

목표: 결정 하나를 저장한 뒤 새 대화에서 관련 근거를 찾고, 같은 설명 없이 다음 작업 연결.

| 현재 책임 | 구현 대상 | 변경 방향 |
| --- | --- | --- |
| 명령 해석·저장·검색 진입 | `crates/hive-cli/src/knowledge.rs` | 인자 해석·결과 출력과 작업 절차 분리 |
| 저장·색인·검색 | `crates/hive-wiki/src/{store,rag,shared,collection}.rs` | 기존 저장소 재사용, 대상 범위 판정과 검색 순서 명확화 |
| 소스 지식 | `crates/hive-cli/src/source_wiki.rs` | 소스 전용 조회 경계 유지; 소비자 경로와 무리한 통합 제외 |
| 호스트 행동 | `harness/skills/knowledge-{capture,recall}/` | 최소 자료 전달, 중복 기록 방지, 실패 이유와 다음 행동 표시 |

- [ ] [RF-K01] 합성 결정·질문·기대 근거 20개 이상의 고정 평가 자료와 기존 결과 기록
- [ ] [RF-K02] 저장·조회 진입의 책임 분리, 범위 판정 단일 경로와 기존 CLI·결과 호환성 검증
- [ ] [RF-K03] 고정 자료의 재평가, 새 대화 전달 묶음의 정확성·중복 기록·지연·질문 횟수 비교

평가 자료: 같은 결정 재저장, 결정 교체, 한국어·영어 질문, 결과 없는 질문, 사용자 전역·현재 프로젝트·명시 다른 프로젝트, 미등록 대상, 기밀 승인 거부, 오래되거나 손상된 색인, 선택형 검색 부재 포함.

합격 조건:

- 반복 저장의 새 정본 0건, 변경된 결정의 현재 근거 명확화
- 금지 범위·기밀 자료 노출 0건, 근거 없는 결정 생성 0건
- 기존에 성공한 질문의 기대 근거 유지; 20개 중 최소 18개에서 기대 근거가 상위 5개 안에 포함
- 핵심 시나리오의 사용자 재설명 요청 0회; 모호하거나 권한이 필요한 질문은 별도 집계
- 동일 자료의 CLI 준비 비용·검색 p95를 기준선과 비교; 큰 지식 모음의 기존 수용 기준 유지
- 합성 CLI 평가와 실제 새 대화 성공률을 분리; 실제 앱 검증의 소유 항목은 `RF-H02`

직접 관련 시험: `tests/conformance/integration/test_shared_knowledge_index.py`, `test_root_knowledge_promotion.py`, `test_source_wiki.py`, `crates/hive-wiki/tests/knowledge_retrieval_qualification.rs`.

## 호스트 검증

목표: 실행 앱에서 확인한 기능만 공통 계약에 반영하고, 지원 여부를 추측하는 코드 제거.

대상: `crates/hive-core/src/{run,orchestration,native_workflow}.rs`, `crates/hive-cli/src/run.rs`, `schemas/capability-matrix.schema.json`, 기존 호스트별 투영과 실행 결과 계약.

- [ ] [RF-H01] 현재 호스트 기능 탐지·작업 결과 확인·취소·재개 계약의 코드 및 공식 근거 확인, 호스트·버전·운영체제별 상태표 작성
- [ ] [RF-H02] Windows Codex의 승인된 격리 대상에서 지식 저장→새 대화 조회→작업 연결과 지원되는 실행·완료·취소·재개 실측
- [ ] [RF-H03] 동일 자료로 Claude Code·Antigravity 확대 검증, 모사 시험과 실제 호스트 근거를 별도 표시

상태표 필드: 작업 종류, 호스트·버전, 운영체제, 호출 방법, 대상·세션 결합, 중복 요청 처리, 완료 확인, 취소 확인, 복구, 근거 위치, `verified|unsupported|unverified`.

구현 규칙:

- 준비 명령 성공과 실제 작업 실행 구분; 완료는 해당 실행의 확인 가능한 결과로 판정
- 앱 내장 기능의 존재만으로 Hive 연결 가능 판정 금지; 지원된 호출·결과 경로까지 확인
- 확인 불가능한 전송은 기존 `dispatch-uncertain` 의미 유지; 임의 재전송 제외
- Codex 검증 이후에만 공통 부분 추출. 세 호스트를 위한 빈 추상 계층 사전 생성 제외
- 인증은 각 호스트 소유; 공급자 API·자격 증명·Hive의 모델 프로세스 실행 제외
- 새 Codex 작업 생성이나 실제 사용자 설치가 필요한 수용 절차는 그 정확한 행동의 권한 확보 후 실행
- 실제 호스트 접근 불가 항목은 `awaiting-external-evidence`와 필요한 실행 절차 기록; 모사 성공으로 대체 금지
- `RB104-004`의 15초 감시·진행 중 추론 중단은 이 계획에서 구현 완료로 승격 금지

직접 관련 시험: `tests/conformance/contracts/test_host_capabilities.py`, `test_run_role_contracts.py`, `test_skill_projections.py`와 해당 Rust 실행 상태 시험.

## 완료 증거

- 모든 결과에 코드 지문·자료 지문·실행 호스트·운영체제·실제 실행 여부·실패 이유·증명 범위 기록
- 현재 계획 작성 환경: Windows Codex. 다른 호스트와 새 대화의 제품 행동은 이 작업에서 미실행
