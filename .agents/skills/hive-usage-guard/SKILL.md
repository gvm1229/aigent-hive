---
name: hive-usage-guard
description: Control and enforce Aigent Hive's source-development native-first usage safeguard for every task in the current session. Use automatically when the user clearly asks to change the remaining-usage threshold, turn the guard on or off, bypass or restore the limit, use quota below the stop line, inspect guard status, or verify task blocking; the user does not need to name this Skill.
---

# Hive Usage Guard

Use the bundled script as the only mutation surface:

```bash
python3 .agents/skills/hive-usage-guard/scripts/guard.py <command> --json
```

Run commands from the Aigent Hive source root. Keep consumer harness behavior unchanged.

## Commands

- Gate every user turn and auto-start the watcher: `gate`
- Inspect and enforce now: `check`
- Show quota, selected window, threshold, override, halt marker, and watcher: `status`
- Persist a source-local threshold from 1 through 99: `set-threshold PERCENT`
- Disable only the current session after explicit user intent:
  `session-disable --confirm-session-disable`
- Re-enable the current session: `session-enable`
- Toggle the current session. Pass `--confirm-session-disable` when the toggle would disable:
  `session-toggle --confirm-session-disable`
- Start or inspect the independent watcher: `watch-start`, `watch-status`
- Stop only the verified current-session watcher: `watch-stop`
- Preview the fixed CodexBar fallback install command for one provider:
  `fallback-install --host codex|claude|antigravity --dry-run`
- Apply that fixed install command only after explicit current-action consent:
  `fallback-install --host codex|claude|antigravity --apply --confirm-install`

## Safety contract

1. Before routing, answering, planning, loading another Skill, or using any non-guard tool for
   every user turn, run `gate`. This requirement precedes the simple-question gate.
2. While `gate` exits `10` or `11`, block every ordinary task in the same session. Exit `10`
   means a confirmed limit. Exit `11` is reserved for non-transient session, state, path, or
   sensor-integrity failures. The only permitted work is guard status, threshold, enable,
   disable, toggle, or consented fallback install control and the minimal reply that explains
   the block.
3. Run `status` before changing state.
4. Change the threshold only when the user supplies the new percentage.
5. Disable or toggle off when the user's natural language has obvious current-session bypass
   intent even if the user does not name this Skill. High-confidence examples include “turn off
   the usage guard,” “bypass the limit for this session,” “use the remaining quota,” “continue
   below the threshold,” and “do not stop at the usage limit.” Pass
   `--confirm-session-disable` because that prompt is the confirmation evidence.
6. Re-enable immediately for obvious restore intent such as “turn the usage guard back on,”
   “enforce the limit again,” “stop at the configured threshold,” or “remove the bypass.”
7. Do not infer bypass from urgency, an active goal, “finish this,” or a bare “continue”/“resume”
   that does not mention the usage guard, quota, limit, threshold, or remaining allowance.
8. Treat a session override as non-transferable. The script binds it to the current Codex session
   identifier and process; a new session starts enabled.
9. Run `gate` again before every tool, mutation, delegation, push, external write, and final task
   answer. The independent watcher updates the halt marker between boundaries.
10. Retry a quota-sensor `usage_unknown` once after three seconds. If the retry is still unknown,
    preserve the diagnostic and fallback notification, mark `transient_unknown_ignored`, create
    no new halt marker, and allow the boundary. Never let transient unknown clear an earlier
    confirmed limited marker. When `gate` exits `10` or non-transient `11`, stop at the current
    safe boundary. Do not use `omx cancel` success as proof that a durable goal or Codex App task
    stopped.
11. Use the qualified Codex `app-server` sensor first. Invoke the optional CodexBar adapter at most
   once and only when the native sensor is unavailable, unsupported, or malformed. Never use the
   fallback to override a native limited decision.
12. When CodexBar is used, prefer its primary session window and use the weekly window only when
   the primary window is absent. Treat missing or unqualified CodexBar as a retryable
   `usage_unknown`.
13. When CodexBar is missing, return the provider-specific notification and exact `--dry-run`
   preview. Decline leaves core guard controls usable; the source boundary proceeds only through
   the bounded transient-unknown rule.
14. Never install CodexBar silently. The preview qualifies only the supported package-manager
   adapter and executes nothing. Apply requires `--confirm-install` for that action, uses
   non-interactive bounded execution, then requalifies CodexBar.
15. Never request credentials, reinstall a provider CLI, suggest API-key or manual-cookie setup,
   or take ownership of a package manager.
16. Never edit `.omx/`, provider credentials, consumer `.hive/`, or harness templates through this
   Skill.

The source-local setting, session override, observations, halt marker, watcher PID, and logs live
under ignored `.agents/work/usage-guard/`.
