# Antigravity native usage sensor 조사

- 조사일: 2026-07-26
- 문서 version: Antigravity CLI `1.1.7`, Antigravity `2.3.1`
- 판정: documented machine-readable native source 없음
- CodexBar: explicit-consent fallback-only 후보

## 공식 surface

Antigravity CLI `/usage`와 `/quota`:

- Backend와 disk에서 fresh quota refresh
- Model별 remaining request·token 표시
- Interactive TUI panel
- Keyboard navigation과 panel close 필요
- JSON·JSONL·machine output mode 문서 없음

Quota는 prompt count가 아닌 agent work amount와 capacity에 따라 변화. Local token
estimate의 subscription remaining 대체 금지.

근거:

- [Antigravity Model Quotas](https://antigravity.google/docs/cli/commands/usage)
- [Antigravity Plans](https://antigravity.google/docs/plans)

## Integration 경계

- `/usage` TUI text·screen scraping 금지
- Undocumented local LSP/HTTP·disk cache·backend endpoint probing 금지
- OAuth·browser state·provider credential 접근 금지
- Official CLI·SDK가 structured quota JSON·event·IPC를 문서화하고 live probe까지
  성공한 뒤 native adapter qualification
- 그 전까지 `native=unsupported`
- CodexBar Antigravity adapter는 separately qualified external fallback
- Native unsupported와 sensor failure의 별도 상태 유지
- Native limited 상태가 향후 제공되면 CodexBar fallback 금지

## 남은 qualification

- Official CLI·SDK version matrix의 release별 재검사
- CodexBar local Antigravity source의 bounded fixed-argv·sanitized output
- Actual Antigravity host와 fallback parity
- Future native structured surface fixture·live probe
