# Judge Ed25519 신뢰 경계

Hive의 judge 기능: 모델 실행기나 signing agent가 아닌 로컬 검증기.
`package-review`와 Hive CLI의 judge, model, process, subagent 생성·직접 호출 금지.
실제 독립 judge 실행과 private-key signing은 이미 resolve된
host/OMX/OMC owner와 사용자가 승인한 외부 signer가 소유.

## 보안 목표

Authenticated quorum의 방어 대상:

- task agent가 owner, judge와 human identity 문자열을 임의로 만든 뒤 공개 digest를
  다시 계산해 self-approval하는 공격
- 같은 key를 여러 identity나 quorum slot으로 재사용하는 공격
- assignment, verdict 또는 approval을 수정한 뒤 기존 signature를 replay하는 공격
- owner key를 judge/human key로, judge key를 human key로 사용하는 role confusion
- target 안의 self-certified public key나 caller-writable trust root를 신뢰하는 공격

Ed25519는 trusted public key에 대응하는 private key 보유자가 exact bytes에 서명했다는
사실을 검증. Content encryption, judge 판단의 진실성, 실제 wall-clock 시각 또는
전역 one-shot replay 방지 미제공.

## Artifact chain

판정 체인 순서:

1. `judge-package`가 subject, risk tier, acceptance criteria, artifact/evidence digest를
   RFC 8785 JCS digest로 고정.
2. `judge-assignment`가 verdict 이전에 exact package/criteria, requester, task agent,
   resolved owner, owner provenance와 distinct slot/instance/eligibility tuple을 고정.
3. `assignment` detached attestation을 `judge-assignment` purpose의 trusted owner key로
   검증. 이 signature가 task-agent identity와 전체 roster를 인증.
4. 각 `judge-verdict`는 exact assignment와 slot tuple에 결합되며 별도 detached
   attestation을 해당 principal의 `judge-verdict` key로 검증.
5. Critical 판정의 `judge-approval`은 모든 eligible verdict 뒤 생성하고 별도
   `judge-approval` key로 검증.
6. Normal은 요청 시 1명, elevated는 3명 중 2명 PASS, critical은 3명 전원 PASS와
   별도 human approval을 요구.

Requester와 task agent의 roster·approver 참여 금지. Critical approver와 resolved
owner·assigned judge identity의 중복 금지. Trust root 전체 public-key bytes는 unique.
동일 key의 다른 key ID·identity·purpose 재등록 금지.

## Detached attestation wire format

Artifact body schema는 기존 v1 bytes와 digest contract를 유지. Authentication은
`schemas/judge-attestation.schema.json`의 sidecar를 사용.

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

`artifact_digest`: artifact 전체 object의 RFC 8785 JCS UTF-8 bytes에 대한 SHA-256.
Signature를 제외한 attestation object를 다시 JCS로 canonicalize하고 다음 exact
domain bytes를 앞에 추가.

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

Hive는 raw 32-byte Ed25519 public key와 raw 64-byte detached signature만 허용.
Wire encoding은 각각 lowercase hex `ed25519:<64 hex>`,
`ed25519:<128 hex>`. OpenSSH `SSHSIG`, PEM, PKCS#8, private seed와 provider
credential은 이 contract의 범위 밖. Verification은 `ed25519-dalek`의
`VerifyingKey::from_bytes`와 `verify_strict`만 사용하며 signing API 호출 금지.

## 외부 TOML trust root

Trust root는 consumer target, release bundle과 Git tree 밖의 admin/user-controlled
TOML file. Normalized object는
`schemas/judge-trust-root.schema.json`을 준수하며 다음 public data로 제한.

- `trust_root_id`, monotonic `revision`, `issued_at`
- `root_digest`: 자신을 제외한 normalized object의 JCS SHA-256
- key별 `key_id`, `principal_id`
- exact purpose: `judge-assignment|judge-verdict|judge-approval`
- `algorithm = "ed25519"`와 public key
- `status = "active|revoked"`
- inclusive `valid_from`, `valid_until`

Unknown field와 private-key 형태는 closed schema로 거부. Revoked key는 artifact
timestamp와 관계없이 항상 무효. Active key도 signed artifact의 `created_at`이
validity interval 안에 있어야 유효. Timestamp는 signature로 수정 방지되지만 signer의
실제 wall clock 기록 진실성은 별도 증명 범위 밖.

## Agent-write-denied filesystem gate

`hive judge quorum --trust-root <absolute-path>`의 path는 request artifact나 target
config에서 선택 없음. CLI는 다음 조건을 모두 확인.

- absolute path이며 consumer target 밖
- 모든 ancestor와 final path가 no-follow regular directory/file
- final file 256 KiB 이하, single-link, read 전후 file identity/size 불변
- current process가 file 또는 ancestor를 write/delete/replace/permission-change 불가

Unix/macOS에서는 file과 모든 ancestor를 root-owned로 제한. File은 write bit 0,
hard-link count는 1, ancestor의 group/world write는 없어야 유효. Mode bit와
별개인 macOS ACL까지 반영하도록 current process의 effective write+execute access를
각 ancestor에서 검사하고, 실제 final-file write open도 실패 필수. Read 뒤 같은
file identity·size를 확인하고 protection 전체를 다시 검증하므로 pre-read path
substitution의 protected artifact 승격 차단.

Windows에서는 모든 component의 reparse point를 거부하고 current process token으로
file/ancestor의 write-data, append, EA/attribute write, child create/delete, `DELETE`,
`WRITE_DAC`, `WRITE_OWNER` 권한을 각각 probe. 오직 `ACCESS_DENIED`만 보호 증거로
인정하고 sharing violation이나 다른 오류는 fail closed. Installer는
`%ProgramData%\Aigent Hive\judge-trust-root.toml`의 admin ACL 설정 필수.

이 gate는 현재 process token과 검증 시점의 권한을 증명. Administrator, 다른
token, 이미 열린 foreign handle 또는 이후 ACL 변경을 통제 범위 밖. 그런 authority
자체가 신뢰 경계.

## v1/v2 compatibility

`judge-quorum-request` schema version 2만 detached attestation과 external trust root를
사용. Version 1 unsigned request는 기존 artifact/schema 진단을 위해 parse하지만
`authenticated:false`, `INDETERMINATE`로 종료하며 completion-authorizing PASS 반환
불가. Package bytes와 package digest contract는 불변.

V2 request는 assignment attestation, verdict와 같은 순서의 verdict attestation,
optional approval과 approval attestation을 target-relative path로 참조. Missing,
invalid, unknown, revoked, expired, role-mismatched 또는 duplicate key의 결과:
PASS가 아닌 `INDETERMINATE`.

## Private-key custody

Hive의 private key 생성·질문·읽기·저장·전달·backup·migration·logging 금지.
External signer 허용 범위: OS keychain, hardware token, protected signing service,
user-presence-controlled tool. Agent가 unlocked signer를 사용자
확인 없이 호출할 수 있으면 독립성 보장 상실. Signer policy 별도 보호 필수.

## 파일과 출력 경계

Package, assignment, verdict, attestation과 approval은 target-relative bounded
no-follow read. Trust root만 별도 protected absolute path에서 read-only 조회. Quorum
command의 수정 0건.

Aggregate output은 status, count, `authenticated`, authentication algorithm과 approval
validity만 제공. Identity, key ID, public key, signature, slot, finding, artifact
digest, statement와 개별 verdict는 출력에서 제외.

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
source fixture에 배치. Test용 private key, seed와 certificate는 commit 금지.
