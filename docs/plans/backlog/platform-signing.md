# Platform signing

- 상태: `blocked`
- 마지막 검토일: 2026-08-20
- 관련 안내: [`code-signing-policy.md`](../../guides/code-signing-policy.md)

## 문제

macOS Developer ID·notarization과 Windows Authenticode publisher identity 부재.

## 기대 효과

- 운영체제 경고 감소
- 배포 publisher identity 확인

## 현재 제외 이유

유료 또는 외부 승인 기반 인증서·서명 환경 필요. 현재 SHA-256·GitHub attestation·npm provenance 유지.

## 승격 조건

지속 가능한 무료 또는 승인된 서명 수단과 보호된 CI custody 확정.
