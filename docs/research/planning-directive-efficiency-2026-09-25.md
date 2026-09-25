# 계획 인계와 지침 읽기 개선

- 기준: `0ca942bf`, 2026-09-25 Windows 소스 작업
- 범위: 소스 개발 지침·소스 검사기. 제품 코드·harness·설치본·라이브 훅 설정의 변경 없음
- 목표: 설계 판단은 계획에 명시, 구현자는 같은 결정을 재발명하지 않고 단계별 실행

## 설계와 보존

- [계획 계약](../../.agents/directives/references/planning-contract.md): 목표·제외 범위·파일/함수·결정과 이유·순서·입출력·오류·호환성·검증·복구·중단 조건
- 불확실한 API를 사실처럼 제시하는 계획 금지. 중요한 가정 실패는 근거와 함께 계획자에게 반환, 독립 단계는 계속 진행
- 모델 이름 고정 없이 계획자와 구현자의 역할 분리. 기존 대화나 같은 추론 능력에 의존하지 않는 인계
- AGENTS·00·01·04·08의 공통 문구는 의미 보존 압축. 버전 선택·동의·계속 수행·사실 보존·호스트 권한·사용자 바이트 보존의 변경 없음
- 02·07은 원문 유지. 05의 갱신/제거와 06의 작성 양식은 해당 시점에만 로드
- 기존 문체 예외는 같은 문구 지문에 한해 새 위치로 연결. 예외 확대 없음
- 소스 검사기 수정은 해당 부정 입력 시험 필수. 출시 권한 기록·배포 흐름·제품 동작을 소스 전용 검사로 대체하는 예외 금지

## 이동 대조

| 이전 소유 파일·절 | 새 정본 | 대조 |
| --- | --- | --- |
| `03-workflow.md` / Branch Strategy | [git-branches.md](../../.agents/directives/references/git-branches.md) | 원문 일치 |
| `03-workflow.md` / Temporary Worktree and Clone Lifecycle | [git-worktrees.md](../../.agents/directives/references/git-worktrees.md) | 원문 일치 |
| `03-workflow.md` / Commit Rules | [git-commits.md](../../.agents/directives/references/git-commits.md) | 원문 일치 |
| `03-workflow.md` / Iterative Commit Checkpoints | [git-commits.md](../../.agents/directives/references/git-commits.md) | 원문 일치 |
| `03-workflow.md` / Verification Tiers | [verification.md](../../.agents/directives/references/verification.md) | 원문 일치 |
| `03-workflow.md` / Test Artifact Lifecycle | [verification.md](../../.agents/directives/references/verification.md) | 원문 일치 |
| `03-workflow.md` / Risk-Tier CI and Candidate Economy | [ci-and-candidates.md](../../.agents/directives/references/ci-and-candidates.md) | 원문 일치 |
| `03-workflow.md` / Documentation-Only Integration | [documentation-verification.md](../../.agents/directives/references/documentation-verification.md) | 소스 전용 범위 명확화, 제품·출시 검증 보존 |
| `03-workflow.md` / Release Qualification Ordering | [release-qualification.md](../../.agents/directives/references/release-qualification.md) | 원문 일치 |
| `04-documentation-state.md` / Canonical locations | [knowledge-and-preservation.md](../../.agents/directives/references/knowledge-and-preservation.md) | 원문 일치 |
| `04-documentation-state.md` / User and source fact gates | [knowledge-and-preservation.md](../../.agents/directives/references/knowledge-and-preservation.md) | 원문 일치 |
| `04-documentation-state.md` / Current-truth preservation | [knowledge-and-preservation.md](../../.agents/directives/references/knowledge-and-preservation.md) | 원문 일치 |
| `04-documentation-state.md` / Language | [knowledge-and-preservation.md](../../.agents/directives/references/knowledge-and-preservation.md) | 원문 일치 |
| `04-documentation-state.md` / Reconciliation | [plan-reconciliation.md](../../.agents/directives/references/plan-reconciliation.md) | 원문 일치 |
| `04-documentation-state.md` / Final Response Closure Gate | [run-closure.md](../../.agents/directives/references/run-closure.md) | 원문 일치 |
| `04-documentation-state.md` / Stable Release Plan Gate | [stable-plan-gate.md](../../.agents/directives/references/stable-plan-gate.md) | 원문 일치 |
| `08-human-documentation-style.md` / Korean mixed-language prohibitions | [korean-style-examples.md](../../.agents/directives/references/korean-style-examples.md) | 원문 일치 |
| `08-human-documentation-style.md` / Exact bad and good examples | [korean-style-examples.md](../../.agents/directives/references/korean-style-examples.md) | 원문 일치 |
| `05-security-safety.md` / Update Safety | [update-and-removal.md](../../.agents/directives/references/update-and-removal.md) | 원문 일치 |
| `05-security-safety.md` / Destructive Operations | [update-and-removal.md](../../.agents/directives/references/update-and-removal.md) | 원문 일치 |

