## Reconfiguration

- Do not lead with a technical validation result. Complete any authenticated Hive-only refresh
  automatically, state whether settings are ready or local changes were preserved, then offer the
  relevant next meaningful preference choice.
- Without pending progress and without an explicitly named setting, start with `change one setting` or `review everything from the beginning`.
  With pending progress, offer `review everything`, `review selected settings`, or `continue from
  where I left off`; do not infer the choice.
- If the user already names the setting and desired value, use that value without asking again;
  show its current value and preview the requested change. Ask only for a missing required value.
  Keep all other answers unchanged. Otherwise `Change one setting` and `Review selected settings` begin with the full
  partial-reconfiguration catalog. Translate descriptions into the saved interface language,
  preserve product terms such as `Aigent Hive`, `Skill`, `Wiki`, `Discord`, and `CodexBar`, and
  show every parent and child as a separate Markdown list entry.
  1. **Interface language** — language for future Hive questions and summaries.
     - Values: `English` or `한국어`.
  2. **Daily update check** — checks for an update at most once every 24 hours and never installs it.
     - Value: enabled or disabled.
  3. **Wiki** — local Markdown knowledge Wiki and its writing language.
     - Enablement: enabled or disabled; disabling preserves existing Markdown.
     - Language: `en`, `ko`, or `both`.
  4. **User context** — background for Hive; it never selects a project workflow or priority.
     - Contexts: any combination of `web-developer`, `game-developer`, and `non-developer`.
     - Description: optional one-line background, interest, or preference.
  5. **Agent persona** — default communication style for Hive-assisted work.
     - Values: `strict`, `balanced`, `friendly`, or `custom`.
     - Custom description: required only when persona is `custom`.
  6. **Active hosts** — subscription hosts that receive user-scope setup.
     - Hosts: one or more of `codex`, `claude`, and `antigravity`.
  7. **Judge invocation** — independent acceptance-review policy.
     - Values: `explicit` or `implicit`.
     - `explicit`: iterative, team, and multi-goal terminal acceptance only.
     - `implicit`: explicit routes plus strict material-risk routes; simple, read-only,
       format-only, deterministic failure, tick, heartbeat, and retry routes remain excluded.
  8. **Built-in Skills** — active built-in Hive Skills.
     - Selection mode: all built-in Skills or individually selected built-in Skills.
     - Individual selection: one enabled/disabled decision per Skill; mandatory `user-setup` remains active.
  9. **Usage guard** — stops new Hive work at a chosen remaining-usage limit.
     - Enablement: enabled or disabled.
     - Stop threshold: integer from `1` through `99` percent remaining.
     - Discord usage notification: enabled or disabled; available only when the usage guard is enabled and sends outbound-only notices.
     - Discord webhook environment variable: uppercase variable name such as `HIVE_DISCORD_WEBHOOK_URL`; Hive records the name, never the URL.
     - Discord request privacy: default `summary` or explicit `raw-prompt` opt-in after preview and redaction.
     - Discord notification format: safe field list and order. Fields: `remaining-usage`, `project`, `request`, `progress`, `host`, `resume`, `measured-at`, and `evidence`.
     - Discord notification language: always the interface language. A test message differs from a real alert only by its first-line test disclaimer.
     - For a canonical run, the actual and test message show the run title and completed checklist
       count. Raw prompts, absolute paths, session identifiers, and webhook values remain local.
- After this catalog, ask for exactly one numbered parent setting or named child setting. Do not ask
  the user to rediscover a hidden Discord option through the usage-guard question.
- During a full review, language remains the first question and all saved quick-answers remain defaults.
- Preserve canonical Wiki Markdown when Wiki is disabled.
- Treat Wiki deletion, arbitrary host uninstall, Skill data deletion, and provider configuration
  changes as separate actions. Only authenticated Hive-owned recovery uses the bounded reinstall
  procedure below; a structurally valid manifest alone does not establish ownership.
- Re-run dry-run, apply, and validate with one consistent quick-answer file.
