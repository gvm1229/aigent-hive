# 브랜치 이름 규칙 조사와 수정안

- 날짜: 2026-09-18
- 대상: Aigent Hive 소스, 기준 `119a4058`
- 범위: 지침·문서·로컬 Git·검사 코드·GitHub 규칙의 읽기 전용 조사와 수정 제안
- 환경: Windows Codex, Git `2.45.1.windows.1`
- 이번 작업의 브랜치 변경·검사 구현·호스트 설정·원격 규칙 변경 제외

## 조사 순서

1. 지침 정본과 사용자 문서의 허용 이름·생성 권한·예외 대조
2. 현재 브랜치와 생성 기록, 이름 검사·훅·자동 검사·시험 연결 추적
3. GitHub 실제 적용 규칙과 공식 Git 문서 확인
4. 원인·현재 이상·과거 기록 구분, 단계별 수정과 수용 조건 제안

## 결론

- 직접 원인: 명시적 저장소 규칙을 확인한 뒤에도 호스트 기본 이름 제안을 우선한 에이전트 판단 오류
- 구조적 원인: 문서 규칙을 실제 브랜치 생성·게시·병합에 연결하는 이름 검사 부재
- 올바른 현재 작업 이름: `refactor/hive-foundations`
- 사용자 최신 요구: `main`·`develop` 외 브랜치의 작업 종류 접두사 의무. 기존 단독 `staging` 예외와 불일치

## 근거와 이상 항목

| 항목 | 관찰 근거 | 영향·판정 |
| --- | --- | --- |
| A1: 금지 규칙의 오적용 | [03-workflow](../../.agents/directives/03-workflow.md)의 작업 종류 8개·에이전트 이름 금지, [사용자 안내](../guides/branching-rules.md)의 동일 규칙 | 규칙 부재가 아닌 적용 실패. 직전 작업에서 지침 확인 뒤 금지 이름 생성 |
| A2: 잘못된 검증 | 직전 작업의 `git check-ref-format --branch codex/refactor-hive-foundations` 성공 뒤 브랜치 생성 | Git 문법 적합성만 검사, 프로젝트 이름 정책은 미검사 |
| A3: 로컬 차단 부재 | `core.hooksPath` 설정 없음, `.git/hooks`는 `.sample`만 존재, [dev-check](../../scripts/dev-check.py)의 `pre-push`는 Rust·Python 실행만 연결 | 직접 Git 명령·일반 커밋에서 이름 위반 차단 불가. `pre-push` 이름의 실행 모드와 실제 Git 훅은 별개 |
| A4: 자동 검사 누락 | [CI](../../.github/workflows/ci.yml)의 push 대상은 `main`·`develop`, 이름 판정 없음 | 작업 브랜치의 push만으로 검사 미시작. PR 실행에도 이름을 검증하는 단계 부재 |
| A5: 원격 규칙 누락 | GitHub 실제 규칙 2개 모두 `main`·`develop` 각각에만 적용; 조사 대상 브랜치의 적용 규칙 조회 결과 0개 | 새 이름의 원격 생성·게시를 막는 규칙 부재 |
| A6: 예외 충돌 | 지침과 안내의 승인된 단독 `staging` 허용 | 최신 사용자 요구인 두 기본 브랜치 외 작업 종류 접두사와 충돌 |
| A7: 회귀 시험 공백 | [정적 계약](../../tests/conformance/contracts/test_static_contracts.py)의 병합 경로 문장 검사만 존재 | `codex/` 거부·유효 이름 허용·실제 생성 무변경을 증명하는 시험 부재 |
| A8: 잘못된 이름의 현재 문서 전파 | [PLAN](../plans/PLAN.md), [CURRENT](../state/CURRENT.md), [리팩터링 총괄](../plans/active/refactor-foundations.md)의 동일 금지 이름 | 다음 작업에서 정상 이름처럼 재사용할 위험 |

생성 경위: `develop@87b84f42` → `codex/refactor-hive-foundations` 전환 → `5829f07a`·`119a4058` 계획 커밋. 조사 시 로컬은 두 기본 브랜치와 문제 브랜치, GitHub 원격은 `main`·`develop`만 존재. 현재 문제 브랜치의 원격 게시 근거 없음.

원격 규칙의 구체적 범위:

- `Develop safety` (`20107178`): `refs/heads/develop` 삭제·비선형 갱신 금지
- `Protect main` (`19602033`): `refs/heads/main` 삭제·비선형 갱신 금지, PR과 `Protected merge gate` 요구
- 두 규칙의 우회 대상 목록: 빈 목록
- 옛 branch-protection API의 404는 보호 부재의 근거로 사용 불가; 실제 ruleset API에서 보호 확인
- 이름 검사와 별개인 인접 공백: `main` PR의 출발점을 `develop`으로 한정하는 문서 규칙도 현재 자동 검사에서 미확인

