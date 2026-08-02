# `0.9.0` 시험·정식 릴리스 계획

> Target: `0.9.0`
> Checklist owner: `REL9-*`
> Decision: [`ADR-0017`](../../decisions/ADR-0017-0.9-full-release.md)
> Authority: 2026-08-01 사용자 요청의 full release 계획·원격 `develop` push 승인

## 목표

- 기본 시험판 `0.9.0-test`, 추가 시험판 선택 시에만 `0.9.0-test.N`
- npm `test`의 독립 배포와 기존 stable `latest` 보존
- 시험판 수용 뒤 별도 승인·시점의 protected `main` 기반 정식 `0.9.0`
- GitHub normal Release와 5개 native artifact·SHA-256·attestation·서명 증거
- npm 여섯 package exact `0.9.0`, `latest=0.9.0`
- 시험판·정식판의 기능·기본값·진단 계약 일치
- 소비자 공통 `report to developer`의 명시적 수집·preview·export, 자동 업로드 0건
- npm·Unix·PowerShell 5.1·CMD 설치와 `0.8.0 → 0.9.0` update 검증
- Release·npm·direct installer의 native binary byte identity
- Canonical source·release bundle·installed harness의 version·digest 계보

## 현재 기준선

- Source version·release date: `0.9.0`, `2026-08-01`
- Local qualification: Rust 459개 통과, Python 628개 통과·42개 platform skip
- RAG 50,000 chunk와 `.hivekb` 100 collection·50,000 chunk 성능 gate 통과
- `0.8.0` frozen project·user base와 synthetic release fixture 무변경
- Release notes: [`docs/releases/0.9.0.md`](../../releases/0.9.0.md)
- 기능 마감: [`v0.9.0-test-finalization.md`](v0.9.0-test-finalization.md)
- 기존 `release.yml`: remote `develop` candidate만 허용
- 기존 `release-publish.yml`: exact `0.8.0` hardcode와 `develop` candidate 결합
- GitHub normal Release·tag 생성 workflow 부재
- Repository release tag 부재
- Apple·Windows·external TUF production signer 증거 미확인
- Remote `develop`: `3c2c03dc792976c0acafb4377e8591e2a83ef848`

## 현재 프로젝트 상태 — 2026-08-03

- Git 기준선: local `develop` HEAD와 `origin/develop` exact
  `3c2c03dc792976c0acafb4377e8591e2a83ef848`
- 이 계획 변경 제외 작업 트리: tracked 수정 4개와 untracked Notion 구현·시험 2개;
  사용자 소유
- Local foreign state: `docs/.obsidian/`; 릴리스 범위 제외
- 부분 구현: `markdown|notion` setup schema·install 분기, provider-neutral Notion
  adapter·revision/tombstone·RAG projection과 fail-closed 시험
- 대상 검증: Notion backend 시험 3개 PASS, setup schema JSON parse PASS
- 미완료 검증: `cargo fmt --all --check` FAIL, `hive-cli` compile은
  `GlobalProjectPreferences.wiki_backend` field 부재로 FAIL
- Source Wiki: dirty source에 연결된 source digest mismatch 10건; qualification 전
  bilingual fact 갱신·index rebuild 필요
- 릴리스 identity: Cargo·harness template·release note의 `0.9.0` 정렬
- npm Trusted Publisher: platform 5개와 umbrella 1개 모두
  `gvm1229/aigent-hive`·`release-publish.yml`·`release-publication`으로 검증 완료
- 공개 npm: 6개 package 모두 `latest=0.8.0`; `0.9.0` publication 0건
- GitHub Actions: 2026-07-31 이후 candidate·publish run 0건; 최신 성공 계보는
  `0.8.0` commit `420e244577b6b4c340a0b03074cdf019edb348f2`
- Credential 전환: `release-publication`의 `NPM_TOKEN` 유지; OIDC 실제 게시 성공과
  rollback 확인 전 삭제 금지
- Checklist 판정: 신규 완료 증거 0건, `TST9-*` 부분 구현 상태,
  `REL9-001–026` 중 `REL9-001`만 완료

## Version·channel 계약

- Product version: `0.9.0`
- 기본 시험 package version: `0.9.0-test`
- 선택형 추가 시험 package version: `0.9.0-test.1`, `0.9.0-test.2`, …
- 시험 배포: npm dist-tag `test`, GitHub prerelease, stable `latest` 변경 0건
- 정식 배포: npm dist-tag `latest`, GitHub normal Release, 시험 배포와 동시 실행 금지
- npm immutability에 따른 bare 시험판 재게시 금지, 변경 시 numbered suffix 사용
- 시험판 전용 기능·기본값·logging·build flag 0건
- 시험판 publication의 정식 workflow trigger·tag·`latest` mutation 0건

## Artifact 계보

```text
remote develop qualification
  → 0.9.0-test[.N] candidate
  → GitHub prerelease + npm test
  → 수용 기간·필요 시 numbered test 반복
  → develop → main PR·required CI
  → protected main exact stable candidate
  → 5 native archives + 6 npm tarballs
  → SHA-256 + GitHub attestation + platform signing
  → external TUF production metadata
  → annotated v0.9.0 tag on the same main commit
  → GitHub Release + npm 0.9.0|latest
  → public clean install·0.8.0 update verification
```

- `develop` artifact: pre-integration qualification 전용
- `main` artifact: publication 대상 최종 계보
- 각 시험판의 candidate·GitHub prerelease·npm `test` exact commit 일치
- Stable candidate·tag·GitHub normal Release·npm `latest` exact commit 일치
- Merge 뒤 재빌드 없는 develop artifact 재사용 금지
- Publish workflow의 untrusted input·run·branch·SHA·attestation 재검증

