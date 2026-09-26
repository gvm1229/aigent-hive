# 정책 규칙과 최종 검사 위치

- 목표: `0.11.0`, 소유 항목 `HK-001`, `RFP-001`
- 범위: 소스 개발·소비자 프로젝트·사용자 전역·출시의 구분
- 목록의 역할: 기존 검사와 지침의 연결. 임의 자연어를 실행 규칙으로 바꾸는 별도 언어·서버 제외
- `강제`: 아래 Hive·Git 경로 내부의 검사. 호스트·외부 프로그램 전체 통제의 의미와 구분
- `부분`: 관측 가능한 형식·상태만 검사. `판단`: 사용자 의도와 근거의 의미 검토

| 규칙 ID | 범위·정본 | 기존 검사·최종 위치 | 진입 경로·직접 시험 | 수준 |
| --- | --- | --- | --- | --- |
| `source-branch` | 소스, `branching-rules.md` | `scripts/branch-policy.py`의 `validate`, Git 참조·게시 및 원격 규칙 | `current`, `check`; `test_branch_policy.py` | 강제·로컬 이름 변경 우회 한계 |
| `commit-concern` | 소스, `commit-rules.md` | 변경과 목적의 검토, Git 커밋 전 | 소스 작업 검토; 정적 지침 계약 | 판단 |
| `fresh-verification` | 소스·출시, `03-workflow` | `test_artifacts.Run`, `check-test-release-gate.py`의 후보 자료 결합 | 시험 실행기·공개 시험 검사; `test_test_release_gate.py` | 강제·시험 충분성은 판단 |
| `active-plan` | 소스·출시, `plan-state.md` | `scripts/plan_state.py`의 버전·ID·의존·근거 검사 | `check-plan-state.py`; `test_plan_state.py` | 강제 |
| `task-closure` | 소비자, `00-project-harness` | `hive-cli/src/run.rs`의 실제 종료 상태 계산 | `hive run closure`; `test_run_role_contracts.py` | 기록된 기준 강제 |
| `source-consumer` | 소스·소비자, `AGENTS.md` | `hive-core::ensure_consumer_target`, 대상 생성 전 | 설정·지식·갱신 직접 호출; `test_setup.py` | 강제 |
| `hive-owned-state` | 소비자·전역, 소유권 대장 | `user_install`·`hive-render::activation`의 예상 바이트·권한 검사 | 설치·갱신·복구; `test_ownership_hostile.py` | Hive 쓰기 강제·파일 훅 부분 |
| `concurrent-edit` | 소비자·전역, 세션 지침 | 열린 경로 핸들·변경 직전 비교·독점 게시 | `file_ops`·설치 충돌 회귀 | 강제·예약만으로 통제 불가 |
| `update-recovery` | 소비자·출시, 갱신 계약 | `hive-update::transaction`, 실제 백업·저널·파일 복구 | 갱신·복구; `test_update_migration.py` | 강제 |
| `historical-bytes` | 소스·출시, 과거 기준본 대장 | `historical-surfaces.yml`과 공개 태그의 정확한 자료 | 과거 설치·배포 목록 시험 | 검사 경로 강제 |
| `canonical-freshness` | 소비자·전역, 지식 계약 | `RagStore::checked_retrieve`, 반환 근거와 원본 지문 확인 | `knowledge retrieve`; 저장소·조회 회귀 | 조회 경로 강제 |
| `knowledge-scope` | 소비자·전역, `01-project-knowledge` | `derive_optional_current_collection_authority`, 반환 전 필터 | 일반·명시 프로젝트·기밀 조회; 지식 통합 시험 | 강제 |
| `reviewed-memory` | 전역·소비자, 기록 Skill | `plan_remember`, 출처·중복·비밀 검사 뒤 정본 저장 | `knowledge remember`; 기억 요청 회귀 | 구조 강제·의미 판단 |
| `usage-preflight` | 소스·소비자, 사용량 지침 | `usage_control`의 공통 관측·세션 검사, `run::resume`·`loop`의 허가 직전 확인 | 사용량·자동 실행 시험 | 실행 준비 강제·진행 중 추론 별도 |
| `exact-authority` | 소비자·전역·출시, 승인 계약 | `hive-core::orchestration`의 대상·작업·지문·세대 결합 | 위조·재사용·대상 변경 회귀 | 강제·자연어 해석 별도 |
| `release-authority` | 출시, 활성 버전·출시 지침 | 후보 검사·게시 workflow·보호된 원격 | 버전·출시 계약 시험 | 해당 게시 경로 강제 |
| `host-owned-model` | 전체, 아키텍처 지침 | 공급자 호출 없는 제품 경로·호스트 소유 실행 계약 | `test_host_capabilities.py`·실행 상태 시험 | 제품 경계·외부 도구 별도 |
| `language-quality` | 전체, 언어·문서 지침 | `hive-core::korean`, 문서 검사와 검토 | 한국어·문서 시험 | 부분·정확함과 쉬움은 판단 |
| `intent-routing` | 전체, Skill 계약 | 명시적 요청·기존 Skill 경로·사용자 검토 | `test_skill_routing.py`·질문 분리 시험 | 판단·고정 사례 부분 |
| `cancel-resume` | 소비자, 실행 계약 | `orchestration`의 세대·중복·늦은 결과 검사, 호스트 확인 | 실행 상태·영수증 회귀 | Hive 상태 강제·실제 호스트 별도 |

