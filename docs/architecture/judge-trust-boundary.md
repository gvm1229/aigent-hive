# Judge Ed25519 신뢰 경계

Hive의 judge 기능은 모델 실행기나 signing agent가 아니라 로컬 검증기다.
`hive-judge-package`와 Hive CLI는 judge, model, process, subagent를 생성하거나
호출하지 않는다. 실제 독립 judge 실행과 private-key signing은 이미 resolve된
host/OMX/OMC owner와 사용자가 승인한 외부 signer가 소유한다.

## 보안 목표

Authenticated quorum은 다음 공격을 막아야 한다.

- task agent가 owner, judge와 human identity 문자열을 임의로 만든 뒤 공개 digest를
  다시 계산해 self-approval하는 공격
- 같은 key를 여러 identity나 quorum slot으로 재사용하는 공격
- assignment, verdict 또는 approval을 수정한 뒤 기존 signature를 replay하는 공격
- owner key를 judge/human key로, judge key를 human key로 사용하는 role confusion
- target 안의 self-certified public key나 caller-writable trust root를 신뢰하는 공격

Ed25519는 trusted public key에 대응하는 private key 보유자가 exact bytes에 서명했다는
사실을 검증한다. Content encryption, judge 판단의 진실성, 실제 wall-clock 시각 또는
전역 one-shot replay 방지는 제공하지 않는다.

## Artifact chain

판정 체인은 다음 순서다.

1. `judge-package`가 subject, risk tier, acceptance criteria, artifact/evidence digest를
   RFC 8785 JCS digest로 고정한다.
2. `judge-assignment`가 verdict 이전에 exact package/criteria, requester, task agent,
   resolved owner, owner provenance와 distinct slot/instance/eligibility tuple을 고정한다.
3. `assignment` detached attestation을 `judge-assignment` purpose의 trusted owner key로
   검증한다. 이 signature가 task-agent identity와 전체 roster를 인증한다.
4. 각 `judge-verdict`는 exact assignment와 slot tuple에 결합되며 별도 detached
   attestation을 해당 principal의 `judge-verdict` key로 검증한다.
5. Critical 판정의 `judge-approval`은 모든 eligible verdict 뒤 생성하고 별도
   `judge-approval` key로 검증한다.
6. Normal은 요청 시 1명, elevated는 3명 중 2명 PASS, critical은 3명 전원 PASS와
   별도 human approval을 요구한다.

Requester와 task agent는 roster나 approver가 될 수 없다. Critical approver는
resolved owner와 assigned judge identity도 될 수 없다. Trust root 전체에서 public-key
bytes는 unique여야 하므로 같은 key를 다른 key ID, identity 또는 purpose로 재등록할
수 없다.

## Detached attestation wire format

Artifact body schema는 기존 v1 bytes와 digest contract를 유지한다. Authentication은
`schemas/judge-attestation.schema.json`의 sidecar를 사용한다.

```json
{
  "schema_version": 1,
  "trust_root_id": "security-team",
  "artifact_kind": "verdict",
  "artifact_digest": "sha256:<JCS artifact digest>",
  "principal_id": "judge-instance-2",
  "key_id": "judge-2-2026",
  "signature": "ed25519:<128 lowercase hex>"
}
```

`artifact_digest`는 artifact 전체 object의 RFC 8785 JCS UTF-8 bytes에 대한 SHA-256이다.
Signature를 제외한 attestation object를 다시 JCS로 canonicalize하고 다음 exact
domain bytes를 앞에 붙인다.

```text
41 49 47 45 4e 54 2d 48 49 56 45 00
4a 55 44 47 45 2d 41 54 54 45 53 54 41 54 49 4f 4e 00
56 31 00
```

문자열 표현:

```text
"AIGENT-HIVE" || NUL || "JUDGE-ATTESTATION" || NUL || "V1" || NUL
|| UTF-8(JCS(attestation_without_signature))
```

Hive는 raw 32-byte Ed25519 public key와 raw 64-byte detached signature만 허용한다.
Wire encoding은 각각 lowercase hex `ed25519:<64 hex>`,
`ed25519:<128 hex>`다. OpenSSH `SSHSIG`, PEM, PKCS#8, private seed와 provider
credential은 이 contract가 아니다. Verification은 `ed25519-dalek`의
`VerifyingKey::from_bytes`와 `verify_strict`만 사용하며 signing API를 호출하지 않는다.

## 외부 TOML trust root

Trust root는 consumer target, release bundle과 Git tree 밖의 admin/user-controlled
TOML file이다. Normalized object는
`schemas/judge-trust-root.schema.json`을 따르며 다음 public data만 가진다.

