# Ed25519 judge attestation 운영 가이드

이 문서는 Aigent Hive의 authenticated judge quorum을 운영하는 public-key 관리자와
external signer 구현자를 위한 절차. Exact security contract는
[`../architecture/judge-trust-boundary.md`](../architecture/judge-trust-boundary.md),
채택 이유는
[`../decisions/ADR-0007-ed25519-judge-trust.md`](../decisions/ADR-0007-ed25519-judge-trust.md)
참조.

## 책임 분리

| 주체 | 소유하는 것 | 소유하지 않는 것 |
| --- | --- | --- |
| Hive CLI | schema, JCS digest, trust-root protection, Ed25519 signature와 quorum 검증 | private key, signer 호출, model/judge 실행 |
| Resolved host·OMX·OMC owner | 독립 judge 실행, artifact 전달 순서와 verdict 격리 | Hive trust-root 변경 권한 |
| Public-key 관리자 | trust-root 발급·보호, key 등록·rotation·revocation | judge verdict 내용 |
| External signer | private-key custody, user-presence와 signing policy | Hive consumer target mutation 권한 |
| Human approver | critical verdict 이후 별도 approval 결정 | owner 또는 judge key 재사용 |

Hive process의 private key 읽기 또는 user confirmation 없는 unlocked signer
호출은 independent approval 경계와 양립 불가. Private key, seed,
PEM, PKCS#8, recovery material과 signer credential을 consumer project, release bundle,
Hive config, environment dump, log 또는 test fixture에 배치 금지.

## 1. Key와 principal 준비

하나의 public key는 정확히 하나의 `principal_id`와 purpose에만 배정.

- Resolved owner: `judge-assignment`
- 각 independent judge: `judge-verdict`
- Critical human approver: `judge-approval`

Owner, judge 3명과 human approver는 서로 다른 key를 사용. 같은 raw public-key
bytes를 다른 key ID나 principal로 다시 등록하면 trust root 전체 거부.
External signer는 raw 32-byte Ed25519 public key를 lowercase hex
`ed25519:<64 hex>` 형식 export 필수.

## 2. Trust root 작성

Trust root: `schemas/judge-trust-root.schema.json`을 따르는 TOML.

```toml
schema_version = 1
trust_root_id = "security-team"
revision = 7
issued_at = "2026-07-24T00:00:00Z"
root_digest = "sha256:<JCS digest excluding root_digest>"

[[keys]]
key_id = "owner-2026"
principal_id = "resolved-owner-42"
purpose = "judge-assignment"
algorithm = "ed25519"
public_key = "ed25519:<64 lowercase hex>"
status = "active"
valid_from = "2026-07-01T00:00:00Z"
valid_until = "2027-06-30T23:59:59Z"
```

`root_digest`는 TOML bytes의 digest와 별개. TOML을 normalized object로 parse한 뒤
`root_digest` field를 제외하고 RFC 8785 JCS로 canonicalize한 UTF-8 bytes의
SHA-256. Revision을 포함한 field가 하나라도 바뀌면 digest를 다시 계산.
Unknown field, duplicate key ID, duplicate public key, invalid raw key와 inverted
validity interval은 거부 대상.

Repository의
[`tests/fixtures/judge/trust-root.toml`](../../tests/fixtures/judge/trust-root.toml)은
public-only conformance 예심. 이 fixture에는 대응 private key가 없으며 production
authority 사용 금지.

## 3. Trust root 보호

`--trust-root` 요구 사항: absolute path, consumer target 밖. Request JSON이나 target
config의 path 선택 권한 없음.

macOS·Unix 설치기는 root-owned directory에 single-link regular file로 설치하고
file write bit 전체 제거 필수. 모든 ancestor도 root-owned이며 group/world
write가 없어야 유효. Hive는 macOS extended ACL을 포함한 current-process
write+execute access를 각 ancestor에서 확인하고 final file의 실제 write open도
검사. 하나라도 허용되면 검증 중단. Read 뒤 path protection과 file
identity를 다시 검증. 예시 위치는 `/etc/aigent-hive/judge-trust-root.toml` 또는
macOS의 root-owned
`/Library/Application Support/Aigent Hive/judge-trust-root.toml`.

Windows 설치기는 예를 들어
`%ProgramData%\Aigent Hive\judge-trust-root.toml`의 administrator ACL 적용 필수.
Hive는 모든 component의 reparse point를 거부하고 현재 process token으로
file과 ancestor의 write, append, child create/delete, `DELETE`, `WRITE_DAC`,
`WRITE_OWNER` 권한을 각각 probe. 모든 probe의 `ACCESS_DENIED` 결과 필수.

