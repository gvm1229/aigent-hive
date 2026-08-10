# `0.9.0` 시험·정식 릴리스 계획

> Target: `0.9.0`
> Checklist owner: `REL9-*`
> Decision: [`ADR-0017`](../../decisions/ADR-0017-0.9-full-release.md)
> Authority: 2026-08-01 사용자 요청의 full release 계획·원격 `develop` push 승인

## 목표

- `0.9.0-test[.N]` 독립 시험 배포와 stable `latest` 보존
- 시험 수용 뒤 protected `main` exact commit 기반 정식 `0.9.0`
- GitHub normal Release·5개 native artifact·6개 npm package·3개 direct installer의
  SHA-256·attestation·서명·byte identity
- 시험·정식 기능·기본값·진단 일치와 명시적 `report to developer`·자동 업로드 0건
- `0.8.0 → 0.9.0` 무손실 update와 source·release·installed version·digest 계보

## 현재 기준선

- Product: `0.9.0`; notes: [`0.9.0.md`](../../releases/0.9.0.md)
- 최신 공개 시험판: `0.9.0-test.7`; 여섯 npm package `test=0.9.0-test.7`, `latest=0.8.0`
- `0.9.0-test.7`: exact `567cce0` 기반 GitHub prerelease와 npm `test` 게시 확인
- 다음 시험판: 소비자 자율 실행 규칙 `0978a6e`를 포함한 `0.9.0-test.8`; `test`만 변경,
  `latest=0.8.0` 유지
- 상세 run·failure·external signer evidence: [`CURRENT.md`](../../state/CURRENT.md)

## 이번 정식 릴리스의 명시적 제외와 면제

- Codex 실제 plugin 활성화·global setup: `REL9-011` 필수 출시 gate
- Antigravity 실제 설치·설정·프로젝트·업데이트 회귀와 Claude fixture 공개: 유지보수자 요청에 따른 제외 유지. 해당 host 사용성·호환성 검증 완료 주장 금지
- `REL9-017`의 CI: 유지보수자 요청에 따른 병합 gate 일시 면제. 실패·미실행 CI는 통과 증거가 아니며 `CURRENT.md`에 정확한 실패 범위 기록

## Codex plugin 활성화 장애

- 관찰: `0.9.0-test.5`의 Codex plugin 활성화 실패와 부분 설치 되돌리기 완료
- 현재 환경: Codex CLI `0.146.1`; `hive setup --scope user --help` 제공 확인
- 원인 판정: 실제 호스트 명령 응답·marketplace package·활성화 이후 상태의 전환 구간 미검증
- 해결 범위: 현재 Codex JSON 계약에 맞춘 adapter·parser·version qualification, 임시
  `plugin → codex plugin` bridge와 호스트 전역 설정 수동 쓰기 제거
- local 재현·수정 증거: Codex CLI `0.147.0`, macOS 격리 user root에서 `/tmp`가 host JSON의
  `/private/tmp`와 달라 activation이 중단됨. no-follow user root 확인 뒤 physical path 정규화 적용,
  `install → setup dry-run → setup apply → setup validate → install validate`와 structured list PASS
- 회귀 흐름: 격리 user root의 marketplace add → plugin add → 구조화 목록 검증 →
  `hive setup --scope user` dry-run·apply·validate → 실패 되돌리기·foreign byte 보존
- 출시 조건: 수정된 numbered 시험판의 Windows clean install·fresh Codex session 수용 뒤 stable 진행.
  macOS local evidence는 구현 회귀 증거이며 Windows 수용의 대체 근거 아님

## Version·channel 계약

- 상세 정본: [`ADR-0017`](../../decisions/ADR-0017-0.9-full-release.md)
- 시험: `0.9.0-test[.N]`, npm `test`, GitHub prerelease, stable `latest` 변경 0건
- 정식: exact `0.9.0`, npm `latest`, GitHub normal Release, 시험과 동시 게시 금지
- Bare 시험판 재게시·시험판 전용 기능·기본값·진단·정식 workflow trigger 0건
- 시험·정식 publication: 하나의 `release-publish.yml`에서 `channel=test|stable` 입력으로만 분기
- 게시 인증: 여섯 npm package의 동일 GitHub Actions Trusted Publisher·OIDC만 사용, write token 경로 0건

## Artifact 계보

- 흐름: `develop` 시험 후보 → prerelease·npm `test` → 수용 → protected `main` stable 후보 →
  signing·TUF → `v0.9.0` normal Release·npm `latest`
- `develop` artifact: 사전 검증 전용. `main` artifact: 정식 게시 전용
- Channel별 candidate·tag·Release·npm exact commit 일치와 develop artifact 재사용 금지

## 구현·검증 checklist

### A. 기준선과 workflow activation