## 과거 기록과 현재 규칙의 구분

[ADR-0017](../decisions/ADR-0017-0.9-full-release.md)의 `codex/release-0.9.2`와 [출시 시험](../../tests/conformance/release/test_release_qualification_order.py)의 동일 문자열은 과거 출시 기록 확인용. 현재 브랜치 이름 허용 규칙의 근거로 사용 금지.

- 과거 Git 이력·출시 기록·동결 기준본의 일괄 이름 치환 제외
- 필요한 변경: 해당 문단의 과거 기록 표시와 현재 정책 링크
- 해당 시험은 과거 기록 보존 검사로 유지, 새 이름 정책의 행동 시험을 별도로 추가
- 과거 기록이 이번 판단의 직접 원인이라는 증거는 없음; 혼동 가능성과 직접 원인의 구분

## 제안 1: 단일 정책과 명시적 생성 전 검사

허용 형태:

```text
main
develop
<feature|fix|release|docs|test|refactor|build|chore>/<비어 있지 않은 Git 유효 이름>
```

이름 정책과 Git 문법 검사의 교집합만 허용. `codex/`·`claude/`·개인 이름 접두사는 허용 목록 밖이므로 거부. 대소문자·접두사를 자동 보정하지 않고 정확한 입력을 검사. 중첩 이름은 Git 문법이 허용하는 범위에서 유지; 새 소문자·날짜·버전 형식 제한의 임의 추가 제외.

접두사 판정 예시:

```regex
^(main|develop|(feature|fix|release|docs|test|refactor|build|chore)/.+)$
```

- `git check-ref-format`만으로 성공 판정 금지. `@{-1}` 같은 이전 브랜치 확장도 원문 정책 검사를 대체하지 않는 조건
- 이름 유효성과 생성 권한 분리: 올바른 이름도 별도 브랜치 생성의 사용자 요청 필요
- 단독 `staging` 예외 제거 권장. 별도 사전 운영 브랜치가 필요한 경우 `release/staging`과 기존 명시 승인·보호 조건 유지
- 에이전트 진입점에 짧은 예시 추가: 작업 종류가 지정된 새 브랜치 요청은 해당 접두사로 해석, 도구 기본 제안은 명시 정책을 대체하지 않는 조건
- 의미 판정의 한계: 코드로 접두사·문법 검증 가능, 자연어 생성 권한이나 가장 적절한 작업 종류의 완전 자동 판정 제외

구현 위치안: 하나의 이름 판정 모듈과 공통 시험 자료. 지침·안내는 정책 설명을 소유하고, 로컬 명령·훅·자동 검사·원격 설정은 같은 허용 목록에서 파생. 동일 정규식의 수동 복사로 두 번째 정본 생성 금지.

## 제안 2: 생성·게시·병합 단계별 강제

| 단계 | 제안 | 증명 범위와 한계 |
| --- | --- | --- |
| 생성 명령 | 이름 검사 후 `git switch -c` 실행하는 작은 소스 개발 도구 | 도구 경로의 사전 거부. 직접 Git 사용까지 강제한다는 주장 제외 |
| 로컬 Git | `reference-transaction`의 `prepared`에서 `refs/heads/*`의 새 값 검증 | 브랜치 참조 변경 확정 전 거부. 설치된 Git·운영체제별 실제 시험 필요 |
| 게시 전 | `pre-push`에서 표준 입력의 목적지 브랜치 검사, `dev-check.py pre-push`에도 같은 검사 선행 | `HEAD:codex/name`처럼 로컬 이름과 다른 목적지의 우회 방지 |
| 자동 검사 | 가벼운 이름 검사에 작업 브랜치 push·PR 포함, PR의 실제 head/base 사용 | 무거운 전체 시험을 모든 작업 브랜치 push에 확대하지 않는 별도 조건 |
| 원격 저장소 | 모든 브랜치에 허용 이름 ruleset 적용 | 로컬 훅 생략·웹·API 생성에도 서버 단계 검사. 실제 저장소 기능 지원·권한과 거부 시험 별도 확인 |

