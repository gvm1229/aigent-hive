# ADR-0008: Verifier-only Ed25519/TUF release authorization

- 상태: accepted
- 날짜: 2026-07-24

## 맥락

Phase 6 release/update는 tampering, metadata mix-and-match, expiry, rollback,
same-version substitution, key rotation, threshold compromise의 통합 처리가 필요.
Judge attestation format은 owner/judge/human identity에 특화되어 release target graph와
rollback semantics 부재.

Hive가 signing key를 생성하거나 저장하면 local verifier, source repository와 release
authority가 하나의 trust domain이 되어 agent가 자신의 output을 self-authorize할 수
있음.

## 결정

Choice 1을 채택.

- TUF `1.0.31` compatible root/targets/snapshot/timestamp metadata
- strict Ed25519 verification only
- offline root 2-of-3와 분리된 release/snapshot/timestamp role
- target path, length와 SHA-256을 signed metadata chain으로 결합
- expiry, metadata version, release sequence와 same-sequence manifest digest rollback floor
- root rotation은 이전 threshold와 새 self-threshold를 모두 요구
- root public-key material은 전역 unique, role 간 재사용 금지, root는 exact 2-of-3
- public root는 consumer/release target 밖의 agent-write-denied path에서만 read-only 조회
- release private key, signing API, key generation과 credential은 Hive source/runtime에
  포함 금지
- signed release class를 그대로 신뢰하지 않고 compiled historical surface와 signed
  cumulative inventory를 비교해 observed delta를 계산
- cross-major apply authority를 exact source/target, renderer plan, observed
  compatibility/preservation report와 signed migration-table digest에 결합

Release-facing provenance, Apple Developer ID/notarization과 Windows Authenticode는
별도 evidence layer. TUF가 그 evidence bytes를 authorize하지만, Hive Phase 6
runtime의 Apple/Microsoft/GitHub identity PKI 전체 대리 검증 주장 금지.
Hive는 in-toto/SLSA source·builder·subject와 platform/scheme/path/digest/status를
semantic 검증. Public fixture status는 integrity test에서만 허용하고 public
publication은 exact archive target 전체에 대해 `verified` evidence와 별도 GitHub
Sigstore bundle 검증을 요구.

## Release workflow

Candidate build/signing과 public publication을 분리.

1. Protected candidate workflow가 OS-signed artifact, offline GitHub Sigstore bundle과
   verified platform-evidence fragment 생성.
2. External signer가 exact candidate bytes로 TUF repository 생성.
3. 별도 publication approval이 protected root로 TUF를 검증하고 Sigstore, candidate
   SHA와 signed source commit, candidate bytes와 merged platform evidence를 exact
   compare한 뒤 tag/release를 생성.

Candidate workflow는 public release authority가 없고 publication workflow의 private
TUF key 접근도 없음.

## 거부한 대안

- Judge envelope 재사용: release role, expiry, snapshot consistency와 rollback semantics
  부재
- Custom single-signature manifest: threshold/rotation/mix-and-match protection 재구현
- Hive 내장 signing/key generation: verifier-only boundary와 self-authorization 방지
  위반
- Full Sigstore-only update root: offline local update와 독립 root/rollback lifecycle을
  GitHub identity availability에 결합
- Release script/wasm migration: signed artifact가 arbitrary code execution channel이 됨

## 결과

- Release repository와 network source를 신뢰하지 않고 protected public root에서
  authorization을 시작.
- Private signing material 없이 public-only fixture와 hostile verification을 재현할 수
  있음.
- Root rotation ceremony, platform code-signing credential와 production publication은
  external authority가 필요.
- Exact architecture는
  [`../architecture/release-update-trust-boundary.md`](../architecture/release-update-trust-boundary.md),
  운영 절차는
  [`../guides/signed-update-and-release.md`](../guides/signed-update-and-release.md)를
  참조.