- [x] [REL9-001] release source baseline `cee06e013cfbeca907c018b26c35a89bee0b703b`의 `develop` 포함 확인
- [x] [REL9-002] `0.9.0` product identity와 stable·bare test·numbered test grammar, 문서·fixture·release notes parity
- [x] [REL9-003] package version을 product version과 분리하고 `0.9.0-test[.N]` parser·installer·receipt·upgrade 계약 구현
- [x] [REL9-004] `release.yml` candidate를 explicit channel·version·ref·SHA에 결합하고 시험·정식 ref downgrade 차단
- [x] [REL9-005] 독립 test prerelease와 stable normal Release workflow 추가: 상호 trigger 0건, tag·asset·checksum·attestation·idempotency·existing-version refusal

### B. Clean-clone qualification

- [x] [REL9-006] Fresh clone의 Rust format·strict Clippy·workspace all-target·all-feature test와 Python 전체 적합성
- [x] [REL9-007] Ubuntu·macOS·Windows CI와 Linux musl x86_64·arm64 release runtime PASS
- [x] [REL9-008] Installer·update·rollback·recovery·secret·symlink·path confinement hostile suite PASS
- [x] [REL9-009] 시험·정식 feature/default parity, `markdown|notion` backend,
  Discord outbound와 공통 `report to developer` preview·collect·보존·redaction·no-upload conformance
- [x] [REL9-010] RAG 50,000 chunk와 `.hivekb` 100 collection·50,000 chunk release profile 재측정·threshold PASS
- [ ] [REL9-011] Codex CLI `0.146.1`의 실제 marketplace·plugin 활성화, structured state
  검증, `hive setup --scope user` dry-run·apply·validate, 실패 되돌리기와 fresh-session
  discovery 회귀 통과. 임시 bridge·호스트 전역 설정 수동 쓰기 0건

### C. 시험 배포와 수용

- [x] [REL9-012] Remote `develop` exact SHA의 `0.9.0-test` candidate 5 target·6 npm·installer·byte identity PASS
- [x] [REL9-013] 기본 시험판의 GitHub prerelease·npm `0.9.0-test|test` 독립 게시와 기존 `latest` snapshot 불변 확인
- [ ] [REL9-014] Public test install·update·문제 보고 기능과 stable parity acceptance
- [ ] [REL9-015] 수용 기간의 blocker triage·privacy·disk retention 검증, bare 시험판 재게시 금지 확인
- [x] [REL9-016] 추가 시험판 필요 시에만 `0.9.0-test.N|test` 생성·검증·이전 시험판 계보 보존
- [ ] [REL9-017] 시험 수용 뒤 `develop → main` PR·non-force merge와 exact main SHA 고정. CI 결과 기록은 유지하되 이번 `0.9.0` 병합 gate 제외

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
- [x] [REL9-027] `release-publish.yml` 단일 OIDC publication: `channel=test|stable`별 exact branch·candidate·tag·Release guard, `NPM_TOKEN` fallback 제거, six-package Trusted Publisher 설정 안내와 static contract 검증 — `0.9.0-test.6`의 six-package OIDC publication PASS
- [x] [REL9-028] Copier와 Rust harness renderer가 Discord `message_fields` 기본값을 byte-exact하게 동일 출력하는 parity PASS — `Copier 9.17.0` 격리 검증에서 4개 parity PASS

## 실행 순서

1. `TST9-*`·`PRF-*` 기능 마감과 release handoff
2. `REL9-001` 원격 `develop` 기준선 재고정
3. `REL9-002–005` version grammar·분리 workflow 구현·독립 커밋
4. `REL9-006–012` clean clone·cross-platform 시험 후보와 Codex 실제 활성화 검증
5. `REL9-013–016` bare 시험판 독립 게시·수용·선택형 numbered 시험판
6. `REL9-027` 단일 OIDC publication 구현·Trusted Publisher 설정
7. `REL9-028` Copier·Rust Discord 설정 parity 복구
8. `REL9-017–024` main 통합·stable candidate·signing·별도 정식 publication
9. `REL9-025–026` 관찰·current-truth 완료 기록

## 외부 권한 경계

- `main` PR review·merge와 protected `release-publication` approval
- Apple·Windows signing identity와 external TUF threshold signer
- GitHub App write 권한·npm Trusted Publisher·test workflow 등록 권한
- Credential·private key·2FA material 노출 금지와 외부 mutation 직전 exact 대상 재확인

## 완료 기준

- 모든 in-scope `REL9-*` evidence-backed 완료
- 시험 `test`와 stable `latest`의 독립 mutation·exact commit 증거
- GitHub tag·Release·npm `latest`의 exact `0.9.0`·main SHA 일치
- 5개 platform artifact·6개 npm package·3개 direct installer 검증
- `0.8.0` 사용자 데이터·설정·project harness의 non-breaking upgrade
- Signing·provenance·TUF·rollback·public acceptance의 미확인 항목 0건
