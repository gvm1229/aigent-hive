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

## `test.4` 준비 근거

- `32bf5df`: `0.9.2` historical `AGENTS.md`의 `wiki_backend` 재현과 local override 한 번 적용 뒤 validate 수렴
- `5f3bb93`: default stable 유지, explicit `hive update --channel test`·`--user-root`·`--confirm` 추가
- `45a9af8`: PortareFolium Hive-owned subset과 Codex 전용 시험 root의 public-artifact acceptance 실행기 추가
- 현재 Windows: PortareFolium `0.9.2` 48개 ledger projection·9개 support file copy의 `scan`·`dry-run`·`apply`·`validate`, local marker·foreign sentinel 보존, tampered ledger no-mutation 성공. Codex 전용 시험 root의 public `0.9.5-test.3` setup·install·validate 성공
- `0.9.5-test.4`: 위 product·acceptance 변경의 public artifact 수용. `test.4 → test.5`는 explicit test-channel user projection 갱신 수용
- `0.9.5-test.5`: direct installer의 optional signer fallback을 unresolved marker로 오판해 user projection 전 중단. `b8e4c79` 수정 뒤 `test.6 → test.7` 수용 필요
- `0.9.5-test.6`: npm 여섯 package·`test` tag·annotated tag 생성 뒤 GitHub Release API와 candidate artifact API가 HTTP 503. exact candidate 자산 회수 뒤 25-asset prerelease 복구 완료
- `0.9.5-test.7`: Windows dedicated test root의 direct `test.6 → test.7` 호출에서 실행 중 `hive.exe` 잠금으로 installer 교체 거부. `DUP95-001` 수정 뒤 `test.8 → test.9`에서 staged installer handle 유지로 Windows read sharing 거부 확인. `test.10 → test.11` 재수용 필요

## PortareFolium `0.9.2` fixture 수용 계획

대상: 실제 PortareFolium consumer의 `harness_version`·`project-base.json` `0.9.2` 확인. 원본 project와
Codex host configuration 변경 금지. 이 계획은 `REL95-004` 수용 증거 전용이며 `REL95-003` 사용자
projection·bare update 수용의 대체 근거 아님.

1. 원본 read-only preflight: canonical project root·`hive-source.json` 부재·48개 full-ledger entry·ledger digest·필수 support config 다이제스트 기록
2. 새 `tests/work/` fixture: ledger가 열거한 48개 Hive-owned projection과 `.hive/setup-answers.yml`, `harness.toml`, `capability-resolution.yml`, `active-skills.yml`, `approved-skills.yml`, `knowledge-scope.yml`, `project-overrides.json`, `role-seeds.yml`만 exact byte copy
3. 제외: project source·`.git`·`node_modules`·`.env*`·`.auth`·`.local`·`.omx`·`.omc`·`.hive/runtime`·`.hive/index`·knowledge·backup·host-global configuration. Fixture 전용 foreign sentinel 생성
4. source와 fixture의 allowlist manifest·SHA-256 비교 뒤 public npm `aigent-hive@0.9.5-test.3` binary로 `project upgrade --scan`, `--dry-run`, `--apply`, `--validate` 실행
5. fixture 별도 copy에서 shared-marker local addition 보존·foreign sentinel byte 불변·tampered ledger no-mutation 검증. debug-only fault injection은 public release 수용 근거 제외
6. 종료 조건: 원본 snapshot 변화 `0건`, public artifact 결과·source commit·Windows package digest·fixture manifest digest를 `REL95-004` evidence에 결속. 실패 시 stable promotion 중단

Codex plugin marketplace: fixture 경로 미사용. `project upgrade`에는 project-local ledger·setup
answers·capability resolution·harness만 필요. 기존 Codex marketplace ownership conflict: `REL95-003`
전역 user projection 검증의 별도 문제.

## `REL95-003` clean Codex profile 수용 계획

대상: Windows 11 Home과 M2 MacBook Air의 전용 시험 루트. 기존 maintainer profile의 marketplace·plugin·user-root
변경 금지. 여기서 marketplace는 Codex plugin marketplace이며 project upgrade fixture와 다른 전역 host configuration.

1. 시험 실행기: 고유 `tests/work` 루트 안의 Hive user root·Codex configuration root만 자식 프로세스에 주입. 외부 root 참조는 실행 전 실패
2. public `0.9.4` baseline의 user setup·Codex user install `--apply`·`--validate`로 authenticated user setup과 host manifest 생성
3. Codex configuration의 foreign marketplace·plugin entry와 user-root foreign sentinel 추가, Hive ownership 밖 byte snapshot 기록
4. `hive update --check` 조회 전용·mutation `0건` 확인. explicit `hive update --channel test`가 exact test target·owner·saved host refresh scope를 표시하는지 확인
5. 동의한 update 실행. owner install 뒤 새 executable만 `install --scope user --hosts <authenticated saved hosts> --apply`와 `--validate` 실행 여부 확인
6. exact public test version·Codex manifest·Hive-owned projection digest·foreign marketplace/plugin/sentinel byte 보존 확인. selected host에 없는 Claude activation `0건` 확인
7. Windows x64와 macOS arm64에서 같은 실행기 사용. macOS x86_64는 candidate CI archive evidence 유지

## 영구 회귀 보강과 후보 순서

1. PortareFolium acceptance와 별도로 `0.9.2` tag-derived golden project fixture 추가. 현재 renderer로 fixture를 생성하지 않고 frozen setup input·support config·full ledger·projection byte 사용
2. migration table의 declared full-base source 집합과 `FULL_HISTORICAL_PROJECT_BASE_VERSIONS`를 단일 assertion으로 비교. hard-coded version list 금지
3. golden fixture의 public-artifact-equivalent `scan`·`dry-run`·`apply`·`validate`, local shared-marker preservation, foreign sentinel preservation, tampered ledger no-mutation 회귀 추가
4. 이 보강은 acceptance 변경. source change 뒤 `0.9.5-test.4` candidate·public test 발행과 `REL95-004` 수용. `0.9.4`에는 explicit test-channel update 명령이 없으므로 `REL95-003`의 package-owner activation 증거로 사용 불가
5. `test.4`는 explicit `hive update --channel test`를 제공. `test.5` direct installer 검증 실패 뒤 `b8e4c79`를 포함한 `test.6` 발행. `test.6 → test.7`의 Windows 실행 파일 잠금 거부 뒤 `DUP95-001` 보강. `test.8 → test.9` staged installer handle 오류 뒤 `test.10 → test.11`로 `REL95-003` user projection refresh 수용
6. `REL95-003`과 `REL95-004` 모두 accepted 상태 뒤 `REL95-005` protected `main` integration 진행

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
