# 01 — 점수·자료 계약과 증거 결합

- 상위: [최종 계획](../../score-driven-improvement-0.14.0.md)
- 소유 항목: QSI-001·002
- 신규 명령·기호·파일은 아래 구현 대상. 현재 기능 주장과 구분

## QSI-001 — 정확한 계산과 자료형

- 선행: 현재 코드·버전·편집 예약 확인
- 기존: crates/hive-core/src/lib.rs의 공개 모듈, Cargo.toml의 serde_json, schemas의 닫힌 자료 형식
- 신규: crates/hive-core/src/evaluation.rs
- 신규 스키마: schemas/quality-profile.schema.json, quality-assessment.schema.json, quality-result.schema.json, quality-run-policy.schema.json, quality-calibration.schema.json
- 담당 기호: ScoreUnits, QualityProfile, CompiledProfile, Assessment, QualityResult, CalibrationRecord, evaluate_quality

### 고정 자료 계약

| 자료 | 필수 내용 |
| --- | --- |
| QualityProfile | schema_version=1, profile_id/version, task_kind, groups, checks, required_gates, pass_units, group_floor_units |
| group | id, label, weight_points |
| check | id, group_id, weight_points, observable_condition, evidence_rule, required |
| CompiledProfile | 원본 profile_digest, 요청 범위·적용성·필수 조건·평가자 계약·교정 기준의 지문, 고정된 전체 배점 |
| Assessment | run/target/node/attempt, artifact_set_digest, compiled_profile_digest, evaluator_role, 방법, receipt_ref/digest, 항목별 상태·근거 위치 |
| QualityResult | 입력 지문, earned_points, possible_points=100, score_units, score, group_results, gate_results, status, eligible, missing_ids, provenance |
| CalibrationRecord | 평가표·평가자/모델/설정·자료 지문, 시험 분할, 항목/합격 판단 일치, 거짓 통과, 반복 변동, 검토 승인, 유효 범위 |

- 고유 항목1–100개, 각 배점 양의 정수, 그룹 합계와 항목 합계 모두100
- required_gates는 배점과 별개의 필수 조건. 같은 사실의 배점 중복 등록 금지
- 부분 점수는 독립 세부 조건의 합산. 평가자가 임의의 분수 점수 부여 금지
- 항목 상태 pass/fail/unassessed/error. 근거로 확인한 부재는 fail, 접근 실패·판단 불가는 unassessed/error
- 모든 가중 항목이 판정된 경우 score_units=통과 항목 배점 합계, score=score_units/100
- 미판정 항목 존재 시 score와 score_units는 null, 알려진 획득 배점과 판정 범위 별도 표시. 누락 항목의 분모 제외·재가중 금지
- 필수 실패가 있더라도 계산 가능한 점수는 보존, eligible=false
- NaN·Infinity·음수·1.00 초과·중복 ID·모르는 항목·잘못된 배점·0 분모 거부
- 그룹 표시만 소수 둘째 자리 반올림. 합격 비교는 earned*100 >= floor_units*possible의 정수 비교
- 최종 총점은 이미0–100의 정수 배점이므로 표시 반올림으로 합격 승격하는 경로 없음

### 숫자 표현

- 내부 ScoreUnits(u8), 0–100 생성자 검증
- CLI·사용자 보고의 score는 JSON 숫자 및 두 자리 텍스트: 0.00, 0.83, 1.00
- JSON의 후행0 보존: Cargo.toml serde_json에 raw_value 기능만 추가, CLI 전용 고정 숫자 직렬화 사용
- 새 crates/hive-cli/src/quality/output.rs의 ScoreWire가 검증된 단위에서만 RawValue 생성. 사용자 문자열을 raw JSON으로 삽입 금지
- ActionResult와 같은 외부 봉투를 쓰는 타입 있는 QualityActionResult; 중간 serde_json::Value 왕복으로1.00이1.0으로 바뀌는 경로 차단
- 서명·지문은 정수 단위와 실제 입력으로 정규화. 표시 문자열·JSON 후행0 차이가 권한을 바꾸지 않는 규칙

