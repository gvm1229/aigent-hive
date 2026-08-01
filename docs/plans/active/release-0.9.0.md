# `0.9.0` 정식 릴리스 계획

> Target: `0.9.0`
> Checklist owner: `REL9-*`
> Decision: [`ADR-0017`](../../decisions/ADR-0017-0.9-full-release.md)
> Authority: 2026-08-01 사용자 요청의 full release 계획·원격 `develop` push 승인

## 목표

- Protected `main` exact commit에 결합된 annotated `v0.9.0` tag
- GitHub normal Release와 5개 native artifact·SHA-256·attestation·서명 증거
- npm 여섯 package exact `0.9.0`, `latest=0.9.0`
- npm·Unix·PowerShell 5.1·CMD 설치와 `0.8.0 → 0.9.0` update 검증
- Release·npm·direct installer의 native binary byte identity
- Canonical source·release bundle·installed harness의 version·digest 계보

## 현재 기준선

- Source version·release date: `0.9.0`, `2026-08-01`
- Local qualification: Rust 459개 통과, Python 628개 통과·42개 platform skip
- RAG 50,000 chunk와 `.hivekb` 100 collection·50,000 chunk 성능 gate 통과
- `0.8.0` frozen project·user base와 synthetic release fixture 무변경
- Release notes: [`docs/releases/0.9.0.md`](../../releases/0.9.0.md)
- 기존 `release.yml`: remote `develop` manual candidate만 허용
- 기존 `release-publish.yml`: exact `0.8.0` hardcode와 `develop` candidate 결합
- GitHub normal Release·tag 생성 workflow 부재
- Repository release tag 부재
- Apple·Windows·external TUF production signer 증거 미확인
- GitHub ruleset: `Develop safety`는 deletion·non-fast-forward만 적용,
  `Protect main`은 PR·required checks 4개 적용
- Remote `develop` 기준선: `e4f1d6001a0a6ad5f41dccc350a0e585bbe9c9d0`
- `staging` branch: 현재 final-main candidate 계보에 불필요하여 생성 0건

## Artifact 계보

```text
remote develop qualification
  → develop → main PR·required CI
  → protected main exact final candidate
  → 5 native archives + 6 npm tarballs
  → SHA-256 + GitHub attestation + platform signing
  → external TUF production metadata
  → annotated v0.9.0 tag on the same main commit
  → GitHub Release + npm 0.9.0|latest
  → public clean install·0.8.0 update verification
```

- `develop` artifact: pre-integration qualification 전용
- `main` artifact: publication 대상 최종 계보
- Final candidate·tag·GitHub Release·npm publication의 exact commit 일치
- Merge 뒤 재빌드 없는 develop artifact 재사용 금지
- Publish workflow의 untrusted input·run·branch·SHA·attestation 재검증

## 구현·검증 checklist

### A. 기준선과 workflow activation

- [x] [REL9-001] 현재 implementation·계획·변경점 커밋의 원격 `develop` 반영과 exact remote SHA `e4f1d6001a0a6ad5f41dccc350a0e585bbe9c9d0` 확인
- [ ] [REL9-002] `0.9.0` version·release date·README·CURRENT·Cargo lock·harness template·fixture parity와 release notes gate
- [ ] [REL9-003] `release-publish.yml`의 `0.8.0` hardcode 제거, exact requested version·final candidate run·branch·SHA 결합
- [ ] [REL9-004] `release.yml`의 remote `develop` 사전 후보와 protected `main` 최종 후보를 explicit input으로 분리하고 ref downgrade 차단
- [ ] [REL9-005] GitHub normal Release workflow 추가: annotated tag·release body·asset·checksum·attestation·idempotency·existing-version refusal

### B. Clean-clone qualification

