# 독립 security review 완료 계획

> Checklist owner: `SEC-*`
> Load condition: 독립 code·security review finding 교정·검증

Completion gate:

- [x] [SEC-001] Update target capability 고정과 전체 handle-relative no-follow 연산,
  root·ancestor·file 교체 adversarial regression
- [x] [SEC-002] 허가된 macOS·Windows signer identity의 evidence·TUF·installer·Rust verifier
  결합과 valid wrong-signer negative test
- [x] [SEC-003] Dry-run의 incomplete journal 자동 recovery 금지, recovery-required 오류와
  target·journal·state·index byte 불변 regression
- [x] [SEC-004] Windows direct installer의 binary·receipt 대칭 ownership 검사와
  receipt-only·binary-only·malformed·mismatch·reparse 실행 regression