Protection 확인 불가 또는 read 중 file identity·size 변경 시 fail closed.
Hive를 root/Administrator처럼 trust root를 수정할 수 있는 token으로 실행하는 것도
보호 증거 아님; 거부 대상.

## 4. Artifact별 detached attestation 생성

Artifact 순서: package → assignment → verdict → approval. 다음 artifact를
만들기 전에 앞 artifact의 digest와 검증 결과를 고정.

1. `judge-package`: Hive가 package 자체의 JCS digest를 생성.
2. `judge-assignment`: resolved owner가 package, requester/task agent와 exact
   judge roster를 고정.
3. Assignment attestation: owner의 `judge-assignment` key로 서명.
4. `judge-verdict`: 각 judge가 자기 slot과 assignment digest에 결합된 final
   verdict를 생성.
5. Verdict attestation: 각 judge의 `judge-verdict` key로 별도 서명.
6. `judge-approval`: critical tier에서 모든 eligible verdict 이후 human approver가
   별도 artifact를 생성.
7. Approval attestation: approver의 `judge-approval` key로 서명.

Sidecar는 `schemas/judge-attestation.schema.json` 준수.

```json
{
  "schema_version": 1,
  "trust_root_id": "security-team",
  "artifact_kind": "verdict",
  "artifact_digest": "sha256:<JCS digest of the complete artifact>",
  "principal_id": "judge-instance-2",
  "key_id": "judge-2-2026",
  "signature": "ed25519:<128 lowercase hex>"
}
```

External signer가 서명할 exact message는 다음 bytes.

```text
"AIGENT-HIVE" || NUL || "JUDGE-ATTESTATION" || NUL || "V1" || NUL
|| UTF-8(JCS(attestation_without_signature))
```

Signer의 OpenSSH `SSHSIG`, PEM envelope, pre-hashed Ed25519 variant 출력 금지.
Raw Ed25519 signature 64 bytes를 lowercase hex로 encode.

## 5. Authenticated quorum 실행

Schema v2 request는 assignment attestation, verdict와 같은 순서의 verdict
attestation, critical approval과 approval attestation을 target-relative path로
참조.

```bash
hive judge quorum \
  --target /path/to/consumer-project \
  --request judge/quorum-request-v2.json \
  --trust-root /absolute/admin-protected/judge-trust-root.toml \
  --output json
```

성공적인 authenticated result에는 `authentication:"ed25519"`와
`authenticated:true` 표시. Elevated PASS는 authenticated verdict 2/3,
critical PASS는 distinct authenticated verdict 3/3과 별도 authenticated approval을
요구. Aggregate output은 identity, public key, key ID, signature, finding과 개별
verdict는 출력에서 제외.

## 6. Rotation과 revocation

Rotation의 기존 protected root 제자리 수정 금지.

1. 새 key를 external signer에서 준비.
2. Trust-root revision을 증가시키고 새 public key와 validity를 추가.
3. 교체할 key는 `status = "revoked"`로 바꿈. Revoked key는 과거 artifact
   timestamp라도 즉시 무효.
4. `root_digest`를 다시 계산하고 별도 staging에서 schema와 known-answer vector를
   검증.
5. Administrator가 protected path를 atomic replacement.
6. Hive process가 새 revision을 read-only로 검증하는지 확인.

과거 quorum을 장기 보존해야 한다면 당시 trust-root revision과 public metadata를
별도 audit archive에 보존. Hive consumer target과 SQLite index는 trust-root
history의 정본에서 제외.

## 7. 실패 해석

| 조건 | 결과 |
| --- | --- |
| Unsigned v1 request | `authenticated:false`, `INDETERMINATE`; completion 권한 없음 |
| Missing·malformed attestation | 해당 chain은 PASS 불가 |
| Wrong artifact digest, principal, purpose 또는 trust-root ID | authentication 실패 |
| Revoked·out-of-window key | authentication 실패 |
| Owner/judge/approver key 재사용 또는 duplicate public key | quorum 제외 또는 trust-root 거부 |
| Target-contained, caller-writable, symlink/reparse 또는 replaceable trust root | command `blocked` |
| Signature bit flip 또는 domain mismatch | authentication 실패 |
| Valid signature지만 부정확한 judge 판단 | Cryptography로 해결 불가. Deterministic test와 independent review 별도 필요 |

`INDETERMINATE`는 우회 가능한 warning이 아니라 completion authority가 없다는
fail-closed 결과. Operator는 identity 문자열이나 public digest를 수동으로
신뢰 상태 처리 금지. Trust root, signer policy, artifact 순서 수정 필요.
