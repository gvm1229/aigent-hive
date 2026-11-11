# 05 — 경험 재사용·승격·복구

- 상위: [최종 계획](../../score-driven-improvement-0.14.0.md)
- 소유 항목: QSI-009
- 후보 시험과 채택: [04 하네스](04-harness.md)

## QSI-009 — 경험·지속 적용·복구

- 선행: QSI-008
- 신규: quality/experience.rs와 schemas/improvement-experience.schema.json
- 기존 연결: knowledge remember/promote의 검토·범위·중복·기밀 경계, run/policy_review의 실패 분류·반례 기록
- 기존 활성화 재사용: hive-render/src/activation.rs의 stage_and_validate·activate_staged_impl_with_pin·rollback_projection. 공개된 최소 래퍼로 호출, 별도 설치 엔진 없음

### 경험 계약

- 내용: 적용 조건, 기준/후보 파일 지문, 원인 가설, 관찰된 변화, 효과/반례 참조, 평가표·호스트 계약, 검증 상태, 허용 범위, 철회 이유
- 실제 재사용 구성 파일은 소유된 후보 저장소에 유지. 지식에는 검토된 요지와 위치·지문만 기록
- 원시 대화·숨은 추론·비밀·기밀 원문의 자동 저장 금지
- 상태: observed → validated → approved-for-scope → active 또는 rejected/revoked
- 점수가 높거나 선택됐다는 이유만으로 validated·전역 지식 승격 금지
- 기본 프로젝트 범위, 검색 상위3개·합계16KiB. 전체 구성은 필요할 때만 검증 후 로드
- 매 새 작업은 승인된 기준 구성에서 시작, 적용 조건이 맞는 경험만 후보로 사용
- 오래된 프로필·환경·모델·소유 지문은 재시험 전 비활성. 자동 삭제 대신 철회·대체 연결 보존

### 적용과 실패

- 프로젝트 적용은 현재 승인된 정확 diff·대상 지문·소유 목록 확인 후 기존 transaction 사용
- 원본 동시 변경 시 무변경 거부. 부분 실패는 정확한 변경 전 바이트로 복구, 복구 실패는 근거와 임시 자료 보존
- 새 실제 작업에서 회귀 확인 시 해당 경험 비활성·기준 복귀. 사용자의 이후 편집을 덮어쓰는 복구 금지
- 소스의 경험은 제품 변경 제안으로 인계, 출시 묶음이나 설치 하네스의 직접 갱신 금지

### 검증·완료

- 신규 tests/conformance/contracts/test_improvement_experience.py
- python -m unittest tests.conformance.contracts.test_improvement_experience
- cargo test -p hive-render activation
- 반례: 다른 프로젝트 유출, 승인 없는 전역 승격, 같은 후보 중복, 기밀 승격, 오래된 증거, 적용 중 충돌·복구 실패
- 완료: 적용 범위·근거·철회 추적, 사용자 바이트 보존과 검증된 복구