## 구현·검증 checklist

### A. 기준선과 workflow activation

- [x] [REL9-001] 현재 implementation·계획·변경점 커밋의 원격 `develop` 반영과 exact remote SHA `e4f1d6001a0a6ad5f41dccc350a0e585bbe9c9d0` 확인
- [ ] [REL9-002] `0.9.0` product identity와 stable·bare test·numbered test grammar, 문서·fixture·release notes parity
- [ ] [REL9-003] package version을 product version과 분리하고 `0.9.0-test[.N]` parser·installer·receipt·upgrade 계약 구현
- [ ] [REL9-004] `release.yml` candidate를 explicit channel·version·ref·SHA에 결합하고 시험·정식 ref downgrade 차단
- [ ] [REL9-005] 독립 test prerelease와 stable normal Release workflow 추가: 상호 trigger 0건, tag·asset·checksum·attestation·idempotency·existing-version refusal

### B. Clean-clone qualification

- [ ] [REL9-006] Fresh clone의 Rust format·strict Clippy·workspace all-target·all-feature test와 Python 전체 적합성
- [ ] [REL9-007] Ubuntu·macOS·Windows CI와 Linux musl x86_64·arm64 release runtime PASS
- [ ] [REL9-008] Installer·update·rollback·recovery·secret·symlink·path confinement hostile suite PASS
- [ ] [REL9-009] 시험·정식 feature/default parity, `markdown|notion` backend,
  Discord outbound와 공통 `report to developer` preview·collect·보존·redaction·no-upload conformance
- [ ] [REL9-010] RAG 50,000 chunk와 `.hivekb` 100 collection·50,000 chunk release profile 재측정·threshold PASS
- [ ] [REL9-011] Codex·Antigravity 실제 install·setup·project·update 회귀와 Claude fixture·미검증 범위 공개

### C. 시험 배포와 수용

- [ ] [REL9-012] Remote `develop` exact SHA의 `0.9.0-test` candidate 5 target·6 npm·installer·byte identity PASS
- [ ] [REL9-013] 기본 시험판의 GitHub prerelease·npm `0.9.0-test|test` 독립 게시와 기존 `latest` snapshot 불변 확인
- [ ] [REL9-014] Public test install·update·문제 보고 기능과 stable parity acceptance
- [ ] [REL9-015] 수용 기간의 blocker triage·privacy·disk retention 검증, bare 시험판 재게시 금지 확인
- [ ] [REL9-016] 추가 시험판 필요 시에만 `0.9.0-test.N|test` 생성·검증·이전 시험판 계보 보존
- [ ] [REL9-017] 시험 수용 뒤 `develop → main` PR required CI·review·non-force merge와 exact main SHA 고정

### D. 정식 candidate·publication·public acceptance

- [ ] [REL9-018] Protected `main` exact SHA의 stable candidate 재빌드·attestation·artifact inventory PASS
- [ ] [REL9-019] macOS·Windows signing과 external TUF production authorization·rollback floor 검증
- [ ] [REL9-020] Final main SHA의 annotated `v0.9.0`과 GitHub normal Release·signed artifact 게시
- [ ] [REL9-021] npm platform 5개 선행·umbrella 최종 OIDC publication과 `latest=0.9.0`, `test` 보존 확인
- [ ] [REL9-022] npm·Unix·PowerShell 5.1·CMD public clean install·repeat·pending receipt recovery PASS
- [ ] [REL9-023] `0.8.x → 0.9.0`과 `0.9.0-test[.N] → 0.9.0`의 knowledge·preference·foreign byte 보존과 SQLite rebuild PASS
- [ ] [REL9-024] GitHub·npm·direct binary byte identity·provenance와 public update discovery·consent PASS
- [ ] [REL9-025] Release·registry·installer 관찰, critical 회귀 0건과 rollback 판정
- [ ] [REL9-026] PLAN·CURRENT·release notes·ADR·bilingual fact에 run ID·SHA·digest·지원·미검증 범위 최종 반영

## 실행 순서

1. `TST9-*`·`PRF-*` 기능 마감과 release handoff
2. `REL9-001` 원격 `develop` 기준선 재고정
3. `REL9-002–005` version grammar·분리 workflow 구현·독립 커밋
4. `REL9-006–012` clean clone·cross-platform 시험 후보 검증
5. `REL9-013–016` bare 시험판 독립 게시·수용·선택형 numbered 시험판
6. `REL9-017–024` main 통합·stable candidate·signing·별도 정식 publication
7. `REL9-025–026` 관찰·current-truth 완료 기록

## 외부 권한 경계

- `main` PR review·merge와 protected environment approval
- Apple·Windows signing identity와 external TUF threshold signer
- GitHub prerelease·normal Release 생성 권한과 npm Trusted Publisher environment
- Credential·private key·2FA material의 저장소·agent 노출 금지
- 각 외부 mutation 직전 exact SHA·artifact digest·대상 preview 재확인

## 완료 기준

- `REL9-001–026` 전부 evidence-backed 완료
- 시험 `test`와 stable `latest`의 독립 mutation·exact commit 증거
- GitHub tag·Release·npm `latest`의 exact `0.9.0`·main SHA 일치
- 5개 platform artifact·6개 npm package·3개 direct installer 검증
- `0.8.0` 사용자 데이터·설정·project harness의 non-breaking upgrade
- Signing·provenance·TUF·rollback·public acceptance의 미확인 항목 0건
