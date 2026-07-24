# Release와 update trust boundary

## 목적

Hive update는 이미 인증된 local release를 consumer harness에 안전하게 적용.
Release 획득, release 승인, OS code signing, GitHub provenance와 consumer activation은
서로 다른 authority.

```text
source commit
  └─ candidate workflow ──> OS-signed platform artifacts + GitHub attestation
                              └─ external release signer ──> signed TUF repository
                                                           └─ Hive verifier
                                                                └─ staged consumer update
```

Hive는 마지막 두 단계에서 **검증과 local activation만** 수행. Model-provider API,
provider SDK, downloader, package-manager 실행, release signing과 private-key custody는
제품 runtime에 포함 없음.

## Choice 1: verifier-only Ed25519/TUF

Release authorization은 TUF `1.0.31` wire metadata의 제한된 verifier subset을 사용.
Crypto primitive는 `ed25519-dalek`의 strict verification이며
`default-features=false`. Production crate에는 signing API, `SigningKey`, seed,
PEM/PKCS#8 private material 또는 key generation이 없음.

| Role | 권장 key 경계 | 책임 |
| --- | --- | --- |
| `root` | offline Ed25519 2-of-3 | role/key/threshold와 rotation authority |
| `targets` | root와 분리된 protected release role | exact target path, length, SHA-256 |
| `snapshot` | 별도 online key | targets metadata version/hash 결합 |
| `timestamp` | 별도 short-lived online key | snapshot freshness와 expiry 결합 |

Hive는 distinct valid key ID만 threshold 계산에 포함. Public-key material은 root 전체에서
unique하며 한 key의 두 role 배정 금지. Root role은 exact 2-of-3이며
unassigned key도 거부. Unknown key, duplicate signature, wrong role, unsupported
algorithm, malformed raw key/signature, threshold 미달은 모두 거부. Metadata
expiry는 `now >= expires`에서 거부.

### Root rotation

`verify_root_rotation`은 candidate root가 현재 root의 exact next version인지 확인한 뒤:

1. 이전 root의 `root` threshold로 candidate의 `signed` bytes를 검증.
2. candidate root 자체의 새 `root` threshold로 같은 bytes를 다시 검증.
3. 새 role/key 집합, TUF version과 expiry를 검증.

따라서 새 key만으로 과거 trust를 탈취할 수 없고, 탈취된 과거 key만으로 새 root를
고정도 금지. Rotation file의 실제 교체는 Hive 자동 수행 범위 밖이며
agent-write-denied 운영 경계 적용.

## Protected public root

`hive release verify`와 `hive update`는 absolute path의 외부 public root만 read-only 조회.
Consumer target 안의 root, symlink/reparse point, writable file, replace 가능한
ancestor와 read 도중 identity/length가 바뀐 file은 거부.

- Unix/macOS: root-owned, non-writable, single-link regular file과 root-owned
  non-group/world-writable ancestor를 요구하고 current process의 file/ancestor mutation
  access가 실제로 거부되는지 확인.
- Windows: 모든 path component의 reparse point를 거부하고 current token으로 file과
  ancestor의 write/delete/ACL/owner mutation right open 차단 필수.

Public key는 secret이 아니지만 agent가 자신이 만든 release를 스스로 승인하지 못하도록
write authority를 분리.

## Signed repository contract

```text
metadata/
├── root.json
├── targets.json
├── snapshot.json
└── timestamp.json
targets/
├── bundle-manifest.json
├── migration-table.json
├── release-surface-inventory.json
├── provenance.intoto.json
├── platform-signing-evidence.json
└── aigent-hive-<version>-<target>.<archive>
```

Verifier는 metadata와 target을 bounded no-follow regular file로 두 번 확인.
`timestamp → snapshot → targets → target`의 version, length와 SHA-256 결합을 모두
통과한 뒤에만 payload JSON을 해석. Target path는 `targets/` 아래의 normalized
project-relative path만 허용.

`bundle-manifest.json`은 product, exact version, monotonic release sequence, release
classification, source repository/commit/tag, minimum updater/harness version, Apache-2.0
license와 다음 target digest를 결합.

- deterministic surface inventory
- compiled migration table
- in-toto provenance statement
- public platform-signing evidence

Hive는 provenance를 단순 blob이 아닌 검증 대상 구조로 취급. in-toto statement type, SLSA
predicate, exact source repository/commit, locked build, release workflow builder,
invocation time 순서와 artifact subject digest를 semantic 검증. Platform evidence는
macOS=`developer-id`, Windows=`authenticode`, unique artifact path, SHA-256과 status를
검증.

