# Release와 update 운영

## Local release 검증

전제: 공식 npm package 또는 GitHub Release에서 받은 bundle, consumer project 밖의 위치

```bash
hive release verify \
  --bundle /absolute/releases/aigent-hive-0.9.2 \
  --output json
```

- 성공 code: `hive.release-verified`
- Consumer project mutation: 없음
- Npm 선행 확인: registry integrity·provenance
- GitHub 선행 확인: exact tag·SHA-256·artifact attestation
- Hive 확인: local manifest의 length·SHA-256

## Consumer update

### Dry-run

```bash
hive update \
  --target /absolute/consumer-project \
  --bundle /absolute/releases/aigent-hive-0.9.2 \
  --dry-run \
  --output json
```

검토 대상: source·target version, plan digest, planned path

Dry-run mutation: backup·journal·update state·index 생성 없음

이전 Skill ownership: compile된 historical registry와 실제 bytes로 확인. Consumer-local
ledger 단독 신뢰 금지

### Apply

```bash
hive update \
  --target /absolute/consumer-project \
  --bundle /absolute/releases/aigent-hive-0.9.2 \
  --apply \
  --output json
```

- Release·baseline 재검증
- Dry-run 뒤 consumer bytes drift: conflict
- 성공 결과: backup transaction ID·rebuilt index digest

### Recovery

```bash
hive update \
  --target /absolute/consumer-project \
  --recover \
  --output json
```

- 다른 release option 동시 전달 금지
- Live bytes가 journal before·after 모두 아님: concurrent user edit 보존과
  `hive.update-conflict`

## Major migration

1단계: exact target dry-run

```bash
hive update \
  --target /absolute/consumer-project \
  --bundle /absolute/releases/aigent-hive-X.Y.Z \
  --dry-run \
  --exact-major-target X.Y.Z \
  --output json
```

2단계: `major-release-confirmation.schema.json` 작성

- Source·target version
- Plan digest
- Compatibility report digest
- Migration table digest
- Human review 뒤 `confirmed: true`

3단계: apply

```bash
hive update \
  --target /absolute/consumer-project \
  --bundle /absolute/releases/aigent-hive-X.Y.Z \
  --apply \
  --exact-major-target X.Y.Z \
  --major-confirmation .hive/config/major-release-confirmation.json \
  --output json
```

Apply: 모든 digest 재계산. Drift 시 중단. Release-provided executable migration 실행 없음

## Stable candidate와 publication

1. `develop` qualification과 reviewed PR
2. Protected `main` exact commit 고정
3. `Release candidate` workflow에서 native archive 5개·npm package 6개 한 번 build
4. SHA-256 sidecar·GitHub artifact attestation·native/npm binary byte identity 확인
5. `release-integrity-bundle` manifest·migration·surface inventory 검증
6. `release-publication` environment 승인 한 번
7. 같은 bytes의 GitHub Release·npm `latest` 게시
8. Public checksum·attestation·npm registry provenance 재확인

Publication workflow 입력 금지: 장기 npm token, signing key, certificate, provider credential

Npm 인증: Trusted Publishing OIDC

Release notes 필수 공개: macOS ad-hoc seal·Windows unsigned

Paid code signing: publication 선행 조건 아님

## Direct install

허용 installer: official versioned GitHub Release의 `install.sh`, `install.ps1`, `install.cmd`

금지: repository checkout의 installer template 직접 실행

### macOS·Linux

```bash
version=0.9.2
installer=$(mktemp "${TMPDIR:-/tmp}/aigent-hive-install.XXXXXX")
trap 'rm -f "$installer"' EXIT HUP INT TERM
curl --fail --location --proto '=https' --tlsv1.2 \
  --output "$installer" \
  "https://github.com/gvm1229/aigent-hive/releases/download/v${version}/install.sh"
AIGENT_HIVE_VERSION="$version" sh "$installer"
```

### Windows PowerShell 5.1

```powershell
$Version = "0.9.2"
$Installer = Join-Path ([IO.Path]::GetTempPath()) (
    "aigent-hive-install-{0}.ps1" -f [Guid]::NewGuid().ToString("N")
)
try {
    Invoke-WebRequest -UseBasicParsing `
        -Uri "https://github.com/gvm1229/aigent-hive/releases/download/v$Version/install.ps1" `
        -OutFile $Installer
    & $Installer -Version $Version -Prefix "$env:LOCALAPPDATA\AigentHive"
} finally {
    Remove-Item -LiteralPath $Installer -Force -ErrorAction SilentlyContinue
}
```

### Windows `cmd.exe`

```bat
set "HIVE_VERSION=0.9.2" && set "HIVE_PREFIX=%LOCALAPPDATA%\AigentHive" && powershell.exe -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $repository='https://github.com/gvm1229/aigent-hive'; $installer=Join-Path ([IO.Path]::GetTempPath()) ('aigent-hive-install-{0}.ps1' -f [Guid]::NewGuid().ToString('N')); try { Invoke-WebRequest -UseBasicParsing -Uri ($repository + '/releases/download/v' + $env:HIVE_VERSION + '/install.ps1') -OutFile $installer; & $installer -Version $env:HIVE_VERSION -Prefix $env:HIVE_PREFIX } finally { Remove-Item -LiteralPath $installer -Force -ErrorAction SilentlyContinue }"
```

Installer 검증:

- Archive entry allowlist
- SHA-256
- `hive --version`
- Existing binary·receipt·ancestor의 symlink·reparse 거부
- Receipt owner·digest·version 불일치 시 overwrite 금지

Platform 경계:

- macOS ad-hoc build: Apple publisher trust·notarization 제공 없음
- Windows unsigned build: Authenticode publisher identity 제공 없음
