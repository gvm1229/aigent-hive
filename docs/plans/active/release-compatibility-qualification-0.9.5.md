# 출시 호환성 수용 게이트

> Checklist IDs: `RQC95-001`–`RQC95-007`
> Target: `0.9.5` compatible patch
> Scope: declared compatibility claim·compiled/package artifact·public test evidence의 단일 release gate

## 확인된 누락

`0.9.4` migration table은 `0.1.0`–`0.9.3` project source range 선언. compiled binary의 full
historical project base: `0.7.0`·`0.8.0`·`0.9.0` 한정. Local regression과 public `0.9.4-test.1`:
`0.9.2` consumer project의 `scan → dry-run → apply → validate` 부재. 결과: metadata compatibility
claim과 executable artifact capability 불일치.

## 운영 원칙

- Compatibility range: 설명용 metadata 아닌 executable contract. declared source version마다 local matrix evidence 필수
- Source unit test·source CLI test·package artifact test의 계층 분리. 내부 함수 성공: release artifact 성공 증거 대체 불가
- Matrix source 정본: migration table·historical-base registry·release qualification request. 수동 복제 version 목록 금지
- Public test: 모든 local matrix 재실행 대상 아님. local 전체 matrix의 digest-bound report와 사용자 영향 대표 경로 수용 담당
- Stable promotion: accepted public test artifact·matrix report·release input의 exact digest 일치 전 금지
- Release 뒤 발견된 compatibility defect: 먼저 reproducer와 matrix category 추가, 이후 patch candidate

## 단계별 수용

| 단계 | 필수 범위 | 실패 결과 |
| --- | --- | --- |
| Pull request | 변경한 contract의 unit·black-box regression, declared project source 전체 local matrix | merge gate 실패 |
| Candidate build | compiled binary·release bundle의 route/base coverage report, package sandbox matrix | public test 생성 금지 |
| Public test | prior stable·oldest distinct full-ledger project·multi-host user state의 representative acceptance | stable promotion 금지 |
| Stable promotion | candidate와 stable artifact·matrix report·acceptance evidence digest 동일성 | publication 금지 |

## 실행 checklist

- [x] `RQC95-001` release qualification schema·generator 추가: `check-project-base-coverage.py`가 migration route,
  prior stable user projection, public representative를 한 coverage inventory와 digest로 도출
- [x] `RQC95-002` compiled CLI black-box matrix 추가: local declared full-base source `0.9.1`·`0.9.2`·`0.9.3`에
  `scan`·`dry-run`·`apply`·`validate`, artifact의 exact base authentication·final ledger 검증
- [x] `RQC95-003` preservation/negative matrix 추가: local Hive edit three-way merge, user·foreign byte,
  missing·tampered base no-mutation, failure injection recovery, unsupported range rejection
- [x] `RQC95-004` release bundle·CI gate 연결: `release.yml` candidate가 migration-table route와 registry coverage
  matrix omission·stale report·compiled/package artifact digest mismatch의 candidate build 실패
- [x] `RQC95-005` risk declaration 연결: product·installer·projection·migration 변경은 CI product risk 분류·Rust
  request의 affected surface와 owning test ID 없이 merge·candidate 진행 불가
- [x] `RQC95-006` package sandbox qualification 추가: local npm pack·global install actual executable과 direct
  projection·declared project source representative의 actual executable·JSON result·validate 확인
- [x] `RQC95-007` release handoff·regression rule 추가: 보류된 release protocol이 public test와 stable promotion의
  report를 인용, post-release defect마다 reproducer·matrix category·owning future patch ID 기록

## 완료 기준

- Migration table의 모든 declared source version: exact embedded/authenticated base와 compiled CLI lifecycle 증거
- Public test·stable artifact: candidate coverage report와 exact digest 일치
- 범위·기준본·report·artifact 중 하나라도 불일치: stable publication 전 자동 중단
- Bug escape 재현 사례: smallest black-box regression과 release qualification inventory 항목 보유
- CI 통과: 선언 누락·유효하지 않은 기준본·stale evidence·패키지 차이의 통과 증거 아님

## 범위 경계

이 gate의 대상: Hive가 선언한 설치·갱신·projection·migration compatibility contract. Provider host의
비공개 동작, 모든 외부 plugin bug, 모든 논리 결함의 완전 제거 약속 제외. `REL95-*`: public test·stable
publication 전이의 단일 owner. `HBC95-*`·`AUP95-*`: 각각 historical project base·binary update 구현 owner.