- [ ] [REL9-006] Fresh clone의 Rust format·strict Clippy·workspace all-target·all-feature test와 Python 전체 적합성
- [ ] [REL9-007] Ubuntu·macOS·Windows CI와 Linux musl x86_64·arm64 release runtime PASS
- [ ] [REL9-008] Installer·update·rollback·recovery·secret·symlink·path confinement hostile suite PASS
- [ ] [REL9-009] Host-native loop·Wiki·Skill·RAG·bundle·scan 전체 conformance와 OMX/OMC·tmux 자동 의존성 0건
- [ ] [REL9-010] RAG 50,000 chunk와 `.hivekb` 100 collection·50,000 chunk release profile 재측정·threshold PASS
- [ ] [REL9-011] Codex·Antigravity 실제 install·setup·project·update 회귀와 Claude fixture·미검증 범위 공개

### C. Candidate와 signing

- [ ] [REL9-012] Remote `develop` exact SHA에서 pre-integration candidate 5 target·6 npm·installer·byte identity PASS
- [ ] [REL9-013] `develop → main` PR의 required CI·review·non-force merge와 exact main SHA 고정
- [ ] [REL9-014] Protected `main` exact SHA에서 final candidate 재빌드·attestation·artifact inventory PASS
- [ ] [REL9-015] macOS Developer ID signing·notarization과 stapled verification evidence
- [ ] [REL9-016] Windows Authenticode 또는 Azure Artifact Signing과 clean-machine verification evidence
- [ ] [REL9-017] External TUF root·targets·snapshot·timestamp production authorization, threshold signature와 rollback floor 검증

### D. Publication과 public acceptance

- [ ] [REL9-018] Final main SHA에 annotated `v0.9.0` tag 생성·push와 tag immutability 확인
- [ ] [REL9-019] GitHub `0.9.0` normal Release 생성, 변경점·migration·알려진 제약과 5개 signed artifact 게시
- [ ] [REL9-020] npm platform package 5개 선행, umbrella 최종 OIDC publication과 `latest=0.9.0` 확인
- [ ] [REL9-021] npm·Unix·PowerShell 5.1·CMD public clean install·repeat·pending receipt recovery PASS
- [ ] [REL9-022] `0.8.0 → 0.9.0` npm·direct update에서 knowledge·preference·foreign byte 보존과 SQLite rebuild PASS
- [ ] [REL9-023] Public update discovery·24시간 throttle·offline retry·interactive consent와 exact owner activation PASS
- [ ] [REL9-024] GitHub asset·npm package·direct installer native binary SHA-256 byte identity와 provenance 공개
- [ ] [REL9-025] Release·registry·installer 관찰, critical 회귀 0건과 rollback·yank 조건 판정
- [ ] [REL9-026] PLAN·CURRENT·release notes·ADR·bilingual fact에 run ID·SHA·digest·지원·미검증 범위 최종 반영

## 실행 순서

1. `REL9-001` 원격 `develop` 기준선 고정
2. `REL9-002–005` release workflow activation 구현·독립 커밋
3. `REL9-006–012` clean clone·cross-platform 사전 후보 검증
4. `REL9-013–017` main 통합·최종 후보·production signing
5. `REL9-018–024` tag·GitHub·npm publication과 public acceptance
6. `REL9-025–026` 관찰·current-truth 완료 기록

## 외부 권한 경계

- `main` PR review·merge와 protected environment approval
- Apple·Windows signing identity와 external TUF threshold signer
- GitHub tag·Release 생성 권한과 npm Trusted Publisher environment
- Credential·private key·2FA material의 저장소·agent 노출 금지
- 각 외부 mutation 직전 exact SHA·artifact digest·대상 preview 재확인

## 완료 기준

- `REL9-001–026` 전부 evidence-backed 완료
- GitHub tag·Release·npm `latest`의 exact `0.9.0`·main SHA 일치
- 5개 platform artifact·6개 npm package·3개 direct installer 검증
- `0.8.0` 사용자 데이터·설정·project harness의 non-breaking upgrade
- Signing·provenance·TUF·rollback·public acceptance의 미확인 항목 0건
