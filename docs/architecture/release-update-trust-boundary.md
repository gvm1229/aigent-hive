# Release와 update 신뢰 경계

## 목적

- Release: 검토된 source commit과 공개 artifact 출처 증거의 결합
- Update: 이미 받은 local bundle의 무결성 확인과 consumer project 안전 적용

```text
protected main commit
  └─ candidate build ──> SHA-256 + GitHub artifact attestation
                          └─ protected publication ──> GitHub Release + npm provenance
                                                       └─ local bundle verification
                                                            └─ transactional update
```

경계 분리:

- Artifact 획득: npm registry 또는 GitHub Release
- Local byte 검증·activation: Hive update core
- Update core 비소유: network downloader, package-manager 실행, release signing key,
  provider API client

## 출처와 local 무결성

| 단계 | 증거 |
| --- | --- |
| npm 설치 | Registry integrity·Trusted Publishing OIDC provenance |
| GitHub 설치 | Exact tag·commit, SHA-256 sidecar, GitHub artifact attestation |
| Hive verifier | Local manifest, file length·SHA-256 |

보장 범위 분리:

- Local manifest: network 출처 증명 불가
- Registry·GitHub 증거: consumer update 중 local file 변경 허용 불가

## Bundle 계약

```text
bundle-manifest.json
targets/
├── migration-table.json
├── release-surface-inventory.json
├── aigent-hive-<version>-<target>.<archive>
├── aigent-hive-<version>-<target>.<archive>.sha256
└── <npm-package>-<version>.tgz
```

Manifest binding:

- Product·exact release version·monotonic release sequence
- Source repository·commit·tag
- Minimum updater·harness version
- Artifact별 normalized relative path·exact length·lowercase SHA-256

Verifier 거부 조건:

- Duplicate·정렬 위반·path traversal
- Symlink·reparse point
- Bounded no-follow read 중 file identity 변화
- 더 낮은 version·sequence
- 같은 sequence의 다른 manifest digest

Installed state: 마지막 수락 release version·sequence·manifest digest. 용도: transaction
recovery와 downgrade 방지. 별도 release 서명 권한과 무관

## Version과 migration

Version 형식: exact `X.Y.Z`

| Surface change | 허용 transition |
| --- | --- |
| Shipped feature 추가 | Exact next minor, patch `0` |
| Compatible fix | 같은 minor의 exact next patch |
| Shipped surface 변화 없음 | Version 유지 |
| Removal·incompatible semantic change | Same-major 거부 |

Surface 판정: `harness/release/historical-surfaces.yml`의 compiled baseline과 target
inventory의 category-prefixed set 비교

Fail-closed 조건:

- Historical registry 부재
- 정렬·unique 위반
- Same-major breaking removal

Cross-major apply authority:

- 사용자가 지정한 exact target
- Source·target, release plan, compatibility·preservation report
- Migration table digest
- 위 항목을 결합한 별도 human confirmation

실행 금지: release 제공 script, binary, DLL, WASM, shell migration. 허용 범위:
running Rust binary의 compiled `migration_id` allowlist

## Historical Skill ownership

소유권 증거:

- Running binary의 `harness/skills/historical-builtins.yml`
- Release별 name·SHA-256·side-effect class
- 실제 projection bytes
- Optional Skill의 기존 consent·source·content proof

거부·보존: unknown version, forged digest, 임의 host Skill path 거부와 foreign bytes 보존

## Backup, journal과 activation

Dry-run: release·installed baseline 검증과 exact plan 반환. Target mutation 없음

Apply 순서:

1. Incomplete journal 복구
2. Release·version·migration route·renderer dry-run 검증
3. Changed manifest-owned path와 canonical config·team·run·knowledge backup
4. 첫 live mutation 전 durable `prepared` journal 기록
5. Ownership-protected atomic activation
6. Exact after digest 확인과 `committed` journal 기록
7. Accepted release state를 마지막 commit marker로 기록
8. Canonical Markdown·YAML·TOML 기반 disposable SQLite index rebuild
9. Journal 제거와 검증된 7일 초과 unreferenced backup 정리

Migration input·canonical backup authority 제외: SQLite, runtime, backup, `.omx/`, `.omc/`

Recovery 규칙:

- Live bytes = journal before 또는 after digest: recovery 허용
- 어느 쪽도 아님: concurrent user edit 판정, bytes·journal 보존

Cross-major preservation: project file, docs, preference, user Markdown, symlink identity와
shared `AGENTS.md` marker 밖 foreign bytes의 activation 전후 비교. Mutable 범위:
compiled Hive-owned system representation

## Candidate와 publication

Candidate build:

- Protected `main` exact commit
- Native archive 5개·npm package 6개·installer 3개 한 번 build
- 모든 artifact의 SHA-256·GitHub attestation
- Native/npm binary byte identity
- macOS ad-hoc seal·Windows unsigned 상태 검증

Publication:

- Protected `release-publication` environment 승인 한 번
- Rebuild 없는 GitHub normal Release·npm `latest`
- Npm Trusted Publishing OIDC·registry provenance
- 장기 `NPM_TOKEN` 없음
- Developer ID·notarization, Authenticode·SignPath: 선택 기능, stable gate 아님

Direct installer:

- Official versioned GitHub Release archive·checksum 사용
- Entry allowlist·SHA-256·binary version 검증
- Owner·binary digest·version receipt 기록
- npm·Homebrew·WinGet 소유 binary overwrite 없음

## 보장 범위

- SHA-256·local manifest: exact bytes
- GitHub attestation·npm provenance: build·publication identity
- Transaction journal: 실패 rollback·crash recovery
- macOS ad-hoc seal: publisher identity·notarization 제공 없음
- Windows unsigned release: publisher identity 제공 없음
- Package acquisition network·account 보안: GitHub·npm·package manager 경계
