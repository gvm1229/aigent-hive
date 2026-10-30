## Question Order

For initial setup, ask interface language first and immediately switch to it. Then ask update-check
consent and setup mode. Ask the remaining preference questions only for `Custom`.

1. **Interface language** — `English` (`en`) or `한국어` (`ko`).
2. **Daily update check** — explicit opt-in; default disabled. Explain that it checks at most once
   per successful 24-hour window, retries on the next host session after offline failure, and
   never installs an update.
3. **Setup mode** — `Expedited — set everything to default` or `Custom`.
4. **Wiki language** — `en`, `ko`, or `both`.
5. **Wiki enablement** — default `enabled`; offer explicit opt-out without deleting canonical Markdown.
6. **User contexts** — select any combination of `web-developer`, `game-developer`, and
   `non-developer`.
   - These contexts help Hive understand the user. They never select a project workflow,
     implementation approach, delivery priority, or active Skill set.
   - Ask one optional follow-up question for a single-line user description. A description is
     required only when no context is selected.
7. **Agent persona** — `strict`, `balanced`, `friendly`, or `custom`.
   - For `custom`, ask the next single question for a non-empty custom description.
8. **Active hosts** — select one or more of `codex`, `claude`, and `antigravity`.
9. **Judge invocation** — `explicit` (recommended) or `implicit`.
   - `explicit` invokes the independent Judge only at an iterative, team, or multi-goal terminal
     acceptance gate.
   - `implicit` additionally permits a strict material-risk route; simple questions, read-only,
     format-only, deterministic failure, tick, heartbeat, and retry routes remain excluded.
10. **Skills** — every built-in Skill is active by default.
   - Ask whether to keep every built-in Skill active or choose Skills individually.
   - For individual choice, present every built-in Skill as one Markdown list item per line and collect each on/off decision independently.
   - Always include mandatory `user-setup` and preview the full dependency closure.
   - Never derive active Skills from the user profile, persona, or host selection.
   - Existing `recommended` configuration is a legacy saved value. Preserve its exact recorded closure until the user reviews and approves a new `all` or `individual` preview.
   - The signed catalog's `optional_third_party_skills` list is empty in this release. Do not offer or activate a third-party Skill until a later release defines its explicit consent contract.
11. **Usage guard** — offer `Enabled (recommended)` first and `Disabled` second.
   - Expedited setup enables protection at `20%` remaining without another question.
   - In Custom setup, when enabled, ask the user to choose an integer remaining threshold from `1`
     through `99`; do not silently replace that choice with the expedited default.
   - A project may later choose its own registered project identity and an equal-or-higher threshold.
     The global value remains the minimum protection for every project; no project category has a preset value.
12. **Discord usage notification** — ask only when the usage guard is enabled.
   - Offer `No` by default and `Yes — send a test notification` as the opt-in choice.
   - When enabled, guide the user to create one Discord incoming webhook and set its URL in a
     local environment variable such as `HIVE_DISCORD_WEBHOOK_URL`. Hive records only the
     variable name, never the webhook URL.
   - Confirm that the environment variable name is uppercase letters, digits, and underscores,
     then ask for the notification fields in the exact order the user wants. The default order is:
     - `remaining-usage`
     - `project`
     - `request`
     - `progress`
     - `host`
     - `resume`
   - The notification language always follows the selected interface language. Do not mix Korean
     and English labels in one notification.
   - Interpret a request such as “include remaining usage and project in Korean” as the typed
     `message_fields` selection plus the already-selected interface language. Do not invent an
     unbounded free-text webhook template.
   - Run `hive discord test --webhook-env <ENVIRONMENT_NAME> --language <en|ko> --fields
     <ordered-field-list> --output json`. Its payload must use the same fields, order, and
     language as a real usage-guard alert. Only its first line identifies it as a test message
     and explains that the user may freely ask to change the format.
   - A sent test permits the next question. A missing, invalid, offline, or rejected delivery
     keeps the integration disabled and preserves progress at `discord-test`.
   - The installed visual guide is `<user-root>/.hive/guides/discord-usage-notifications.html`.
     Open that exact local file only when the user asks for a visual guide; do not inspect a project.
13. **Preview and automatic apply** — show the exact write set and dependency closure, then apply
    a conflict-free built-in-only result without another approval question.
