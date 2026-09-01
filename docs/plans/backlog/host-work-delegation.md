# 작업 자동 분담

- 상태: `blocked`
- 마지막 검토일: 2026-08-20
- 관련 결정: [`ADR-0019`](../../decisions/ADR-0019-hive-native-iterative-execution.md)
- 완료 조사 계획: [`host-work-delegation-research-0.10.0.md`](../../archive/plans/releases/0.10.0/host-work-delegation-research-0.10.0.md)
- 조사 결과: [`host-work-delegation-2026-08-20.md`](../../research/host-work-delegation-2026-08-20.md)

## 문제

Hive 계획의 작업 분할·중단·재개·독립 검토를 Codex·Claude의 실제 별도 agent 실행과 안전하게 연결할 계약 필요.

## 기대 효과

- 긴 작업의 역할별 분담
- 독립 검토와 완료 판정 강화
- session 변경 뒤 작업 상태 복구

## 현재 제외 이유

- 요청 역할·model·추론 수준의 실제 사용 receipt 부족
- host별 취소·재개·late result 차이
- Antigravity의 tier-only model 계약과 로컬 CLI 부재

## 승격 조건

Codex·Claude의 fresh-session 실행·결과 receipt·mismatch fail-closed 증거, Antigravity의
exact model·effort mapping, 외부 signer 경계 확정.
