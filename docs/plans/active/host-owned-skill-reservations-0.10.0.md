# Host-owned Skill 예약 `0.10.0`

> Checklist owner: `SCP10-002`

## Checklist

- [x] [SCP10-002] Codex·Antigravity `.agents/skills/<skill>/...`, Claude `.claude/skills/<skill>/...`의 host-matched 세션 예약 허용, 다른 host 경로 `hive.session-host-owned-namespace`, live·unverifiable reservation 한정 해결 안내, forbidden path 불변, three-host 회귀·문서 계약 검증 — `96f2b06`; Rust session unit 3건과 three-host consumer lifecycle 회귀 통과

## 경계

- 예약: `.hive/runtime`의 충돌 조정 기록
- Host-owned Skill bytes의 Hive 소유·수정 주장 금지
- 직접 사용자·외부 편집기 쓰기 통제 주장 금지
