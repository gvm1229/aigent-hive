# Claude Code native usage sensor 조사

- 조사일: 2026-07-26
- 판정: machine-readable native source 존재, live qualification 필요
- CodexBar: explicit-consent fallback-only 후보

## 공식 surface

Claude Code status-line command는 session JSON을 stdin으로 수신. Subscription quota field:

- `rate_limits.five_hour.used_percentage`
- `rate_limits.five_hour.resets_at`
- `rate_limits.seven_day.used_percentage`
- `rate_limits.seven_day.resets_at`
- `session_id`
- `version`

`rate_limits` availability: Claude.ai Pro/Max subscriber의 first API response 이후. 각
window의 독립 omission 가능.

근거:

- [Claude Code status-line JSON](https://code.claude.com/docs/en/statusline)
- [Claude Code commands](https://code.claude.com/docs/en/commands)
- [Claude Code plugin settings 범위](https://code.claude.com/docs/en/plugins-reference)

## Integration 경계

- Plugin `bin/`의 stdin JSON capture executable만 projection
- User의 Claude host-owned `/statusline` opt-in 필요
- Hive의 `~/.claude/settings.json` read·write 없음
- Existing status line 자동 교체·wrapper 실행 없음
- Existing script composition용 copy-ready snippet과 exact command preview만 제공
- Workspace trust 거절, first API response 이전, missing window, stale callback:
  native unavailable
- Callback receipt time과 exact `session_id` binding으로 freshness 판정
- Raw stdin, transcript path, cwd, repository identity, account 정보 저장 없음
- Sanitized window·host·session digest·receipt time만 ignored runtime에 저장
- Native limited 판정 뒤 CodexBar fallback 금지

## 남은 qualification

- Actual Claude Pro/Max session
- 5-hour·7-day window parity
- Cancellation·concurrent callback·stale snapshot
- Existing status line non-clobber
- CodexBar fallback source의 credential-free Hive boundary
