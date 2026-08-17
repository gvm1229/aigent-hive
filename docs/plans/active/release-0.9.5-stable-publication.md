# `0.9.5` 공개 시험·정식 출시

> Checklist owner: `REL95-*`
> 대상: `0.9.5` patch, 현재 활성 목표
> 선행: `HBC95-001–005`, `AUP95-001–006`, `RQC95-001–007` 완료

## 출시 결정

유지보수자 명시 승인: `0.9.5` 번호 공개 시험판·정식 출시·protected `main` 통합·현재 Windows
안정판 설치 진행. 이 문서는 `PLAN.md` active fragment·completion index의 release owner.

## 원칙

- Stable 채널: 탐색·회귀·수용 시험 경로 제외
- 번호 public test: exact `develop` commit·artifact digest·Windows x64 실제 수용의 유일한 제품 수용 근거
- public test 수용 뒤 product·package·installer·metadata 변경: 다음 번호 test와 영향 범위 재수용
- protected `main` 통합·stable publication: accepted test와 동일 product bytes 사용
- stable publication 뒤 현재 Windows x64 컴퓨터에 public stable artifact 설치·`hive --version`·user projection validate 확인

## Checklist

- [x] `REL95-001` 모든 `0.9.5` 구현·문서·정적·Rust·Python local gate 완료. exact `develop` source
  commit·release input·artifact digest 고정
- [x] `REL95-002` 고정 source에서 번호 public `0.9.5-test.N` prerelease 1회 게시. stable·`latest`
  mutation과 rebuild `0회`
- [ ] `REL95-003` Windows x64 실제 host에서 exact public test artifact의 clean install·preserving
  upgrade·validate·bare update path 수용. `AUP95` installed acceptance 포함
- [ ] `REL95-004` exact public test artifact에서 `0.9.2 → 0.9.5-test.N` project upgrade와 `HBC95`
  matrix 수용. artifact version·commit·digest·실행 결과 기록
- [ ] `REL95-005` latest accepted test source만 protected `main` workflow로 통합. stable input과 test
  input의 product bytes·release note·artifact digest 동등성 확인
- [ ] `REL95-006` `0.9.5` stable publication 뒤 stable-release-dependent test 전체 실행. current Windows x64에서
  exact public stable artifact 설치·`hive --version`·user projection `--validate`·public release/package 확인

## `0.9.5-test.3` 공개 시험 증거

- source: `develop@224170eb52a65b0259fa9bbef52dbfaf4c8701da`
- candidate: [run `31980927136`](https://github.com/gvm1229/aigent-hive/actions/runs/31980927136), 다섯 native target·npm umbrella·direct installer 성공
- publication: [run `31981450374`](https://github.com/gvm1229/aigent-hive/actions/runs/31981450374), exact candidate run ID 결속 성공
- public: [GitHub prerelease `v0.9.5-test.3`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.5-test.3), 25 assets, Windows zip SHA-256 `e60865181a8e2dc3850eb871676588aa285d50f70604b776a0af319e595f4288`
- registry: umbrella와 다섯 platform package의 exact `0.9.5-test.3`, `test=0.9.5-test.3`, `latest=0.9.4`
- Windows isolated acceptance: public direct installer와 npm `0.9.4 → 0.9.5-test.3` upgrade의 version identity 성공
- blocked: existing Codex marketplace ownership과 user installation manifest가 authenticated Hive release와 불일치. public test binary의 user projection validate·bare update 및 public `0.9.2` project setup은 mutation 없이 중단

## 수락 기준

- `0.9.5-test.N`: 공개·고유 번호·시험판 표기
- stable: accepted test artifact와 다른 제품 byte `0건`
- public artifact만 설치·업그레이드·전체 범위 수용
- 현재 Windows x64: stable artifact 설치와 validation 성공
- stable publication 뒤 의존 검사 성공

## 공개 시험판 증거 규약

- `Release candidate` 성공은 private artifact 생성만 의미. 공개 시험판 완료 표기 금지
- 공개 시험판은 exact candidate run ID를 받는 별도 게시 작업 성공, npm exact version·`test`
  tag, GitHub prerelease tag를 각각 외부 조회한 뒤에만 완료 처리
- historical Git tag 또는 commit을 읽는 두 출시 작업은 full checkout history 필수. `latest`
  태그는 test publication 뒤 이전 stable version과 동일 확인

## 호환성 인계

- candidate artifact의 `release-project-base-coverage.json`과 `coverage_digest`를 public test·stable
  handoff에 같은 값으로 인용
- public test·stable artifact의 source commit·package artifact·coverage report 불일치 시 promotion 중단
- release 뒤 호환성 결함은 smallest compiled CLI reproducer·coverage category·다음 patch checklist ID를
  함께 기록한 뒤에만 후속 후보 준비

## Ralph loop

- graph: [`v0.9.5-stable-release-loop.graph.md`](v0.9.5-stable-release-loop.graph.md)
- run ID: `v095-stable-release`
- retry: node별 최대 `3`회, 동일 failure 최대 `2`회
- dispatch: active host 소유. `prepared_only=true`, `spawned=false` 외 Hive 실행 경로 없음

## 범위 제외

- `NHA10-001–012`·`N10-002–011` `0.10.0` 후보
- stable 채널의 exploratory test
- provider API·credential 처리
