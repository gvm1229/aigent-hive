# ADR-0013: `0.8.0` 프리뷰 릴리스 범위

- 상태: accepted
- 날짜: 2026-07-28
- Target: `0.8.0`
- 관련 결정: ADR-0008 verifier-only TUF, ADR-0010 native usage sensor,
  ADR-0012 global onboarding

## 결정

### Release label

- 공개 명칭: `0.8.0 Claude-unverified preview`
- Stable·production-ready 표현 금지
- Known limitation과 검증 host의 exact version 공개

### Host qualification

| Host | `0.8.0` gate |
| --- | --- |
| Codex | 실제 install·update·usage·prompt refine 검증 |
| Antigravity | 실제 install·update·fallback usage 검증 |
| Claude Code | Package·fixture·projection conformance와 unverified disclosure |
| Windows | 실제 기기 사용자 acceptance 필수 |

Claude Code 실제 subscription session과 Pro/Max usage parity:

- `0.8.0` 차단 조건 제외
- Subscription 확보 뒤 별도 qualification 재개
- 실제 검증 전 verified 지원 주장 금지

### Preview trust baseline

- Protected `main`·tag와 exact source commit
- GitHub Actions 기반 reproducible candidate build
- Release asset SHA-256
- GitHub artifact attestation과 source provenance
- Package-manager 또는 digest 고정 수동 update
- Network self-update 비활성

### Deferred hardening

- macOS Developer ID signing·notarization
- Windows Authenticode 또는 Azure Artifact Signing
- External TUF 2-of-3 production authorization
- Independent key-holder 운영 ceremony

ADR-0008의 verifier·migration·rollback 구현 보존. Deferred 항목은 future hardened update
channel의 자산으로 유지하고 `0.8.0` preview publication 필수 gate에서만 제외.

## 근거

- Preview 단계의 release friction 대비 과도한 private-key·external signer 운영 비용
- Download·self-update channel 부재에 따른 TUF production quorum의 낮은 즉시 효용
- GitHub artifact attestation·SHA-256·protected tag로 확보 가능한 초기 provenance
- Windows 실제 사용성 검증과 red CI 수정의 더 높은 release 위험 감소 효과
- Claude subscription 부재를 숨기지 않는 명시적 미검증 상태

## 결과

- `0.8.0`의 현실적 공개 경로
- 실제 검증과 이론 지원의 분리
- Heavy signing 미비를 안정성 검증 완료로 오인할 위험 차단
- Existing verifier code와 future hardened channel의 보존
- Production publication credential과 사용자 최종 확인의 외부 경계 유지
