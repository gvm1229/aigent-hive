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

- [ ] [REL94-001] 모든 `0.9.4` 구현·문서·정적·Rust·Python local gate 완료, exact `develop` source
  commit·release input·artifact digest 고정
- [ ] [REL94-002] 고정 source에서 번호가 붙은 public `0.9.4-test.N` GitHub prerelease 1회 게시.
  stable·`latest` mutation과 rebuild `0회`
- [ ] [REL94-003] Windows x64 실제 host에서 exact public test artifact의 clean install·preserving
  upgrade·validate·update path 수용. `UPV94`·`KRV94`·`PML94` installed acceptance 포함
- [ ] [REL94-004] exact public test artifact로 `SID94`·`HGD94`·`RNL94` 전 범위 수용과 artifact
  version·commit·digest·실행 결과 기록
- [ ] [REL94-005] latest accepted test source만 protected `main` workflow로 통합. stable input과
  test input의 product bytes·release note·artifact digest 동등성 확인
- [ ] [REL94-006] `0.9.4` stable publication 뒤 stable-release-dependent test 전체 실행과
  public release·package·Windows x64 installer observation 기록

## 수락 기준

- `0.9.4-test.N`은 공개·고유 번호·시험판 표기
- stable은 accepted test artifact와 다른 제품 byte `0건`
- public artifact로만 설치·업그레이드와 complete scope 수용
- stable publication 뒤 의존 검증 성공

## 범위 제외

- `0.10.0` 후보 구현·게시
- stable 채널의 exploratory test
- provider API·credential 처리
