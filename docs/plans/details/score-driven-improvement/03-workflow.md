# 03 — 두 반복의 제어·예산·소스 적용

- 상위: [최종 계획](../../score-driven-improvement-0.14.0.md)
- 소유 항목: QSI-005·006

## QSI-005 — 산출물 수정과 완료 판정

- 선행: QSI-002·004
- 기존 수정: crates/hive-core/src/loop_graph.rs의 LoopNode·LoopGraphDocument·node_completion_ready_unchecked·validate_loop_transition
- 기존 수정: crates/hive-cli/src/loop_engineering.rs의 VerifierEnvelope·verify_verifier_envelope·verify_current_evidence_files
- 기존 재사용: orchestration::ActionAuthority·HostReceipt·Lease·SchedulerLimits, native_workflow::GoalBudget
- 신규: crates/hive-core/src/improvement.rs의 QualityRunPolicy·ImprovementState·next_improvement_action
- 신규 스키마: schemas/quality-verifier-envelope.schema.json, quality-run-state.schema.json

### 호환과 결합

- loop-graph 형식2 추가. QualityGate 선택 필드: compiled_profile_digest, run_policy_digest, calibration_digest, evaluation_evidence_id
- 형식1에는 새 필드 금지, 기존 읽기·쓰기·바이트 계약 보존. 형식2의 평가 필수 노드는 gate 삭제·약화 거부
- 새 품질 필드가 없는 구형 실행은 자동 이전 없음
- VerifierEnvelope 형식3: 기존 v1/v2 base_verification 봉투와 quality_result의 위치·지문
- base_verification은 기존 결정적 검사 또는 인증된 Judge quorum 검증을 그대로 통과할 의무
- 봉투·graph·assessment·실제 산출물의 대상/역할/노드/시도/지문 모두 일치
- verify_quality_evidence로 호스트의 accepted final-result·권한·평가자 독립성 재검증, 점수와 eligible 재계산
- CLI 계산 결과만으로 기존 Judge의 필수 경로 우회 금지. Judge 입력에는 작업자의 자기 점수·이전 판정 비공개
- 결과만 전달한 표식으로 terminal 승격 금지; checkpoint와 recover 모두 현재 원본 지문·교정 범위 재확인

### 실행 상태와 결정

| 상태·사건 | 다음 처리 |
| --- | --- |
| 결과물 없음 | 승인된 작업자에게 작성 준비 |
| 결과물 생성 | 평가자에게 현재 결과물·고정 기준만 전달 |
| 평가 오류·정보 부족 | 증거 확보 또는 평가 오류 복구, 작업자 오답으로 감점 금지 |
| 필수 실패 | 해당 실패 우선 수정, 높은 총점의 상쇄 금지 |
| 유효 점수·합격선 미달 | 감점 근거에 연결한 수정 준비 |
| 점수·그룹·필수 조건 통과 | 독립 최종 검증 준비 |
| 모든 최종 검증 통과 | 현재 산출물 지문에 한해 완료 |
| 동일 실패 반복·한도 도달 | 해당 반복 종료·미완료 보존, 독립된 다른 작업 지속 |
| 사용자 취소 | 공통 control_epoch 갱신, 이후 결과 격리 |

- 최초 결과와 수정3회: 최대4개 산출물 버전. 기존 RetryPolicy.max_attempts의 최초 포함 의미 유지
- 평가 재시도는 오류 복구에만 최대1회. 바뀌지 않은 결과를 다시 채점해 높은 값 선택 금지
- 두 번의 수정 합계 개선이0.02 미만이고 같은 결함 잔존 시 원인 재분류
- 정책 FailureClass의 규칙 부재/미준수/해로운 규칙/환경 오류/근거 부족 분류 재사용
- 원인 불명 또는 환경·권한 장애에는 하네스 수정으로 우회하지 않는 처리
- 점수 차이는 동일 입력·평가표·평가자 계약에서만 비교. 다른 버전 간 증감 표시 금지

### 하나의 전체 예산

