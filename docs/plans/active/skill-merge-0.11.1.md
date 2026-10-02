# 새 Hive 개선과 사용자 스킬 결합

> Plan version: 0.11.1
> Scope: product
> 다음 공개 시험: `0.11.1-test.7`

## 목적과 경계

- 사용자 의도: 새 Hive 스킬 개선과 사용자 수정의 결합, 기본 스킬 제외·사용자 파일 이동을 대신 해결책으로 사용하는 방식 제외
- 기존 0.11.1 안정판·main·공지 전송 승인 유지, test.6 근거 보존; 새 제품 변경의 다음 번호 test.7 필요
- WProject 실제 적용은 Hive 경로 검증 후 정확한 결합 내용 검토 단계, 현재 원본 불변
- 모델·의미 검토는 호스트 소유, CLI는 지문·경로·권한·거래·복구 검증만 수행

## 수용 기준

- [ ] [SGM-001] 기존 인증 원본의 병합과 겹치는 새 변경의 명확한 안내
  - state: agent-owned
- [ ] [SGM-002] 원본 없는 사용자 스킬의 검토 결합·지문 승인·동일 경로 적용
  - state: agent-owned
- [ ] [SGM-003] 동시 변경·허용 경로·복구·다음 갱신의 사용자 수정 보존
  - state: agent-owned
- [ ] [SGM-004] 전체 검사·test.7 공개 수용·WProject 정확한 결합 미리보기
  - state: agent-owned

## 구현 결정과 순서

1. SGM-001: 기존 `hive-update::three_way_merge`의 인증 기준·서로 다른 변경 결합 유지. `project-refresh`에 `omitted_incoming_hunks`의 검토 의무, 새 개선 전체 반영으로 거짓 완료 금지. 겹친 의미는 사용자 규칙·필요한 새 Hive 규칙을 함께 반영한 호스트 작성 결합본으로 검토.
2. SGM-002: 기존 `project upgrade`에 `--skill-merges <json>`과 `--approve-skill-merge <digest>` 추가. `project_upgrade/skill_merge.rs`의 엄격한 요청: schema_version=1, product_version, project_base_digest, files의 path·local_digest·incoming_digest·merged_content. 인증한 기존 프로젝트 원본 목록의 지문과 현재 제품을 요구, 선택된 들어오는 기본 스킬의 이식 가능 경로만 허용. 전체 요청·각 파일 크기 제한, 중복·불명 필드·알 수 없는/다른 경로·소스 없음·지문 변경 거절. 기존 원본이 없는 파일의 가짜 원본 생성 금지. 명시 검토본만 기존 거래에서 대체, 들어오는 Hive 본문을 다음 비교 기준으로 저장하고 결합본은 수정본 목록에 기록.
3. SGM-003: `run`에서 결합 계획과 정규화 대상의 지문으로 승인 값을 계산, 미리보기에는 승인 값과 각 경로의 비교 지문 표시. 실제 적용은 그 값의 명시 확인 필수; 일반 갱신 권한을 사용자 파일 인수 권한으로 확대 금지. 원래 거래의 백업·변경 경쟁·복구 사용. 현재·들어오는·결합 내용 하나라도 바뀌면 새 검토. 적용 뒤 요청 파일 없이 `--validate`와 두 번째 갱신 변경 0건, 다음 개선과 결합의 보존 검증.
4. SGM-004: 인증·누락 원본·겹침·불명 스킬·승인 없음/다름·경쟁·복구·보존의 합성 사례 시험. WProject 내용의 제품 시험 자료 복사 금지. 관련 검사 → 전체 Rust/Python → 필수 CI → 유일한 test.7 후보·공개·세 운영체제 수용. 그 뒤 WProject의 새 원문·사용자 검사 규칙을 포함한 정확한 결합 차이 제시. 시험판 설치와 실제 소비자 적용의 버전·내용별 권한 경계 유지.

## 소유 파일과 검증

- CLI: `project_upgrade.rs`, 새 `project_upgrade/skill_merge.rs`, `main.rs` 도움말; 기존 결과·승인 없는 경로 호환
- 요청 형식: `schemas/project-skill-merge.schema.json`, 기존 CLI JSON 결과의 data에 선택적 승인 값
- 소비자 안내: `harness/skills/project-refresh/SKILL.md`와 배포 사본
- 검증: 해당 CLI Rust 시험·Python 명령/형식 시험, 기존 소유권·Windows Claude 회귀 유지
- 중단: 정확한 결합의 사용자 의미 결정 또는 실제 사용자 시험판 설치 승인만 해당 단계 대기, 독립 작업 계속
