# `0.9.5` 공개 시험·정식 출시 보류 프로토콜

> Checklist owner: `REL95-*`
> 대상: `0.9.5` patch, 현재 활성 목표 제외
> 선행: `HBC95-001–005`, `AUP95-001–006`, `RQC95-001–007` 완료

## 보류 결정

유지보수자 지시: `0.9.5`의 로컬 구현·검증만 진행. 번호 공개 시험판·정식 출시·protected `main`
통합·현재 Windows 안정판 설치는 수행 금지. 이 문서는 다음 명시 재승인 때 사용할 배포 프로토콜이며
`PLAN.md`의 active fragment·completion index에 포함하지 않음.

## 재개 시 원칙

- Stable 채널: 탐색·회귀·수용 시험 경로 제외
- 번호 public test: exact `develop` commit·artifact digest·Windows x64 실제 수용의 유일한 제품 수용 근거
- public test 수용 뒤 product·package·installer·metadata 변경: 다음 번호 test와 영향 범위 재수용
- protected `main` 통합·stable publication: accepted test와 동일 product bytes 사용
- stable publication 뒤 현재 Windows x64 컴퓨터에 public stable artifact 설치·`hive --version`·user projection validate 확인

## 재개 시 checklist

- [ ] `REL95-001` 모든 `0.9.5` 구현·문서·정적·Rust·Python local gate 완료. exact `develop` source
  commit·release input·artifact digest 고정
- [ ] `REL95-002` 고정 source에서 번호 public `0.9.5-test.N` prerelease 1회 게시. stable·`latest`
  mutation과 rebuild `0회`
- [ ] `REL95-003` Windows x64 실제 host에서 exact public test artifact의 clean install·preserving
  upgrade·validate·bare update path 수용. `AUP95` installed acceptance 포함
- [ ] `REL95-004` exact public test artifact에서 `0.9.2 → 0.9.5-test.N` project upgrade와 `HBC95`
  matrix 수용. artifact version·commit·digest·실행 결과 기록
- [ ] `REL95-005` latest accepted test source만 protected `main` workflow로 통합. stable input과 test
  input의 product bytes·release note·artifact digest 동등성 확인
- [ ] `REL95-006` `0.9.5` stable publication 뒤 stable-release-dependent test 전체 실행. current Windows x64에서
  exact public stable artifact 설치·`hive --version`·user projection `--validate`·public release/package 확인

## 재개 시 수락 기준

- `0.9.5-test.N`: 공개·고유 번호·시험판 표기
- stable: accepted test artifact와 다른 제품 byte `0건`
- public artifact만 설치·업그레이드·전체 범위 수용
- 현재 Windows x64: stable artifact 설치와 validation 성공
- stable publication 뒤 의존 검사 성공

## 재개 시 호환성 인계

- candidate artifact의 `release-project-base-coverage.json`과 `coverage_digest`를 public test·stable
  handoff에 같은 값으로 인용
- public test·stable artifact의 source commit·package artifact·coverage report 불일치 시 promotion 중단
- release 뒤 호환성 결함은 smallest compiled CLI reproducer·coverage category·다음 patch checklist ID를
  함께 기록한 뒤에만 후속 후보 준비

## 이전 Ralph loop

- graph: [`v0.9.5-release-loop.graph.md`](v0.9.5-release-loop.graph.md)
- run ID: `v095-stable-release` (현재 비활성)
- retry: node별 최대 `3`회, 동일 failure 최대 `2`회
- dispatch: active host 소유. `prepared_only=true`, `spawned=false` 외 Hive 실행 경로 없음

## 범위 제외

- `NHA10-001–012`·`N10-002–011` `0.10.0` 후보
- stable 채널의 exploratory test
- provider API·credential 처리