Update integrity mode는 public-only crypto fixture의
`fixture-public-evidence`를 허용하지만 `external-production-required`는 허용 불가.
Publication mode는 모든 status가 `verified`여야 하고 provenance subject와 platform
evidence의 signed repository 전체 archive target exact 열거가 필수. Protected
publication workflow는 각 archive의 offline GitHub Sigstore bundle을
`gh attestation verify`로 별도 검증. 따라서 TUF evidence binding과 외부
PKI/Sigstore identity verification은 분리되어 있으면서 exact artifact digest로
결합 상태.

## Version과 migration

Version은 exact `X.Y.Z`만 허용.

| Surface change | 허용 transition |
| --- | --- |
| shipped feature 추가 | exact next minor, patch `0` |
| 같은 public surface의 compatible fix | 같은 minor의 exact next patch |
| shipped surface 변화 없음 | version 유지 |
| removal/incompatible semantic change | same-major 거부 |

Release class는 signed manifest가 선언하지만 updater는 그 선언에서 `SurfaceDelta`를
생성 금지. `harness/release/historical-surfaces.yml`에 compile된 migration
baseline과 signed cumulative inventory의 category-prefixed set을 비교.

- baseline item이 target에서 사라지면 `breaking`
- target item이 추가되면 `additive-feature`
- set은 같고 version이 바뀌면 `compatible-fix`
- set과 version이 모두 같으면 `none`

따라서 공격자의 `classification=feature` 선언을 통한 minor bump 정당화와
breaking removal의 compatible 표시는 불가. Historical registry 부재 또는
정렬·unique contract 위반 시 update는 unsupported/internal failure로 fail-closed.

Major target 자동 계산 없음. Cross-major는 user-supplied exact target과
source/target, release plan, compatibility report와 migration table digest를 결합한
별도 human confirmation 모두 필수.

Cross-major dry-run은 confirmation 없이 exact target만 받아 plan과 report digest
생성 허용. Apply는 confirmation의 source/target, exact plan digest,
independently observed surface+preservation report digest와 signed migration-table
digest가 현재 재검증 결과와 하나라도 다르면 거부. Digest 모양만 맞는 임의
confirmation은 authority가 아닌 입력 증거.

Signed metadata의 executable migration 제공 금지. `migration_id`는 running
Rust binary에 compile된 allowlist 중 하나여야 하며 shell, DLL, dylib, WASM, script,
argv 또는 downloaded code 실행 금지.

- `same-major-render-v1`: supported same-major source를 current deterministic renderer로
  재구성.
- `cross-major-system-representation-v1`: future explicit-major route가 system-owned
  representation만 바꿀 수 있도록 preservation evidence를 검증.

Cross-major preservation gate는 project file, docs, preference, user Markdown body와
symlink identity를 recursive pre/post snapshot으로 비교. Shared `AGENTS.md`는
Hive marker block을 제외한 exact foreign bytes를 별도로 digest해 marker 갱신이 user
text를 숨긴 변경을 차단. Planned protected-path change는 mutation 전에
거부하고 activation 뒤 snapshot을 다시 계산해 실제 drift도 거부·recovery 상태로
전환. Mutable path는 compiled Hive system config/license representation과
authenticated `.agents|.claude/skills/<safe-name>/SKILL.md` projection으로 제한.
SQLite, runtime과 backup은 migration input에서 제외.

### Historical Skill ownership authentication

0.1.0–0.3.0: host Skill projection 없음. 0.4.0–0.6.0: release별 built-in 집합과
bytes 상이. Current renderer만으로 이전 installation을
재생성하면 정상적인 update가 시작되기 전에 ownership 검증이 실패. 반대로
consumer-local `active-skills.yml`의 digest를 그대로 신뢰하면 공격자의 ledger·projection
동시 위조와 foreign bytes의 Hive-owned path 승격 위험.

`harness/skills/historical-builtins.yml`은 지원하는 각 이전 release의 exact built-in
name, SHA-256, side-effect class와 capability set만 담는 typed YAML 정본. Binary는
이 registry를 compile하고 exact release coverage를 semantic 검증. Update 시:

1. installed `harness_version`으로 historical generation 선택.
2. ledger의 built-in metadata가 compiled historical entry와 exact하게 같은지 확인.
3. host projection의 실제 bytes를 읽어 compiled SHA-256과 대조.
4. approved optional Skill은 기존 consent/source/content proof로 별도 재검증.
5. canonical ledger bytes까지 일치한 path만 backup, replace 또는 recovery 대상으로
   인정.

