# `0.9.4` 시험·정식 출시

> Checklist owner: `REL94-*`
> 대상: `0.9.4` patch
> 선행: `SID94-*`, `UPV94-*`, `KRV94-*`, `HGD94-*`, `RNL94-*`, `PML94-*` 완료

## 원칙

- Stable 채널은 시험 경로 금지
- 번호가 붙은 공개 시험판은 exact `develop` commit·artifact digest·Windows x64 실제 수용의 유일한 제품 수용 근거
- 시험 수용 뒤 product·package·installer·metadata 변경 발생 시 새 번호 시험판과 영향 범위 재수용 필요
- protected `main` 통합과 stable publication은 시험 수용 뒤 진행, 동일 product bytes 사용

## Checklist

- [x] [REL94-001] 모든 `0.9.4` 구현·문서·정적·Rust·Python local gate 완료, exact `develop` source
  commit·release input·artifact digest 고정
- [x] [REL94-002] 고정 source에서 번호가 붙은 public `0.9.4-test.N` GitHub prerelease 1회 게시.
  stable·`latest` mutation과 rebuild `0회`
- [x] [REL94-003] Windows x64 실제 host에서 exact public test artifact의 clean install·preserving
  upgrade·validate·update path 수용. `UPV94`·`KRV94`·`PML94` installed acceptance 포함
- [x] [REL94-004] exact public test artifact로 `SID94`·`HGD94`·`RNL94` 전 범위 수용과 artifact
  version·commit·digest·실행 결과 기록
- [x] [REL94-005] latest accepted test source만 protected `main` workflow로 통합. stable input과
  test input의 product bytes·release note·artifact digest 동등성 확인
- [x] [REL94-006] `0.9.4` stable publication 뒤 stable-release-dependent test 전체 실행과
  public release·package·Windows x64 installer observation 기록

## 수락 기준

- `0.9.4-test.N`은 공개·고유 번호·시험판 표기
- stable은 accepted test artifact와 다른 제품 byte `0건`
- public artifact로만 설치·업그레이드와 complete scope 수용
- stable publication 뒤 의존 검증 성공

## Public test acceptance evidence

- source: `cc50bcbe28c771d9f176b27791086b7d05ea3b3d`; candidate run `31765987540`; publication run `31766521620`
- public prerelease: [`v0.9.4-test.1`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.4-test.1), Windows zip SHA-256 `e9fd97fd11535fad9d4ddaac0551f68a2fb23f3d86e1a6f5313667bf0760f411`
- Windows x64: public direct installer clean install, npm `0.9.3 → 0.9.4-test.1` upgrade, preserving reinstall, install/setup validation, enabled `hive update --check` current receipt 확인
- installed scope: safe knowledge receipt 성공·credential-shaped input mutation-free rejection, 26 Skill description ID-first, Korean response directive와 English-default·explicit-Korean prompt contract 확인
- public tag source: HTML·PDF·release note blob과 accepted source 일치; PDF pagination visual inspection, English-first bilingual Release body와 fact ID parity 확인

## Stable publication evidence

- protected integration: PR [#33](https://github.com/gvm1229/aigent-hive/pull/33), merge commit `8b37323daa33b96918933ad629d7c709c3cb6679`; full CI와 protected merge gate 통과
- stable candidate: run `31767805733`, product/package `0.9.4`, release note match, accepted test source 대비 product input diff `0건`
- stable publication: run `31768342121`, public [v0.9.4](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.4), six npm package `latest=0.9.4`
- post-publication: Windows x64 public direct installer·npm `latest`·released zip version 확인, zip SHA-256 `d2a78d7c70613178c9e442a5ce861b4a37e371befa4775218661800b0da2ec93`, public integrity bundle `hive.release-verified` source `8b37323`·sequence `13`
- stable-release-dependent release lane: 40 pass; Windows host unavailable macOS 7건·POSIX shell 1건 skip

## 범위 제외

- `0.10.0` 후보 구현·게시
- stable 채널의 exploratory test
- provider API·credential 처리
