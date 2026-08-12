# ADR-0008: Release 출처와 local 무결성

- 상태: accepted, 2026-08-12 대체 결정
- 날짜: 2026-07-24, 2026-08-12

## 맥락

필요 보장:

- Artifact tampering·same-version substitution·downgrade 차단
- Migration drift와 중단된 activation 복구
- 반복 가능한 `0.9.0` publication
- 별도 signing ceremony·제품 내부 private key 의존 제거

## 결정

- Npm 출처: registry integrity·Trusted Publishing OIDC provenance
- GitHub 출처: protected `main` exact tag·commit, SHA-256 sidecar·artifact attestation
- `hive release verify`: local bundle version·artifact length·SHA-256 no-follow 검증
- Installed state: 마지막 수락 release version·sequence·manifest digest
- 거부: downgrade·같은 sequence의 다른 manifest
- Release surface delta: compile된 historical inventory와 target inventory 비교
- Cross-major apply: exact source·target, renderer plan, compatibility·preservation report,
  migration table digest 기반 human confirmation
- Update safety: backup·durable journal·atomic activation·실패 rollback·crash recovery
- 금지: release-provided executable migration, signing key, certificate, provider credential

## Publication

- Candidate: protected `main` exact commit에서 native archive 5개·npm package 6개·installer
  3개 한 번 build
- Human gate: protected stable environment 승인 한 번
- Promotion: 동일 bytes의 GitHub normal Release·npm `latest`
- Platform 상태: macOS ad-hoc seal·Windows unsigned 공개
- Paid code signing: 선택 기능

## 거부한 대안

| 대안 | 거부 이유 |
| --- | --- |
| Product-owned release signing key | Source·verifier·release authority의 단일 trust domain 결합 |
| 별도 offline threshold ceremony | 반복 가능한 publication 목표 대비 높은 운영 부담 |
| Checksum 단독 | Trusted build 출처 증명 불가 |
| Attestation 단독 | 다운로드 뒤 local bundle 변경·transaction safety 범위 밖 |
| Release script·WASM migration | Artifact의 arbitrary code execution channel 전환 |

## 결과

- 출처 증거와 local byte 검증 분리
- Exact candidate identity 유지
- Judge external trust root: 별도 보안 경계, 영향 없음
- 과거 release fixture: byte-immutable migration 증거로만 보존

상세 구조:
[`../architecture/release-update-trust-boundary.md`](../architecture/release-update-trust-boundary.md)

운영 절차: [`../guides/release-update.md`](../guides/release-update.md)
