# Bootstrap·global setup 복구 계획

> Checklist owner: `BGR-*`
> Target: 다음 `0.9.0-test.N`
> Decision: [`ADR-0012`](../../decisions/ADR-0012-global-onboarding-shared-index.md)

## 목표

- Hive 미설치 사용자의 선택형 one-prompt bootstrap
- Global setup의 초기·부분 변경·전체 검토·복구 상태별 쉬운 안내
- Authenticated user projection의 vanilla 교체와 local-priority three-way merge

## Checklist

- [ ] [BGR-001] User projection의 authenticated historical base·live local·incoming digest
  판별과 vanilla exact replacement
- [ ] [BGR-002] Modified directive·Skill의 local-priority three-way merge, disjoint incoming
  hunk 추가·overlap local 보존·unknown base fail-closed
- [ ] [BGR-003] Dry-run·apply·validate의 merge preview, changed/retained/omitted inventory와
  atomic rollback·recovery
- [ ] [BGR-004] Global setup 초기 language-first, 부분 변경, 전체 재검토의 one-question
  interaction과 drift 상태별 쉬운 안내
- [ ] [BGR-005] Hive 미설치 전 선택형 bootstrap prompt: release 선택·OS install·host activation·
  global-only setup·project setup opt-in 경계
- [ ] [BGR-006] Rust·Python regression: vanilla, disjoint/overlap local edit, unauthenticated
  base, setup UX·bootstrap docs/projection parity
- [ ] [BGR-007] English·Korean README·ADR·bilingual fact와 source Wiki current truth

## 완료 기준

- Vanilla user projection: authenticated old base에서 incoming exact replacement
- Modified user projection: preview의 local retained·incoming omitted disclosure, active
  conflict marker 0건
- Unknown·tampered base: write preview·apply 0건과 recoverable conflict
- 초기 global setup: language 질문 우선
- Reconfigure: 부분 변경 또는 전체 재검토 선택 우선
- Refresh 필요 상태: preference·project 안전 여부와 다음 선택 우선, internal path·hash 기본 노출 0건
- Bootstrap: project inspection·provider credential·silent installer 0건