Git `2.45.1` 공식 문서에서 `prepared` 단계의 비정상 종료는 참조 변경 거부, `post-checkout`은 변경 뒤 호출로 결과 취소 불가. 생성 차단에 `post-checkout`만 사용하는 방안 제외. 로컬 훅 설치·설정은 우회 가능한 개발 보조 장치이며 서버 검사의 대체 수단이 아닌 구조. [Git 2.45.1 훅 설명](https://raw.githubusercontent.com/git/git/v2.45.1/Documentation/githooks.txt)

훅 구현의 필수 예외 처리:

- 브랜치 생성·이름 변경의 새 이름 검사, 기존 위반 이름 삭제는 이름 검사만으로 차단하지 않는 처리
- 보호된 기본 브랜치 삭제 금지는 별도 기존 규칙 유지
- `refs/remotes/*`, 태그, 임시 병합 참조, `HEAD`를 일반 브랜치 이름으로 판정하지 않는 처리
- 기존 훅·사용자 설정 보존, 저장소 범위 설치, 새 복제 환경의 설치 여부 점검
- 실행 파일·정책 파일 부재와 잘못된 입력은 구체적 진단, 검증 성공으로 위장 금지
- detached HEAD의 시험·출시 경로는 명시 검증할 ref 사용, 빈 이름으로 묵시 통과 금지
- 이름 변경 중 일부 단계만 적용되는 경우와 복구 가능성은 실제 Git 명령으로 시험

GitHub 문서상 브랜치 이름의 정규식 제한과 `branch_name_pattern` 지원. 적용 시 전체 브랜치 대상·불필요한 우회 없음·기존 보호 보존. 해당 저장소에서 사용할 수 있는 실제 규칙 유형을 확인한 뒤 활성화하고, 불가한 경우 허용 목록 밖 이름의 생성 제한으로 접두사 경계를 구현. `*`와 `**/*`의 슬래시 처리 차이 검증 필수. 원격 설정 변경은 이번 조사에서 미실행. [규칙 생성](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/creating-rulesets-for-a-repository), [규칙 API](https://docs.github.com/en/rest/repos/rules)

## 제안 3: 현재 브랜치 교정

실제 적용 시 순서:

1. 작업 상태·현재 커밋·동명 브랜치·원격 게시 여부 재확인
2. 로컬 이름을 `refactor/hive-foundations`로 변경
3. `PLAN.md`·`CURRENT.md`·리팩터링 총괄의 현재 참조와 활성 세션 참조 갱신
4. 이름 변경 전후 커밋·작업 파일 일치 확인, 생성 검사와 원격 정책 적용 순서 조정

이름 변경 명령 제안:

```sh
git branch -m refactor/hive-foundations
```

현재 원격에는 해당 작업 브랜치가 없어 원격 삭제·강제 push 불필요. 실행 시점의 상태 재확인 필수. 기존 두 계획 커밋의 재작성·재생성 제외.

## 회귀 시험과 수용 기준

| 사례 | 기대 결과 |
| --- | --- |
| `main`, `develop`, 작업 종류 8개와 정상 이름 | 이름 검사 허용 |
| `codex/refactor-x`, `claude/fix-x`, `person/task`, `staging`, `main/topic` | 정책 거부 |
| `refactor/`, 공백·잘못된 Git 구문 | 거부, 새 브랜치 0개 |
| `feature/topic/nested` | 기존 접두사 규칙 범위의 허용 |
| 직접 `git branch`, `switch -c`, `checkout -b`, `branch -m`, `update-ref` | 훅 적용 범위의 잘못된 대상 참조 거부 |
| 정상 이름에서 `HEAD:codex/x` 게시 | 목적지 이름 검사 거부 |
| 로컬 훅 우회 후 원격 게시·웹·API 생성 | 서버 거부 확인 |
| PR head와 합성 `refs/pull/*/merge` 구분 | 실제 head 이름 검사 |
| 작업 브랜치→`main`, `develop`→`main` | 별도 병합 경로 정책의 거부·허용 |
| 기존 위반 이름→정상 이름 교정 | 커밋과 작업 파일 보존 |
| 태그·원격 추적 참조 갱신 | 이름 정책의 오탐 0건 |

## 이번 조사에서 확인한 범위

- Windows Git 읽기 전용 이름 검사 12개 실행: 금지 이름도 Git 문법에는 적합한 사례 확인. 제안 접두사 판정과 문법 검사의 차이 검증
- 로컬 지침·현재 이름·기록·훅 설정·검사 코드 확인, GitHub 실제 브랜치와 ruleset 조회 성공
- 404 응답의 옛 보호 API와 실제 ruleset 보호의 차이 확인; 보호 없음으로 오판하지 않는 해석
- 새로운 훅·검사·서버 규칙의 차단 효과는 미검증. 이번 작업은 구현 전 분석과 제안에 한정
- macOS·Linux 실행, 실제 잘못된 원격 브랜치 생성, 브랜치 교정은 미실행
- 리팩터링 제품 구현 항목의 완료율·출시 권한 변경 없음
