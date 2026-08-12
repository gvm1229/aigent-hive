# 독립 security review 완료 계획

> Checklist owner: `SEC-*`
> Load condition: 독립 code·security review finding 교정·검증

Completion gate:

- [x] [SEC-001] Update target capability 고정과 전체 handle-relative no-follow 연산,
  root·ancestor·file 교체 adversarial regression
- [x] [SEC-002] macOS ad-hoc·Windows unsigned 공개 상태, SHA-256·GitHub attestation,
  installer·Rust local verifier 결합과 digest mismatch negative test
- [x] [SEC-003] Dry-run의 incomplete journal 자동 recovery 금지, recovery-required 오류와
  target·journal·state·index byte 불변 regression
- [x] [SEC-004] Direct installer의 binary·receipt ownership, ancestor
  symlink·reparse 차단, safe leaf 교체, Windows pending-receipt recovery와 hostile regression

Residual boundary: portable shell·PowerShell bootstrap의 same-owner concurrent parent path
swap. Repeated no-follow·reopen, restrictive mode와 exact leaf move로 축소. Installed CLI
update path는 pinned directory capability 적용.
