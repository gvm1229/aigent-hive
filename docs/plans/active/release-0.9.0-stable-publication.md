# `0.9.0` stable publication 계획

> Parent: [`release-0.9.0.md`](release-0.9.0.md)
> Checklist owner: `REL9-019–035`
> Target: `0.9.0`

## Mandatory knowledge gate

- `KAC-001–008` 완료와 보정 commit을 포함한 replacement stable candidate 전
  `REL9-019–026` 실행 금지
- Existing stable candidate run `31482918509`: historical qualification only, publication authority 없음

## 정식 candidate·publication·public acceptance

- [x] [REL9-018] Protected `main` exact SHA `4b3d585f8e5d014a4b282cfeb6f9b2e9f8fb0f84`의 stable candidate
  [run `31482918509`](https://github.com/gvm1229/aigent-hive/actions/runs/31482918509) PASS. 5개 native archive·
  6개 npm package·direct installer·GitHub attestation·macOS ad-hoc·Windows unsigned evidence와
  public-only external TUF authorization request 생성, tag·GitHub Release·npm mutation `0건`
- [ ] [REL9-019] macOS·Windows signing과 external TUF production authorization·rollback floor 검증.
  서명 작업은 Windows 수용 시험 호스트가 아닌 분리된 Mac에서 수행. exact candidate artifact는
  run `31482918509`의 `release-authorization-request`; `signing-request.json`의 `main`
  `4b3d585f8e5d014a4b282cfeb6f9b2e9f8fb0f84`, product/package `0.9.0`, 10개 target의
  path·length·SHA-256 일치 확인이 선행 조건. Hive 검증기는 raw Ed25519만 허용하므로 PIV·실험적
  Sigstore 기본 경로만 제공하는 도구의 무검증 사용 금지. private key 생성·보관·서명은 Mac 외부 ceremony에
  한정, source tree·Windows 수용 호스트·GitHub environment·workflow artifact 입력 금지
- [ ] [REL9-020] Final main SHA의 annotated `v0.9.0`과 GitHub normal Release·signed artifact 게시
- [ ] [REL9-021] npm platform 5개 선행·umbrella 최종 OIDC publication과 `latest=0.9.0`, `test` 보존 확인
- [ ] [REL9-022] npm·Unix·PowerShell 5.1·CMD public clean install·repeat·pending receipt recovery PASS
- [ ] [REL9-023] `0.8.x → 0.9.0`과 `0.9.0-test[.N] → 0.9.0`의 knowledge·preference·foreign byte 보존과 SQLite rebuild PASS
- [ ] [REL9-024] GitHub·npm·direct binary byte identity·provenance와 public update discovery·consent PASS
- [ ] [REL9-025] Release·registry·installer 관찰, critical 회귀 0건과 rollback 판정
- [ ] [REL9-026] PLAN·CURRENT·release notes·ADR·bilingual fact에 run ID·SHA·digest·지원·미검증 범위 최종 반영
- [x] [REL9-027] `release-publish.yml` 단일 OIDC publication: `channel=test|stable`별 exact branch·candidate·tag·Release guard, `NPM_TOKEN` fallback 제거, six-package Trusted Publisher 설정 안내와 static contract 검증 — `0.9.0-test.6`의 six-package OIDC publication PASS
- [x] [REL9-028] Copier와 Rust harness renderer가 Discord `message_fields` 기본값을 byte-exact하게 동일 출력하는 parity PASS — `Copier 9.17.0` 격리 검증에서 4개 parity PASS
- [x] [REL9-029] `hive setup --describe`의 product-owned Codex expedited default와 disposable user-root
  connected acceptance 추가. `en`·Markdown English Wiki·general knowledge work·strict·all built-in Skill·update off·
  usage guard off profile의 clean install → `dry-run → apply → validate` → install validate, `hive uninstall` 뒤
  knowledge·preference 보존 재설치 → projection 자동 복원 → validate PASS. 수동 `where hive`·schema 답안 추측·
  사용자 홈 답안 파일·setup 질문 0건
- [x] [REL9-030] Python `test_*.py` 전수 단일 lane manifest·owner·contract·release gate 대장, Rust unit·static
  command와 CI job 대장, `os × lane` CI matrix, Windows runtime 기록 추가. documentation 1.01초·security 60.81초·
  contract 262.03초·integration 141.19초·release 12.73초, 순차 477.77초 대비 matrix Python critical path 262.03초
  모델 45.2% 단축. Phase 4 fixture: ignored `tests/work/hive-phase4-<random>`만 사용; tracked `tests/` 인접
  `hive-phase4-*` 생성 0건. 시험 삭제 0건·release gate 손실 0건
- [x] [REL9-031] 무료 배포 신뢰 정책 확정. Apple Developer Program·Microsoft Artifact Signing 유료 필수 gate
  제외, macOS ad-hoc·Windows unsigned 상태의 정확한 공개, SignPath Foundation 무료 승인 시에만 Windows
  Authenticode 추가, SHA-256·GitHub attestation·npm OIDC provenance·external TUF 유지
- [x] [REL9-032] platform signing evidence schema·verifier·candidate workflow가 macOS ad-hoc과 Windows
  signed 또는 unsigned 상태를 엄격한 조합으로 검증. Linux artifact는 provenance 대상이며 platform signing
  evidence 강제 대상 아님
- [x] [REL9-033] stable candidate의 5개 archive·checksum·provenance·platform evidence·migration table·release surface를
  private key 없이 deterministic external TUF authorization request로 생성. key 생성·서명·custody 경로 0건
- [x] [REL9-034] stable publication이 HTTPS TUF repository URL·SHA-256·agent-write-denied public root를 입력받아
  안전 추출·production verifier·candidate byte identity·rollback receipt 검증 뒤에만 tag·GitHub Release·npm 게시
- [x] [REL9-035] 공개 code-signing/privacy policy와 SignPath 무료 신청 조건, external TUF offline ceremony·publication
  입력 절차 문서화. SignPath 신청·private-key ceremony·protected environment 승인은 유지보수자 수동 경계.
  `575ab42` platform evidence, `5da1b49` authorization request, `1b6536e` publication gate와
  safe extraction·rollback receipt 검증 PASS

## 무료 배포 신뢰 정책

- 유료 Apple Developer ID·Microsoft Artifact Signing 구독은 `0.9.0` 필수 조건에서 제외
- macOS archive: release workflow의 명시적 ad-hoc signing과 검증, Apple publisher trust·notarization 미제공 공개
- Windows archive: SignPath Foundation 무료 승인 전 unsigned 상태 공개; 승인 뒤 동일 artifact 계보의 Authenticode 허용
- 모든 platform: archive SHA-256, GitHub artifact attestation, npm OIDC provenance, external TUF authorization 필수
- Hive source·workflow의 private key 생성·저장·서명 금지. 외부 signer는 공개 metadata와 receipt만 반환

## 실행 순서

1. `TST9-*`·`PRF-*` 기능 마감과 release handoff
2. `REL9-001` 원격 `develop` 기준선 재고정
3. `REL9-002–005` version grammar·분리 workflow 구현·독립 커밋
4. `REL9-006–012` clean clone·cross-platform 시험 후보와 Codex 실제 활성화 검증
5. `REL9-013–016` bare 시험판 독립 게시·수용·선택형 numbered 시험판
6. `REL9-029` product-owned 신속 기본값 무인 설치 수용
7. `REL9-030` 테스트 대장·lane·fixture 작업 영역 정리
8. `REL9-027–028` OIDC publication·Discord 설정 parity 유지 확인
9. `REL9-031–035` 무료 배포 신뢰·platform evidence·external TUF handoff 구현
10. `REL9-017–024` main 통합·stable candidate·Mac 외부 authorization·별도 정식 publication
11. `REL9-025–026` 관찰·current-truth 완료 기록

## 외부 권한 경계

- `main` PR review·merge와 protected `release-publication` approval
- Optional SignPath Foundation 승인과 external TUF threshold signer
- GitHub App write 권한·npm Trusted Publisher·test workflow 등록 권한
- Credential·private key·2FA material 노출 금지와 외부 mutation 직전 exact 대상 재확인

## 완료 기준

- 모든 in-scope `REL9-*` evidence-backed 완료
- 시험 `test`와 stable `latest`의 독립 mutation·exact commit 증거
- GitHub tag·Release·npm `latest`의 exact `0.9.0`·main SHA 일치
- 5개 platform artifact·6개 npm package·3개 direct installer 검증
- `0.8.0` 사용자 데이터·설정·project harness의 non-breaking upgrade
- contributor preference 입력 없는 기본 신속 설정의 clean install·재설치 자동 수용
- 테스트 분류 대장·coverage replacement·ignore disposable fixture 경계와 release lane 증거
- Signing·provenance·TUF·rollback·public acceptance의 미확인 항목 0건