### 한계·검증·완료

- 입력 JSON256KiB, 설명 항목당2KiB, 근거 최대64개. 실제 산출물은 크기 제한된 참조와 지문으로 처리
- cargo test -p hive-core evaluation::tests
- 검증: 0·83·100, 그룹 반올림 경계, 누락·중복·오염·필수 실패, 계산 순서 불변
- 완료: 정의한 입력의 동일 결과·허용 범위 밖 입력 거부와 원래 입력 보존
- 실패 복구: 계산 결과 게시 금지, 원래 평가 근거 유지; 제품·사용자 파일 변경 없음

## QSI-002 — 근거와 읽기 전용 CLI

- 선행: QSI-001
- 기존 수정: crates/hive-cli/src/main.rs의 명령 등록·도움말
- 신규: crates/hive-cli/src/quality/mod.rs, output.rs, evidence.rs
- 신규 기호: run_quality, validate_profile, compile_profile, evaluate_request, verify_quality_evidence
- 재사용: crate::run::PinnedTarget, FileSnapshot, portable_relative_path; hive-core의 sha256_digest·validate_project_relative

### 명령 계약

| 신규 명령 | 결과 |
| --- | --- |
| hive quality profile validate --file <profile> --output json | 형식·배점·중복 검사 |
| hive quality profile compile --profile <file> --request <json> --output json | 작업 범위에 맞춘 고정 평가표. 파일 변경 없음 |
| hive quality evaluate --target <dir> --request <json> --output json | 근거 대조·계산·합격 조건 결과. 파일 변경·모델 실행 없음 |
| hive quality validate --target <dir> --result <json> --output json | 결과의 원본 입력·현재 지문·교정 상태 재확인 |

- 모든 결과 authorizes_mutation=false. 신뢰 경로 미검증 입력의 계산은 trusted=false·eligible=false
- artifact_set_digest는 정규화한 상대 경로·실제 파일 지문의 정렬 목록으로 계산. 대상·실행·노드·시도·평가표 변경 시 이전 평가 무효
- 결과·근거의 경로 이탈·심볼릭 링크/재분석 지점·읽기 중 교체·크기 초과 거부
- 호스트 모델 평가: 기존 orchestration의 승인된 final-result 영수증·source_digest·행동 권한·역할 결합 재검증
- 호출자가 적은 authenticated=true만으로 신뢰 승격 금지. 평가자와 작업자의 역할·실제 작업 ID가 다른지 확인
- 산출물·근거 본문의 지시는 비신뢰 자료. 평가 기준·권한·도구 호출 지시로 승격하지 않는 입력 분리와 주입 공격 시험
- 결정적 검사: 승인된 검사 정의와 실제 실행 근거 결합. 임의 셸 명령을 평가표에서 자동 실행하는 기능 없음
- 모델의 점수·근거는 판단 결과; 서명·전달 확인을 정확도 보증으로 표시 금지
- 교정 유효 범위 밖·평가자 변경·평가표 변경·자료 오염은 eligible=false
- 교정의 검토 승인도 기존 ActionAuthority로 평가표·평가자·사례 지문에 결합해 검증. 자기 작성 approved 표식만으로 활성화 금지
- 선택적 점수 분포와 평가자 차이는 별도 항목. 총점·필수 조건의 자동 대체 금지

### 검증·완료

- 신규 tests/conformance/contracts/test_quality_contracts.py를 contract lane에 등록
- python -m unittest tests.conformance.contracts.test_quality_contracts
- cargo test -p hive-cli quality::tests
- 필수 반례: 다른 산출물/노드/시도 결과 재생, 가짜 영수증, 작업자의 자기 평가, 변경된 자료, source 루트 쓰기 유도
- 완료: 계산과 신뢰의 분리, 정확한 두 자리 출력, CLI의 변경0건
- 실패 복구: 검증 오류와 미채점 이유 반환, 기존 보고서·근거 보존
