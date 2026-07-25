# Signed update와 release 운영

## Consumer update

### 1. Release만 검증

Release directory와 public root의 위치: consumer project 밖. Unix/macOS root는
administrator가 agent가 쓸 수 없는 path에 설치.

```bash
hive release verify \
  --bundle /absolute/releases/aigent-hive-0.7.0 \
  --trust-root /usr/local/share/aigent-hive/release-root.json \
  --output json
```

성공 code: `hive.release-verified`. 이 command의 consumer project 수정 없음.

### 2. Dry-run

```bash
hive update \
  --target /absolute/consumer-project \
  --bundle /absolute/releases/aigent-hive-0.7.0 \
  --trust-root /usr/local/share/aigent-hive/release-root.json \
  --dry-run \
  --output json
```

Source/target version, plan digest와 planned path를 검토. Dry-run의 backup, journal,
update state, index 생성 없음.

이전 installation의 host Skill ownership은 consumer-local ledger만으로 판정 불가.
Running binary가 compile한 `harness/skills/historical-builtins.yml` release
history와 실제 Skill bytes의 SHA-256 일치 필수. 정상적인 0.4.0–0.6.0
projection은 migration할 수 있지만 Skill body와 `active-skills.yml`을 함께 바꾼
installation은 conflict로 중단하고 해당 bytes를 그대로 보존.

### 3. Apply

```bash
hive update \
  --target /absolute/consumer-project \
  --bundle /absolute/releases/aigent-hive-0.7.0 \
  --trust-root /usr/local/share/aigent-hive/release-root.json \
  --apply \
  --output json
```

Apply는 release를 다시 검증하고 baseline이 dry-run 이후 바뀌었으면 conflict로
중단. 성공 결과는 backup transaction ID와 rebuilt index digest를 포함.

### 4. Interrupted transaction

```bash
hive update \
  --target /absolute/consumer-project \
  --recover \
  --output json
```

Recovery 이외의 release option 동시 전달 금지. Concurrent user edit가
journal의 before/after 어느 쪽도 아니면 Hive는 그 bytes를 보존하고
`hive.update-conflict`를 반환.

## Major migration

Major version 자동 계산 없음. 사용자가 exact target을 명시적으로 결정한 뒤
별도 confirmation JSON에 다음 항목 결합 필수.

- source version과 exact target
- release plan digest
- compatibility report digest
- migration table digest
- `confirmed: true`

Confirmation은 consumer target 안의 bounded target-relative read.

먼저 confirmation 없이 exact target dry-run을 실행해 현재 plan과 compatibility
report 획득.

```bash
hive update \
  --target /absolute/consumer-project \
  --bundle /absolute/releases/aigent-hive-X.Y.Z \
  --trust-root /absolute/protected/release-root.json \
  --dry-run \
  --exact-major-target X.Y.Z \
  --output json
```

JSON의 `source_version`, `target_version`, `plan_digest`,
`compatibility_report_digest`, `migration_table_digest`를 exact하게 복사해
`major-release-confirmation.schema.json` 문서를 만들고 사람이 preservation report와
breaking scope를 검토한 뒤 `confirmed: true`로 고정. 그 다음 apply.

```bash
hive update \
  --target /absolute/consumer-project \
  --bundle /absolute/releases/aigent-hive-X.Y.Z \
  --trust-root /absolute/protected/release-root.json \
  --apply \
  --exact-major-target X.Y.Z \
  --major-confirmation .hive/config/major-release-confirmation.json \
  --output json
```

Apply는 release를 다시 검증하고 새 dry-run을 다시 계산. Confirmation의 source,
target, plan, observed compatibility/preservation 또는 signed migration digest가
하나라도 달라졌으면 중단. Release가 제공한 script나 binary migration은
실행 없음.

## Trust-root rotation

1. Offline root signer가 현재 root의 exact next version candidate 생성.
2. Candidate metadata는 이전 root threshold와 새 root threshold 서명을 모두 포함.
3. `verify_root_rotation`을 사용하는 release 운영 검증으로 두 threshold와 expiry를
   확인.
4. Administrator가 protected public-root path를 교체.
5. Consumer update는 새 root의 metadata version을 rollback floor에 기록.

Hive CLI의 private key, seed, PEM/PKCS#8, hardware-token secret 입력 금지.
Root 교체와 signing ceremony는 Hive 밖에서 수행.

## Release candidate 생성

`Signed release candidate` workflow는 exact version이 이미 source/Cargo/template에
반영된 reviewed main commit에서만 실행.

필수 protected authority:

- Apple Developer ID certificate, identity와 notarization credential
- Azure OIDC app와 Artifact Signing account/profile
- GitHub Actions artifact-attestation permission

Workflow는 macOS arm64/x86_64 tarball, Windows x86_64 zip, SHA-256 sidecar, offline
GitHub Sigstore bundle과 platform-evidence fragment를 생성. Tag와 GitHub Release
생성은 제외.

## External TUF authorization

External signer는 candidate workflow artifact를 받아 다음 항목 생성.