- `trust_root_id`, monotonic `revision`, `issued_at`
- `root_digest`: 자신을 제외한 normalized object의 JCS SHA-256
- key별 `key_id`, `principal_id`
- exact purpose: `judge-assignment|judge-verdict|judge-approval`
- `algorithm = "ed25519"`와 public key
- `status = "active|revoked"`
- inclusive `valid_from`, `valid_until`

Unknown field와 private-key 형태는 closed schema로 거부한다. Revoked key는 artifact
timestamp와 관계없이 항상 무효다. Active key도 signed artifact의 `created_at`이
validity interval 안에 있어야 한다. Timestamp는 signature로 수정 방지되지만 signer가
실제 wall clock을 정직하게 기록했다는 별도 증명은 아니다.

## Agent-write-denied filesystem gate

`hive judge quorum --trust-root <absolute-path>`의 path는 request artifact나 target
config에서 선택하지 않는다. CLI는 다음 조건을 모두 확인한다.

- absolute path이며 consumer target 밖
- 모든 ancestor와 final path가 no-follow regular directory/file
- final file 256 KiB 이하, single-link, read 전후 file identity/size 불변
- current process가 file 또는 ancestor를 write/delete/replace/permission-change 불가

Unix/macOS에서는 file과 모든 ancestor를 root-owned로 제한한다. File은 write bit 0,
hard-link count 1이어야 하며 ancestor는 group/world write가 없어야 한다. Mode bit와
별개인 macOS ACL까지 반영하도록 current process의 effective write+execute access를
각 ancestor에서 검사하고, 실제 final-file write open도 실패해야 한다. Read 뒤 같은
file identity·size를 확인하고 protection 전체를 다시 검증하므로 pre-read path
substitution이 protected artifact로 승격되지 않는다.

Windows에서는 모든 component의 reparse point를 거부하고 current process token으로
file/ancestor의 write-data, append, EA/attribute write, child create/delete, `DELETE`,
`WRITE_DAC`, `WRITE_OWNER` 권한을 각각 probe한다. 오직 `ACCESS_DENIED`만 보호 증거로
인정하고 sharing violation이나 다른 오류는 fail closed한다. Installer는
`%ProgramData%\Aigent Hive\judge-trust-root.toml`에 admin ACL을 설정해야 한다.

이 gate는 현재 process token과 검증 시점의 권한을 증명한다. Administrator, 다른
token, 이미 열린 foreign handle 또는 이후 ACL 변경을 통제하지 않는다. 그런 authority
자체가 신뢰 경계다.

## v1/v2 compatibility

`judge-quorum-request` schema version 2만 detached attestation과 external trust root를
사용한다. Version 1 unsigned request는 기존 artifact/schema 진단을 위해 parse하지만
`authenticated:false`, `INDETERMINATE`로 끝나며 completion-authorizing PASS를 반환할
수 없다. Package bytes와 package digest contract는 바뀌지 않는다.

V2 request는 assignment attestation, verdict와 같은 순서의 verdict attestation,
optional approval과 approval attestation을 target-relative path로 참조한다. Missing,
invalid, unknown, revoked, expired, role-mismatched 또는 duplicate key는 PASS가 아닌
`INDETERMINATE`다.

## Private-key custody

Hive는 private key를 생성, 질문, 읽기, 저장, 전달, backup, migration 또는 logging하지
않는다. External signer는 OS keychain, hardware token, protected signing service 또는
user-presence-controlled tool을 사용할 수 있다. Agent가 unlocked signer를 사용자
확인 없이 호출할 수 있으면 독립성 보장이 사라지므로 signer policy도 별도로 보호해야
한다.

## 파일과 출력 경계

Package, assignment, verdict, attestation과 approval은 target-relative bounded
no-follow read다. Trust root만 별도 protected absolute path에서 읽는다. Quorum
command는 어느 쪽도 수정하지 않는다.

Aggregate output은 status, count, `authenticated`, authentication algorithm과 approval
validity만 제공한다. Identity, key ID, public key, signature, slot, finding, artifact
digest, statement와 개별 verdict는 출력하지 않는다.

## Conformance

필수 hostile corpus:

- self-authored owner/judge/human forgery
- target-contained, caller-writable, symlink/reparse와 replaceable trust root
- unknown/revoked/out-of-window key와 duplicate public key
- signature bit flip, wrong principal/purpose/domain/artifact digest
- assignment/verdict/approval replay와 timestamp/order violation
- missing attestation과 unsigned legacy request
- aggregate-output identity/signature leakage

Known-answer vector는 private key 없이 public key, canonical message와 signature만
source fixture에 둔다. Test용 private key, seed와 certificate는 commit하지 않는다.