## 결과와 권한

- `hive policy evaluate`: 작업 ID·정책 지문·대상 지문에 결합한 규칙 결과 합산, 읽기 전용
- 필수 결과 누락·중복·오류·시간 초과·미지원·다른 결합 결과: 허용 불가
- 명시적 거부 우선, 비필수 안내 실패의 별도 표시. 결과 순서의 판정 영향 없음
- 호출자가 만든 성공 결과는 실제 검사 증거의 대체물에서 제외. `authorizes_mutation=false` 유지
- 실패 제안: 규칙 부재·미준수·해로운 규칙·환경 오류·근거 부족. 검토 전 원인 확정·자동 정책 수정 제외
- 보호 동작의 실제 쓰기·반환 직전에 해당 소유 코드의 현재 상태 재검사 유지
- 읽기·상태·취소·복구의 접근 보존. 일반 대화 차단이나 셸 문자열만으로 효과 판정 제외

## 호스트 연결의 현재 범위

- 새 `hive policy hook`: Codex `apply_patch`, Claude 파일 편집, Antigravity 파일 편집 입력 변환과 직접 Hive 상태 변경 거부
- 공통 보호 경로 함수 재사용, 필수 결과의 공통 합산. 시작 규칙 전달과 기본 중립 `Stop`, 명시적 실행의 검토 안내 분리
- 별도 CLI 프로세스 시험: 실제 앱 로드·신뢰·도구 차단 증거와 구분
- 셸·기존 터미널 입력·MCP·다른 편집기: 이 파일 편집 변환기의 적용 범위 밖
- 소스 브랜치·게시의 최종 검사는 Git·원격 검사 유지. 새 변환기의 전체 소스 정책 강제 주장 제외
- 실제 등록·철회·작동 진단은 `HK-002`, 실제 효과와 별도 평가 자료는 `HK-003–004`에서 추가 검증

## 실행 후 개선 후보 검토

`hive run policy-review`는 기존 소비자 실행 폴더의 `PLAN.md`·`STATUS.md`에 결합된 검토 명령. 소스 작업 폴더에서 소비자 실행 상태 생성 금지.

1. `list --target <dir> --run <id> --output json`: 후보와 대상 지문 조회. 쓰기 없음
2. `preview`에 `--evaluation <실행-상대-파일> --rule <등록-규칙> --class <실패-종류>` 추가: 해당 실행의 근거 목록에 등록된 정책 검사 JSON 확인과 후보 미리 보기
3. 같은 인자로 `add --confirm <preview_digest>` 호출: 검토한 내용만 `POLICY-REVIEW.md`에 등록
4. `accept|revise|reject|cancel --candidate <id> --confirm <book_digest>` 호출: 현재 후보 내용에 대한 명시적 판단 기록. `revise`는 `--class`로 수정 분류 지정

- 모든 명령에 `--target`, `--run`, `--output json` 필수. 검사 JSON의 작업 ID·정규화한 대상 경로 지문과 실행 결합 필수
- 정책·규칙·실패 종류가 같은 후보의 중복 생성 방지. 허용 결과는 기존 후보의 반례로 추가 가능, 새 실패 후보 생성 근거에서는 제외
- 새 근거·반례 추가 시 `pending` 복귀. 같은 근거 재전달 시 채택·기각 상태 유지
- 채택·수정 전 근거 파일 재확인. 변경된 파일·잘못된 승인 지문·다른 대상 결합은 무변경 거부
- 저장 범위: 등록된 규칙·정책 지문·실패 분류·근거 지문·검토 상태·실행 범위 지문. 대화·도구 출력·기밀 원문·임의 메모 저장 제외
- `accepted`는 검토 판단이며 실제 적용 상태와 구분. 모든 결과의 `authorizes_mutation=false`, 원인 확정 없음. 지침·정책·지식 변경은 별도 승인 작업 필요
- 근거 등록·지문 일치는 검사 결과의 내용이 사실이라는 보증과 구분. 호스트의 사람 검토와 실제 실행 증거 필요
- Windows Codex에서 CLI 회귀 [6개 통과](../../tests/results/runs/20260919T060650-331c65f5dbc9.md), 공개 명령·스키마 [6개 통과](../../tests/results/runs/20260919T052144-7300e8843dfb.md). 실제 앱의 종료 알림·다른 운영체제는 미실행

선택형 종료 안내: Codex·Claude의 `policy hooks preview|apply`에 `--review-run <id>` 추가.
기존 실행의 호스트와 `STATUS.md` 세션 지문이 이벤트에 맞을 때만 `systemMessage` 반환.
다른 세션·취소 상태·이미 검토된 후보는 알림 제외, 기록 변경·작업 재개 없음.
Antigravity의 비재개 종료 안내는 미지원으로 거부하며 명시적 `policy-review list` 유지.
출하 절차는 [run-checkpoint 참조](../../harness/skills/run-checkpoint/references/policy-review.md).

관련 근거: [전체 적용 분석](../research/project-policy-enforcement-2026-09-18.md),
[호스트 훅 계획](../plans/active/host-policy-hooks-0.11.0.md),
[측정 기준](../research/refactor-baseline-0.11.0.md).
