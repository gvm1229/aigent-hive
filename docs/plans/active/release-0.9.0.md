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
- 최신 공개 시험판: `0.9.0-test.16`; 여섯 npm package `test=0.9.0-test.16`, `latest=0.8.0`
- `0.9.0-test.16` candidate [`31514244763`](https://github.com/gvm1229/aigent-hive/actions/runs/31514244763):
  exact `d4ffa337586733fabdecf6a8e0eeca309091de1e`, 5개 native target·npm umbrella·direct installer·attestation PASS
- `0.9.0-test.16` publication [`31515563254`](https://github.com/gvm1229/aigent-hive/actions/runs/31515563254):
  six-package OIDC `test` 게시, annotated `v0.9.0-test.16`, GitHub prerelease, `latest=0.8.0` 유지 PASS
- Windows global npm install: `AIgent Hive v0.9.0-test #16` 확인. `test.13` ownership manifest 충돌의
  Hive 보존형 uninstall→saved-preference reinstall→`hive install --validate` PASS. knowledge·saved preference
  보존, Codex 매 턴 `remember`·receipt 안내와 automatic `knowledge-capture` 표시 확인. Fresh Codex session
  write·다음 session recall은 `KAC-007` 미수용
- `test.16` embedded release date `2026-08-01`: historical input 오류. 기존 byte·tag 불변, 별도 테스트
  배포 없이 다음 정상 배포에서 actual UTC date 입력·표시 검증 필요
- 상세 run·failure·publication evidence: [`CURRENT.md`](../../state/CURRENT.md)

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
  `/private/tmp`와의 physical path 불일치로 인한 activation 중단. no-follow user root 확인 뒤 physical path 정규화 적용,
  `install → setup dry-run → setup apply → setup validate → install validate`와 structured list PASS
- 회귀 흐름: 격리 user root의 marketplace add → plugin add → 구조화 목록 검증 →
  `hive setup --scope user` dry-run·apply·validate → 실패 되돌리기·foreign byte 보존
- 출시 조건: 잠복 Codex plugin activation 보존 수정이 포함된 numbered 시험판의 Windows clean
  install·fresh Codex session 수용 뒤 stable 진행.
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
  integrity·attestation 검증 → `v0.9.0` normal Release·npm `latest`
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
- [x] [REL9-011] Codex CLI `0.146.1`의 실제 marketplace·plugin 활성화, structured state
  검증, `hive setup --scope user` dry-run·apply·validate, 실패 되돌리기와 새 Codex session의 자동 CLI
  탐색 회귀 통과. `0.9.0-test.12` actual clean reinstall·setup·data preservation·Skill·usage guard·persisted
  Discord 설정 완료. 유지보수자 제공 Windows 증거: 새 Codex session 자동 `hive` 탐색·Discord 실제 전달.
  임시 bridge·호스트 전역 설정 수동 쓰기 0건

### C. 시험 배포와 수용

- [x] [REL9-012] Remote `develop` exact SHA의 `0.9.0-test` candidate 5 target·6 npm·installer·byte identity PASS
- [x] [REL9-013] 기본 시험판의 GitHub prerelease·npm `0.9.0-test|test` 독립 게시와 기존 `latest` snapshot 불변 확인
- [x] [REL9-014] Public test install·update·문제 보고 기능과 stable parity acceptance. `0.9.0-test.13`
  candidate·OIDC prerelease와 Windows actual preserving uninstall·global reinstall·`dry-run → apply → validate`·install
  validate PASS. 기존 public update·문제 보고 privacy/no-upload·stable parity regression과 유지보수자 새 Codex session
  탐색·Discord actual 전달 증거 결합
- [x] [REL9-015] 수용 기간의 blocker triage·privacy·disk retention 검증, bare 시험판 재게시 금지 확인.
  test.8 marketplace dangling 상태 복구·test.13 보존형 재설치 수용, problem report의 preview·collect·export와
  raw prompt·자동 upload·knowledge 자동 수집 `0건` regression, `test=0.9.0-test.13`·`latest=0.8.0` 확인
- [x] [REL9-016] 추가 시험판 필요 시에만 `0.9.0-test.N|test` 생성·검증·이전 시험판 계보 보존
- [x] [REL9-017] 시험 수용 뒤 `develop → main` PR #19 non-force merge와 exact main
  `c6f8a283260c2cd38f37a39b4fa3af38fbb2c61e` 고정. head `afd6380` CI 2건 PASS;
  병합 gate 면제 기록 유지
