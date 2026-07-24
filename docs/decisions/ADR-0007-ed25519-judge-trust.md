# ADR-0007: Ed25519 judge identity authentication

- 상태: accepted
- 날짜: 2026-07-24

## 맥락

Digest-bound assignment, verdict와 human approval만으로는 task agent가 decoy
task-agent ID, owner, judge 세 명과 human identity를 함께 만들고 모든 공개 JCS
digest를 재계산하는 공격을 막을 수 없다. 이 구조는 artifact 일관성은 증명하지만
독립 identity와 private authority는 증명하지 않는다.

## 결정

Phase 5 authenticated quorum은 user/admin-controlled, agent-write-denied TOML public-key
trust root와 detached Ed25519 attestation을 사용한다.

- 기존 judge package/assignment/verdict/approval schema v1과 digest bytes는 유지
- `judge-quorum-request` v2가 assignment, 각 verdict와 approval attestation을 추가
- Signature는 domain-separated RFC 8785 JCS bytes에 대한 raw Ed25519
- owner, judge와 human approver key purpose를 분리
- trust root 전체에서 public-key bytes 중복 금지
- revoked key는 항상 무효, active key는 signed artifact timestamp validity 적용
- unsigned v1 quorum은 diagnostic compatibility만 제공하고 PASS 권한 없음
- Hive는 `ed25519-dalek` strict verification만 사용하고 signing/private-key API를
  제품 코드에서 사용하지 않음

Trust root path는 consumer target 밖이어야 한다. Unix/macOS는 root-owned,
non-writable, non-replaceable path를 요구한다. Windows는 reparse point를 거부하고
현재 process token의 file/ancestor mutation 권한이 모두 `ACCESS_DENIED`인지 검증한다.

## Private-key boundary

Hive는 private key, seed, PEM, PKCS#8, hardware-token secret, provider credential을
질문·생성·읽기·저장·전달하지 않는다. Signing과 user-presence policy는 external
signer/host authority가 소유한다.

## 거부한 대안

- target 내부 self-certified key: task agent가 key와 artifact를 함께 교체 가능
- caller-supplied public-key digest: 공개 입력으로 다시 계산 가능
- opaque host ID만 신뢰: runtime correlation은 가능하지만 cryptographic identity 아님
- Hive signing/key generation: credential custody와 provider-neutral verifier 경계 위반
- `ring`/`aws-lc-rs`: Ed25519 verification 하나에 불필요한 native crypto/build surface

## 결과

- task agent가 project artifact 전체를 수정해도 trusted private key 없이 PASS chain을
  만들 수 없음
- trust-root provisioning과 key rotation/revocation은 admin/user 운영 책임
- Signature는 trusted-key possession과 exact signed binding을 증명하지만 judge 판단의
  정직성, 실제 사람의 생체 presence와 전역 replay 방지는 증명하지 않음
- 상세 wire/filesystem contract는
  [`../architecture/judge-trust-boundary.md`](../architecture/judge-trust-boundary.md)를
  따름