- offline root 2-of-3와 분리된 targets/snapshot/timestamp signatures
- exact archive path/length/SHA-256 target
- bundle manifest, migration table와 release surface inventory
- GitHub provenance와 Apple/Windows public signing evidence

Merged `platform-signing-evidence.json`의 entry는 candidate fragment와 byte-equivalent
canonical JSON 필수. `artifact_path`는 `targets/<archive>`이고 digest는 exact
candidate archive SHA-256. Production status는 전부 `verified` 필수.

Private signing material과 signer runtime은 source tree, workflow artifact, consumer
project와 Hive process에 포함 금지.

## Publication

`Publish authorized release` workflow에 exact version, successful candidate run ID,
externally signed TUF repository HTTPS URL과 exact lowercase SHA-256을 입력.
별도 `release-publication` environment approval이 필요.

Publication job은:

1. candidate run이 main의 successful `Signed release candidate`인지 확인.
2. exact candidate artifact 재수신.
3. TUF repository archive SHA-256을 확인.
4. protected environment secret의 public root를 root-owned read-only path에 설치.
5. `hive release verify`의 production evidence gate를 실행.
6. Verified signed bundle manifest의 `source.commit`을 selected candidate run SHA와
   exact comparison.
7. 각 offline Sigstore bundle을 `gh attestation verify`로 확인.
8. TUF repository의 platform archive와 candidate artifact를 byte-compare하고 merged
   platform evidence의 canonical JSON exact comparison.
9. 기존 tag/release가 없을 때만 candidate commit을 tag하고 asset을 공개.

Apple, Azure, external TUF signer 또는 GitHub publication credential 부재 시
production action 실행 불가. Local fixture PASS의 production signing 성공 표시는
금지.

## Install path

Repository의 `scripts/install.sh`와 `scripts/install.ps1`: signer marker 미치환
publication source template. Checkout에서 직접 실행 금지. 설치 입력은 versioned official
GitHub Release의 rendered installer asset으로 제한.

### macOS direct

```bash
version=0.7.0
repository=https://github.com/gvm1229/aigent-hive
installer=$(mktemp "${TMPDIR:-/tmp}/aigent-hive-install.XXXXXX")
trap 'rm -f "$installer"' EXIT HUP INT TERM
curl --fail --location --proto '=https' --tlsv1.2 \
  --output "$installer" \
  "$repository/releases/download/v${version}/install.sh"
AIGENT_HIVE_VERSION="$version" \
AIGENT_HIVE_PREFIX="$HOME/.local" \
sh "$installer"
```

Bootstrap은 fixed official GitHub Release URL에서 archive와 checksum을 받고 exact
archive entry allowlist, SHA-256, Developer ID signature, Gatekeeper assessment와
`hive --version`을 확인한 뒤 installed binary SHA-256과 version을 결합한 closed direct
install receipt를 기록. 기존 binary와 receipt의 symlink를 거부하고 receipt의
exact property set, binary digest와 reported version을 재검증. Stale receipt,
package-manager replacement 또는 foreign binary는 중단.

### Windows direct

```powershell
$Version = "0.7.0"
$Repository = "https://github.com/gvm1229/aigent-hive"
$Installer = Join-Path ([IO.Path]::GetTempPath()) (
    "aigent-hive-install-{0}.ps1" -f [Guid]::NewGuid().ToString("N")
)
try {
    Invoke-WebRequest -UseBasicParsing `
        -Uri "$Repository/releases/download/v$Version/install.ps1" `
        -OutFile $Installer
    & $Installer -Version $Version -Prefix "$env:LOCALAPPDATA\AigentHive"
} finally {
    Remove-Item -LiteralPath $Installer -Force -ErrorAction SilentlyContinue
}
```

PowerShell bootstrap은 zip entry allowlist/traversal, SHA-256, Authenticode `Valid`
status와 binary version을 확인. 기존 binary에 valid direct receipt가 없으면
중단. Receipt property set, current `hive.exe` SHA-256과 reported version은 모두
exact 일치 필수. Reparse point는 허용 대상에서 제외.

### Homebrew와 WinGet

Homebrew formula와 WinGet manifest는 `packaging/` source template에서 release
digest를 주입해 별도 package repository에 제출. 이 경로의 binary는 package
manager가 소유. Hive의 `brew upgrade`, `winget upgrade` 자동 실행과 managed
executable self-update 금지.

## Stable failure classes

| Code | Exit | 의미 |
| --- | ---: | --- |
| `hive.update-invalid-input` | 2 | option/schema/exact-version input 오류 |
| `hive.update-compatibility-blocked` | 3 | version/major/rollback/preservation policy 거부 |
| `hive.update-conflict` | 3 | live bytes와 plan/journal 불일치 |
| `hive.update-migration-unsupported` | 4 | compiled route 없음 |
| `hive.update-release-verification-failed` | 5 | signature/hash/expiry/provenance binding 실패 |
| `hive.update-rollback-failed` | 10 | 안전한 recovery 미완료 |

모든 실패는 `changed_paths: []`를 반환. 실패 뒤 user/foreign bytes를 수동으로
덮어쓰지 말고 journal과 backup evidence를 보존.
