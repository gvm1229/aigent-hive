# `0.9.1` 최종 호환 patch 공개 계획

> Parent: [`release-0.9.0.md`](release-0.9.0.md)
> Checklist owner: `REL9-019–030`
> Target: `0.9.1`

## 확정 결정

- 핵심 목표: 자주 반복 가능한 빠른 publication
- GitHub·npm의 이중 승인 금지. Stable mutation 승인 지점: protected GitHub
  `release-publication` environment 한 곳
- 필수 release trust:
  1. protected `main` exact commit과 annotated `v0.9.0` tag
  2. 동일 candidate byte의 GitHub Release 승격, SHA-256 sidecar, GitHub artifact attestation
  3. npm Trusted Publishing OIDC와 registry provenance, 장기 `NPM_TOKEN` 없음
  4. macOS ad-hoc seal의 정확한 공개, Windows unsigned의 정확한 공개
- macOS ad-hoc: publisher identity·Apple notarization 아님. Binary 변경 감지만 제공
- 유료·외부 선택 기능: Developer ID·notarization, Authenticode·SignPath. Stable gate 아님
- 제거 대상: release TUF, offline root·threshold signer, external authorization request,
  rollback floor metadata, platform certificate evidence gate, 별도 signing ceremony
- 유지 대상: update 전 backup, atomic activation, crash recovery, 실패 rollback, 기본 downgrade
  거부. Release TUF의 metadata rollback floor와 별개
- 별도 보존 경계: judge용 external trust root와 frozen historical release base. 현재 release
  trust가 아니며 active guidance·current code path에서 참조 금지

## 선행 정리·구현·게시

- [x] [REL9-019] Active code·workflow·schema·fixture·guide·decision·plan·fact 전체 inventory와
  allowlist 고정. Release TUF·external signer·platform certificate gate 참조 제거, judge trust와
  frozen historical base만 명시적 예외. Current public surface의 금지 참조 `0건`
- [x] [REL9-020] Release·update 구현 단순화. TUF CLI·metadata·authorization script·workflow input·
  schema·fixture 제거. `hive release verify`: local bundle version·length·SHA-256 검증만 유지.
  npm registry integrity 또는 GitHub attestation을 설치 출처 trust로 사용
- [x] [REL9-021] Candidate·publication workflow 단순화. Protected `main` exact SHA에서 한 번 build,
  5개 native archive·6개 npm package·3개 installer의 byte identity·sidecar·attestation 생성,
  stable environment 승인 뒤 rebuild 없는 GitHub Release·npm `latest` 승격
- [x] [REL9-022] Public install·update 계약 정리. macOS ad-hoc·Windows unsigned 상태 공개,
  paid code signing을 optional enhancement로 분리. Transactional backup·rollback·recovery와
  same-major migration 보존
- [x] [REL9-023] Usage guard onboarding 보정 완료. 활성화 권장·신속 기본 `20%`, CodexBar의
  정상 setup 노출 `0건`, native 실제 실패 뒤에만 별도 fallback 동의
- [x] [REL9-024] User projection·uninstall 수렴 보정 완료. 폐기·개명 Skill file과 중첩 빈
  directory, Hive-owned transient `.hive` artifact 정리. Knowledge·saved preference·foreign byte와
  별도 developer rollback state 보존
- [x] [REL9-025] Replacement stable candidate와 publication 전 acceptance. `0.8.x`·numbered test upgrade,
  npm·Unix·PowerShell 5.1·CMD clean/repeat install, setup·uninstall·reinstall, usage sensor,
  source date·version·byte identity 검증. Consumer·user-root Wiki lint는 `hive-source.json`
  없이 실행. Source-workspace lint 혼동·건너뜀 `0건`. `0.9.1` 게시 전 `KAC-001·007–008`,
  `RAG-005·020`, `UOS-019·022`의 증거 기반 마감. Publication 자체를 소유한 `KAC-012`는
  `REL9-026`과 함께 게시 뒤 마감. Root English `README.md` 기반 npm
  umbrella package README 생성·tarball 수록·link와 asset 변환 검증 포함. Codex 설치 화면의
  developer `Hojin (Tom) Jeong`과 중앙 확대 공식 Hive logo 표시 검증 포함
- [x] [REL9-026] Protected stable publication·관찰. Annotated tag·normal GitHub Release·six-package
  OIDC `latest`, installer·update 확인, critical 회귀 `0건`, PLAN·CURRENT·release notes·ADR·fact 마감

## 보존된 완료 증거

- [x] [REL9-027] `release-publish.yml` single OIDC publication, `NPM_TOKEN` fallback 제거,
  six-package Trusted Publisher 실제 test publication PASS
- [x] [REL9-028] Copier·Rust Discord `message_fields` byte-exact parity PASS
- [x] [REL9-030] Python lane·owner·contract·release gate 대장과 `os × lane` CI matrix 완료

## 실행 순서

1. Active/current release trust 정리: `REL9-019–022`
2. Onboarding·fallback·purge 보정: `UGP-007–008`, `NUS-026–028`, `UOS-020–022`
3. 정적 금지 참조 scan, targeted·workspace·cross-platform 회귀
4. Knowledge autocapture `KAC-*`와 replacement stable candidate
5. Protected environment 1회 승인, GitHub Release·npm `latest` 연속 승격
6. Public install·update 관찰과 current-truth 마감

## npm README 동기화 출시 범위

1. Root English `README.md`를 public umbrella package `README.md`의 정본으로 사용
2. `QA Contributors` 구간 제거와 npm에서 동작하는 절대 GitHub link·asset 변환
3. 생성 package와 tarball에서 본문 포함·QA 표 제외·상대 link 부재 회귀 검증

범위: package generator와 conformance test 변경을 `0.9.1` 후보·실제 npm tarball 수용에 포함.
Registry·version·release tag·GitHub Release·dist-tag mutation은 `REL9-026`의 동일 후보 공개
경계에서만 실행.

현재 로컬 검증: `python -m unittest tests.conformance.test_npm_packaging` 3개 PASS.
출시 검증: candidate artifact와 npm registry의 `aigent-hive@0.9.1` tarball README 동일성 PASS.

Publication 전 수용 증거: candidate run `31574492371`의 7개 작업 PASS, Windows user-root
preserving reinstall·validate·release date `2026-08-12`·retired empty shell `0건`, npm candidate
README와 root English README 동기화, plugin author·logo manifest·asset byte 검증, fresh Codex
ordinary prompt의 자동 기록과 다음 session 자동 회수 PASS. Codex 설치 화면의 app-level 재표시는
게시 뒤 실제 npm `latest` 재설치 수용에서 확인.

## 외부 권한 경계

- `main` PR review·merge
- Stable GitHub environment approval 한 번
- npm package별 Trusted Publisher 등록 유지
- Credential·private key·2FA material의 agent·repository 입력 금지

## 완료 기준

- Active code·docs의 obsolete release trust 참조 `0건`
- Judge trust·frozen historical base 외 TUF 참조 `0건`
- 5개 native archive·6개 npm package·3개 installer의 same-candidate byte identity
- SHA-256·GitHub attestation·npm provenance 검증
- macOS ad-hoc·Windows unsigned disclosure와 optional paid signing의 gate 분리
- Usage guard 권장 기본·failure-only CodexBar·projection purge의 실제 clean-root 수용
- Public install·update·rollback·recovery·data preservation PASS
