# Code signing policy

## 현재 배포 상태

| Platform | `0.9.0` 상태 | 사용자 확인 수단 |
| --- | --- | --- |
| macOS | Release workflow에서 ad-hoc signing. Apple Developer ID·notarization 없음 | archive SHA-256, GitHub artifact attestation, TUF target digest |
| Windows | SignPath Foundation 승인 전 unsigned. 승인 뒤 Authenticode 적용 가능 | archive SHA-256, GitHub artifact attestation, TUF target digest |
| Linux | Platform code signing 대상 아님 | archive SHA-256, GitHub artifact attestation, TUF target digest |

유료 Apple Developer Program·Microsoft Artifact Signing 구독은 `0.9.0` 필수 출시 조건에서 제외.
운영체제 publisher trust 부재를 checksum·attestation·TUF로 오인하는 표현 금지.

## 무료 Windows signing

SignPath Foundation의 무료 open-source code signing 승인 시 Windows Authenticode 적용.
승인 전 상태는 `unsigned`, 승인 뒤 상태는 `authenticode`와 certificate thumbprint로 release evidence에 기록.

승인 뒤 공개 표기:

> Free code signing provided by SignPath.io, certificate by SignPath Foundation.

승인 전 해당 표기·logo·signed claim 사용 금지. 신청·계정 MFA·certificate 발급은 유지보수자 소유 외부 절차.

## Privacy and network behavior

- Hive runtime의 telemetry·automatic crash upload·provider credential 수집 없음
- 설치·update 확인·Discord test·명시적 problem report export 등 사용자가 시작한 기능만 해당 endpoint 접근
- Release workflow의 GitHub·npm·optional SignPath 접근은 release automation 범위
- SignPath 사용 시 SignPath service 전달 대상: 서명 대상 Windows binary와 build provenance
- Knowledge Base·user preference·agent session·prompt의 code-signing service 전송 금지

## Verification boundary

- 모든 native archive: exact SHA-256와 GitHub artifact attestation
- npm package: GitHub Actions Trusted Publisher와 npm OIDC provenance
- Stable release: external TUF root·targets·snapshot·timestamp authorization과 rollback floor
- Hive source·GitHub workflow의 release private key 생성·저장·서명 금지
- Platform evidence의 허용 조합 외 상태는 stable publication 거부

## English policy

Aigent Hive 0.9.0 does not require paid platform certificates. macOS binaries are explicitly
ad-hoc signed and are not Apple Developer ID signed or notarized. Windows binaries remain
unsigned unless the project receives free SignPath Foundation approval. Linux archives do not
use platform code signing. Every native archive is still bound by an exact SHA-256 digest,
GitHub artifact attestation, npm OIDC provenance where applicable, and externally authorized TUF
metadata for the stable release.

Hive has no telemetry and does not automatically upload crash reports, prompts, Knowledge Base
content, preferences, credentials, or agent sessions. A future SignPath integration may transmit
only the Windows release binary and build provenance required for code signing. The repository
and its workflows never create, store, or use TUF private signing keys.