Registry에는 과거 Skill 본문, private key 또는 executable migration이 없음. Unknown
version, 임의 patch version, forged digest, arbitrary `.agents`/`.claude` path는
fail closed. Host namespace 예외 적용 대상은
`.agents/skills/<safe-name>/SKILL.md`와
`.claude/skills/<safe-name>/SKILL.md` exact file로 제한.

## Backup, journal과 activation

Update는 release verification, rollback floor, classification, migration route와 renderer
dry-run이 끝나기 전 target에 쓰기 없음.

Apply 순서:

1. 기존 incomplete journal recovery.
2. installed/source/running/release version과 signed route를 검증.
3. renderer dry-run으로 exact before/after digest와 plan digest 생성.
4. changed manifest-owned path와 canonical config/team/run/knowledge snapshot을
   `.hive/backups/<transaction>/`에 fsync.
5. SQLite, WAL/SHM/journal, runtime, backup, `.omx/`, `.omc/`는 snapshot에서 제외.
6. 첫 live mutation 전에 ignored durable journal을 `prepared`로 기록.
7. renderer의 ownership-protected atomic activation을 실행하고 dry-run과 exact하게
   같은 plan/tree인지 재검증.
8. 모든 live after digest를 확인한 뒤 journal을 `committed`로 기록.
9. rollback floor/update state를 마지막 commit marker로 기록.
10. canonical Markdown/YAML/TOML에서 disposable SQLite index rebuild.
11. journal을 제거하고 valid·unreferenced·7일 초과 backup만 정리.

Prepared/needs-recovery journal은 live path가 기록된 before 또는 after digest일 때만
rollback. 제3의 digest는 concurrent user edit로 보고 bytes와 journal을 보존.
Committed journal은 after digest를 재확인한 뒤 update state와 index rebuild를
forward completion.

Backup cleanup은 exact `txn-<24 lowercase hex>` directory, self-digested valid manifest,
enumerated regular-file set와 모든 entry digest가 일치할 때만 file-by-file 삭제.
Malformed, future-dated, exact 7일 경계, active, symlinked 또는 foreign-entry backup은
보존.

## Release와 installer ownership

`.github/workflows/release.yml`은 protected `release-signing` environment에서:

- macOS arm64/x86_64의 Developer ID signing과 notarization.
- Windows x86_64의 Azure Artifact Signing OIDC 기반 Authenticode signing/timestamp.
- exact artifact SHA-256과 GitHub artifact attestation을 생성.
- offline Sigstore bundle과 `verified` platform-evidence fragment를 candidate artifact로
  보존.

이 candidate workflow는 tag나 GitHub Release를 만들 권한이 없음.

`.github/workflows/release-publish.yml`은 별도 `release-publication` approval 뒤:

- 성공한 main candidate run과 exact commit을 고정.
- external signer가 만든 TUF repository archive와 protected public root를 read-only 조회.
- `hive release verify` 뒤 signed bundle manifest의 `source.commit`이 선택한 candidate
  run SHA와 exact하게 같은지 확인.
- candidate/TUF target byte comparison을 통과.
- 각 candidate의 Sigstore bundle과 merged platform-evidence exact comparison을
  통과.
- 기존 tag/release가 없을 때만 exact commit을 tag하고 immutable asset을 공개.

Direct bootstrap은 official GitHub Release URL만 사용하고 archive entry allowlist,
archive SHA-256, OS signature·Gatekeeper/Authenticode와 binary version을 검증한 뒤
`owner=direct` receipt를 남김. Receipt는 closed exact field set, installed binary
SHA-256과 그 binary가 보고하는 exact version을 결합. 기존 executable 또는
receipt가 symlink/reparse이거나 receipt digest/version이 현재 binary와 다르면 stale
direct receipt로 보아 덮어쓰기 금지. Homebrew/WinGet은 binary owner이며 Hive updater는
그 executable 덮어쓰기와 package manager 실행 금지. `hive update`는
현재 running CLI가 지원하는 signed route로 consumer harness만 갱신.

## 보장하지 않는 것

- Ed25519는 authorized private-key possession과 exact signed bytes를 증명하지만
  signer의 판단이 옳다는 것은 증명 범위 밖.
- Apple/Microsoft/GitHub account와 signing credential은 Hive 소유 범위 밖.
- Production signing/notarization/publication success의 local test 기반 추론 금지.
- Package acquisition network는 installer/package manager 소유. Hive update core
  입력은 extracted local repository로 제한.