## 의미 대조

| 축약 대상 | 유지한 의미 |
| --- | --- |
| AGENTS | 소스/소비자 분리, 호스트 소유 모델, 제공자 API·자격 증명 금지, 정본·파생 상태, 현재 버전·다음 시험판, 버전별 안정판 승인, 보존·브랜치·중단 경계, 소스 스킬 2개 예외 |
| 00 | 근거 먼저, 핵심 선택만 질문, 범위·단순성·기존 스타일·본인 생성 고아 코드만 정리, 지식 삭제 권한 없음, 관측 가능한 검증 |
| 01 | 한국어·명시 언어 선택, 프롬프트 언어, 쉬운 설명, 근거 한계, 한정 조회, 승인 지속, 미완료 작업 지속, 취소·수동 장애·재시작 경계, 실행 영수증과 사실/추정 분리 |
| 04 | 8KiB·단일 계획·소유 ID·중복 집계 금지, 새 근거와 상태 정합화, 본 작업/하위 작업 구분, 실행 연결 유무별 종료 절차 |
| 08 | 문체·언어·번역·어미·인용 예외·전체 문장 검사, 의미·식별자·보안 경계 보존. 상세 예시는 참조에 유지 |

## 측정 범위

이전에는 작업에 맞는 상위 파일 전체를 읽는 묶음, 이후에는 같은 작업의 상위 파일과 필요한 참조를 합친 묶음의 비교.
모든 가능한 작업의 최소 비용이나 실제 과금 토큰 측정은 아닌 구분. 추가 작업 복제본·연결된 실행·어려운 문체 판단이 필요하면 해당 참조 비용 추가.
계획 본문·대화 이력·도구 정의·모델 추론·전역 사용자 지침은 측정 범위 밖. 새 계획 계약 추가로 전체 저장량이 늘어도 필요한 단계의 읽기량만 절감될 수 있는 구조.

## 최종 읽기량

| 경우 | 이전 바이트 | 이후 바이트 | 감소율 |
| --- | ---: | ---: | ---: |
| 시작 | 13,518 | 10,254 | 24.1% |
| 일반 코드 편집 | 50,813 | 30,128 | 40.7% |
| 비파괴 소스 문서·검사기 | 50,813 | 30,460 | 40.1% |
| 계획 작성 | 50,813 | 34,078 | 32.9% |
| 출시 | 55,325 | 45,281 | 18.2% |

- 전체 저장량은 루트·참조 포함 이전 57,382바이트에서 현재 60,250바이트. 상세 계획 계약과 읽기 경로를 추가한 결과이며, 전체 문서 삭제량을 절감률로 제시하지 않는 기준
- 추가 작업 복제본·연결된 실행·문체 예시가 필요한 경우 해당 참조를 더 읽는 조건

## 검증

- Windows: 읽기 목록의 누락·중복·경로 이탈·누락 파일·손상/초과 자료와 참조 크기 증가를 검사한 새 회귀 4개 포함, 관련 정적 계약·위키·출시 경계·자료 소유 검사 53개 통과
- 지침 크기·경로, Markdown 링크, 한국어 문체·기존 인용 예외, 계획 상태와 원본 지문 정합성 확인
- 이동 절 20개 대조: 19개 원문 보존, 1개는 비배포 소스 검사 범위로 명확화. 공통 문구 축약의 의미 대조는 위 표에 보존
- 제품·harness·서명·소유권 코드 변화 없음. 무관한 Rust 전체 검사·교차 운영체제 배포·새 시험판은 실행 대상에서 제외
- 실제 모델의 계획 수행 능력·토큰 과금·대기 시간은 미측정. 계획 인계 검토는 저장된 단계만으로 다음 파일·변경·성공/실패 조건을 찾는 방식, 추가 모델 호출 요구 없음
- 이전 라이브 훅 미리보기는 원문 변경으로 무효화, 개편된 정본에서 새 미리보기 생성. 실제 훅 등록은 미실행·별도 승인 대기 유지