- QualityRunPolicy의 필수 한도: max_dispatches, deadline, max_artifact_revisions, max_harness_rounds, max_evaluator_retries
- 기존 승인된 실행 예산에서 고정. 전체 예산 부재 시 자동 실행 준비 거부, 참고 계산은 제공
- 모든 작업자·평가자·후보 작성·시험·합성·최종 검증·회고가 같은 상위 계정에서 예약
- GoalBudget의 중복 할당·이중 환불 거부와 Lease의 fence·예약 계약 재사용; 별도 무제한 학습 예산 금지
- 분기 시작 전 필요한 최소 시험 묶음과 최종 검증 예산 확보. 부족하면 분기 시작 없음
- 토큰·비용은 호스트의 실제 관측이 있을 때만 표시. 미관측을0으로 채우지 않고 횟수·시간 상한 유지
- 조회·취소·복구는 예산 잠금과 무관하게 접근. 취소·응답 유실 때 검증된 미실행 증거 없이 예산 환불·재실행 금지

### 검증·완료

- cargo test -p hive-core loop_graph::tests
- cargo test -p hive-core improvement::tests
- cargo test -p hive-cli loop_engineering::tests
- 반례: gate 제거, 다른 결과 재생, 허위 자기 평가, 실패 평균 상쇄, 이중 환불, 늦은 결과, 같은 실패 무한 반복
- 완료: 두 반복의 상태와 하나의 예산·취소·검증 일관성. 실패 시 마지막 확인 상태와 모든 근거 보존

## QSI-006 — 소스·소비자 어댑터

- 선행: QSI-005
- 신규: crates/hive-cli/src/quality/run.rs, source.rs, storage.rs
- 담당 기호: BoundQualityStore, ConsumerQualityStore, SourceQualityStore
- 기존 수정: loop_engineering.rs·orchestration.rs·run.rs의 저장 경계 분리와 공통 제어 재사용
- 공통 계산·상태 전이는 동일 core. 두 번째 스케줄러·호스트 실행기 구현 금지

### 신규 외부 계약

- hive quality run initialize|status|prepare|checkpoint|cancel|recover --scope source|consumer --target <dir> --run <id> --request <json> --output json
- 읽기 전용 status와 쓰기 명령 구분. 쓰기는 현재 head·epoch·요청 지문·정확 범위 권한 필요
- consumer 명령은 기존 run/loop/orchestration 계약을 호출하는 얇은 연결
- source 명령은 명시한 source marker·정확한 저장 범위를 검증한 별도 어댑터
- prepare는 선언적 실행 봉투만 반환: prepared_only=true, spawned=false. 실제 실행은 호스트
- 기능 자격 미입증 시 해당 prepare는 unsupported/unverified, 실행한 것처럼 보고 금지
- 기존 hive loop·setup의 소스 루트 거부는 그대로 유지

| 내용 | 소비자 | Hive 소스 |
| --- | --- | --- |
| 정본 상태·평가 | .hive/runs/<id>/quality/의 Markdown | docs/research/quality-runs/<id>/의 Markdown |
| 제어 사건·권한 참조 | 기존 run의 제어 사건 | 동일 reducer를 쓰는 위 source 실행 범위 |
| 실행 임시 자료 | 소유된 run 임시 영역 | .agents/work/quality/<id>/ |
| 시험 사본 | 승인된 격리 대상 | tests/work/quality/<id>/ |
| 경험 승격 | 해당 프로젝트 범위 | 검토된 소스 사실·제품 구성 변경 제안 |

- 정본 Markdown에 타입 있는 머리말·증거 위치·지문. 임시 영역이 유일한 상태 저장소가 되는 처리 금지
- 프로젝트·사용자·소스 범위 간 결과 혼용 거부. 원시 대화·비밀·기밀 원문의 자동 기록 없음
- source 후보 시험은 소비자 시험 사본에서 실행. 실제 소스 지침·활성 코드·설치 하네스 자동 덮어쓰기 없음
- 동시 편집은 PinnedTarget·FileSnapshot·예상 head·원본 지문으로 거부. 복구 시 현재 사용자 변경 보존

### 검증·완료

- 신규 tests/conformance/contracts/test_quality_scopes.py를 contract lane에 등록
- python -m unittest tests.conformance.contracts.test_quality_scopes
- cargo test -p hive-cli quality::tests
- source에서 .hive 생성 시도, 다른 프로젝트 결과, 경로 이탈, 동시 변경, 부분 기록·중단·재개 반례 필수
- 완료: 두 범위의 같은 입력은 같은 점수·판정, 저장 위치와 권한은 분리
- 실제 호스트 봉투 소비·취소·복구의 증명은 QSI-012. 파일 생성만으로 활성 완료 표시 금지
