# 0.14.0 설정 출처·동의·이전 기록

> Plan version: 0.14.0
> Scope: product

## 항목

- [x] [P140-001] 설정 자료 형식과 선택 출처의 원자적 저장
  - state: complete; evidence: repo:docs/research/cli-foundations-0.14.0.md#sha256:aaa49c66c9c3a5a4b41076404e1121a3ddb9f24fee2c523bcce0901ad5042164
- [ ] [P140-002] 이전 설정의 질문 기반 이전·중단 복구
  - state: agent-owned; depends: P140-001,S140-001

## P140-001 — 자료 형식

- 진입: 기존 `user_setup.rs::UserSetupConfig/UsageGuardPreferences`, `hive-render`의 설정 답안, 사용자·프로젝트 설정 스키마 확인. 기존 설치 journal·동시 변경 확인 재사용
- 변경: 사용자 `user-setup.yml`과 프로젝트 `setup-answers.yml`의 schema_version2 읽기/쓰기 추가. v1 입력은 계속 읽기 가능, 성공한 새 적용에서만 v2 저장. 역사 원본 변경 없음
- 두 설정 파일에 `preference_decisions` 추가. 키는 설정의 JSON Pointer, 값은 아래 필드. 설정값과 출처의 같은 저장 단위 유지, 별도 SQLite 정본 금지

| 필드 | 의미·허용값 |
| --- | --- |
| `origin` | `default`, `explicit`, `accepted-default`, `inherited`, `legacy-unknown`, `external-change` |
| `input_channel` | `terminal`, `answers-file`, `agent`, `migration` |
| `scope` | `user` 또는 `project` |
| `question_id`, `question_revision` | 실제 제시한 질문 식별자·의미 변경 번호 |
| `policy_version`, `changed_in_version` | 당시 기본값 정책과 마지막 변경 제품 버전 |
| `decided_at` | 실제 답변 시각. 출처 불명 이전값에는 null |
| `value_digest` | 저장 선택 선언의 JCS SHA256. 상속은 상속 선언만 계산, 전역 값 변경을 직접 편집으로 오인 금지 |
| `inherited_from` | 상속 출처의 범위와 부모 설정 포인터. 프로젝트 키와 전역 키가 다른 경우에도 실제 선언 보존 |

- 직접 값 입력은explicit, 빠른 설정 일괄 수락이나 화면 기본값의 Enter 선택은accepted-default. inherited는 전역 값을 매번 계산하며 프로젝트에 독립 선택으로 복제하지 않는 원칙
- AI가 저장한 값도 사용자 확인 증거가 없으면explicit로 승격 금지. 명시한 답안 파일의 적용 요청은 해당 입력값 선택으로 처리, 자동 보충 기본값과 구분
- 수동 파일 변경으로 값과 지문 불일치 시external-change 판정·해당 값 보존·관련 변경 전에 재확인. 동시 편집은 덮어쓰기 대신 충돌 반환
- secret·대화 원문·모델 추론·호스트 세션 식별자 저장 제외. 경로·필요한 기능 설명은 기존 스키마 경계 유지. 개발자 전송 기본 없음, 진단 내보내기 때만 허용된 필드 선택
- 기존 vector-search 답변 기록은 동일한 질문 상태로 읽어 이미 답한no/yes 보존. 기존 명령의 응답 형식 호환, 선택형 미응답을 일반 작업 전체 차단 조건으로 사용하지 않는 구조
- 완료: v1/v2 읽기, 저장 왕복, 값·지문 일치, 한 파일의 값/출처 동시 교체, 직접 편집·동시 변경·손상 기록 회귀 통과

## P140-002 — 이전과 동의

- 진입: P140-001·S140-001 완료. 사용자 설정·실제 감시 상태·등록된 프로젝트 예외의 읽기 전용 비교
- v1의true/false에서 의도 추정 금지. 검증 가능한 옛 동의가 없는 값은legacy-unknown, 그 값 자체는 보존. 새 bool 누락만으로 기존 보호 전체 해제 금지
- 대화형 갱신: 현재 효과·새 권장값·바뀔 동작 표시. 기존 비활성은 보존, 활성/출처 불명은 유지 또는 권장 해제를 질문. 직접 활성 선택이 확인되면 그 사실 표시. 답변 없이는 정책 변경 없음
- 비대화형 갱신: 기존 효과 보존, 새 선택형 기능 추가 활성 없음, 필요한 질문만pending 기록. 선택형 pending은 설치 실패나 모델 작업 차단 사유 아님
- 새 설치: 별도 과거값 없음. 새 기본값·선택한 빠른/맞춤 설정 적용, 언어가 없고 입력도 불가능하면 `needs-input`과 재개 명령 반환
- 상태: `inspected → awaiting-answers → previewed → applying → validated → complete`. 취소·입력 종료는awaiting-answers 유지, 적용 오류는recoverable-failure. 취소를no로 저장 금지
- 질문 종료 때마다 사용자루트 .hive/config/setup-progress/user.yml 또는 projects/<target-digest>.yml에 비밀 없는 답안과 다음 질문 저장. 최종 확인 전 실제 선호 변경 없음. 재개 때 원본 지문·질문판·허용 범위를 대조, 바뀐 항목만 다시 질문
- 이전 journal에 migration_id·from/to_schema·정책판·before/after 지문·완료 단계 기록. 같은 이전 재실행은no-op, 이후 사용자 변경은 덮어쓰기 금지. 이미 완료한 답변은 질문 의미가 바뀌거나 사용자가 재설정할 때만 재요청
- 적용 실패: 기존 transaction으로 설정·출처·투영 함께 복구. 새 schema를 못 읽는 구버전의 단순 재실행을 복구로 간주 금지, 검증된 이전 설정 백업과 실행 파일을 한 묶음으로 복구
- 검증: `python scripts/dev-check.py rust -- test -p hive-cli --locked`와 `python scripts/dev-check.py rust -- test -p hive-render --locked`; v1 true/false/누락·직접 수정·재시도·취소·동시 설정·질문판 변경의 자료 기반 시험 추가
- 완료 근거: 사용자/프로젝트 전후 설정과 지문·질문 횟수·실제 변경 경로. 사용자 계정 원문이나 실제 지식 복제 없이 합성 입력 사용

## 저장 범위와 수명

- 프로젝트 진행에 대상 식별자·정규화 경로 지문·스키마/질문판·원본 설정 지문 포함. 다른 프로젝트 진행 재사용·이동/복제 자동 승계 금지
- 완료/제거 기록은 .hive/history/lifecycle/<operation-id>.json, 범위별 최근100개 보존. 미완료/복구 기록은 완료 전 자동 삭제 금지, 현재 선택 출처는 설정 파일 유지
- 처리 기록에 자유 입력 복제 없이 필요한 지문만 사용. 출처 기록은 실행 권한·동의 증명으로 사용 금지
