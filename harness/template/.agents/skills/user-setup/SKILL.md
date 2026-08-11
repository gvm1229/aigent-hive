---
name: user-setup
description: Configure or reconfigure global user-scope Hive preferences. Use for bare Hive setup or preference changes without an explicit project, repository, folder, or path; never inspect or configure a project harness.
---

# Setup Hive

Configure user-scope Hive preferences without modifying a project harness or provider credentials.

## Scope Routing

- Select this Skill for global or user-scope preference, language, host, Skill, Wiki, persona,
  or usage-guard requests.
- Treat a bare request to set up, install, configure, or reconfigure Hive as global user-scope
  setup. Do not inspect an ambient working directory or request a project path.
- When the user explicitly requests both global and project setup, finish global setup first, then
  ask whether to start the separately scoped project setup.
- Never create, preview, or apply a project harness through this Skill.

## Workflow

1. Resolve the installed signed CLI before asking any preference question.
   - On Windows, run this Skill's `scripts/resolve-hive.ps1`. It tries `Get-Command hive`, then
     `where.exe hive`, then `(npm prefix -g) + '\\hive.cmd'`, and verifies the selected exact
     executable with its own `--version` call. Use the returned absolute executable path for the
     remaining commands; do not require a `PATH` refresh or a copied path from the user.
   - On other systems, use `command -v hive` and `hive --version`.
   - If no authenticated CLI is available, stop before questions and give the exact repair command. Never ask for a copied path, recursively search npm folders, or reproduce setup writes manually.
2. Read `hive setup --scope user --describe --output json` and use only its schema, localized catalog, question order, and answer example. Never guess a YAML key, Skill ID, or default.
3. Detect whether this is initial setup or reconfiguration.
   - Resolve the exact `<user-root>` selected during user installation.
   - For reconfiguration, read the saved quick-answers and run
     `hive setup --scope user --quick-answers <user-root>/.hive/config/user-setup.yml --user-root <user-root> --validate --output json` before offering writes.
   - Do not show raw path, hash, manifest, projection, or drift diagnostics by default.
   - If validation finds an authenticated Hive-file refresh, run the smallest matching
     `hive install --scope user --host <host> --dry-run --output json`, then apply that exact
     safe Hive-owned refresh automatically. If that preview reports a structurally valid installed
     ownership manifest that no longer matches an authenticated Hive release, run
     `hive uninstall --user-root <user-root> --output json` followed by
     `hive install --scope user --host <host> --apply --user-root <user-root> --output json`
     automatically. This preserving reinstall is an authorized setup or update recovery: it removes
     only Hive-managed activation, projections, packages, indexes, backups, and runtime state, while
     preserving canonical knowledge and saved preferences. If the saved-quick-answer validation reports an outdated
     user projection, run `hive setup --scope user --quick-answers <saved-quick-answers> --user-root
     <user-root> --dry-run --output json`, then apply that exact projection refresh automatically.
     Rerun both validations. The explicit global setup request already authorizes these
     deterministic prerequisites; do not ask whether to review or continue.
   - State a short plain-language result, then begin the next meaningful setup question. If the
     preview preserves local edits, state that they were preserved; do not ask a review-only
     question. Ask only if authentication fails, the preview needs a material user choice, or a
     separate authority boundary applies.
   - Explain the underlying file or digest only after the user asks `Why?` or requests diagnostics.
   - Inspect pending non-secret setup progress with `hive setup --progress status --scope user
     --user-root <user-root> --output json`. If no pending progress exists, use the normal
     initial or reconfiguration route.
3. For initial setup, ask for interface language first.
   - Offer `English` and `한국어`.
   - After the user chooses, ask every remaining question and explain every preview in
     that language.
   - Start with this one question only: `Welcome to Aigent Hive. Would you like to continue in English or Korean?`
4. For a valid reconfiguration without pending progress, start with this one question in the saved interface language:
   `Your Hive settings are ready. Would you like to change one setting or review everything from the beginning?`
   - `Change one setting`: first show the full partial-reconfiguration catalog below in the saved
     interface language. This required list is not an examples-only prompt: do not say `for
     example`, use an ellipsis, or omit conditional children. Then show the current quick-answer for
     each requested setting, preserve every other quick-answer, and ask one question at a time.
   - `Review everything`: ask the interface-language question first, using the saved language as
     the default, then ask every remaining setup question one at a time with saved quick-answers as
     defaults.
   - When pending progress exists, offer exactly these three choices instead: `Review everything`,
     `Review selected settings`, or `Continue from where I left off`.
   - `Continue from where I left off`: keep the recorded non-secret quick-answers, recheck every saved
     host receipt, and restart at the pending step. Never trust a prior OAuth or webhook result.
5. Ask whether daily update checking should be enabled.
   - This is explicit opt-in and defaults to disabled.
   - Explain the 24-hour successful-check throttle, next-session offline retry, and no-install
     boundary.
6. Ask for setup mode in the selected language.
   - Offer `Expedited — set everything to default` and `Custom`.
   - Initial setup asks for update-check consent next. Expedited performs no further preference
     questions after that consent and uses the fixed defaults below.
   - Custom asks exactly one question at a time in the required order below.
   - Reconfiguration preserves existing quick-answers and asks only for requested changes.
7. Resolve expedited defaults from the signed catalog.
   - Interface language: the language already selected by the user.
   - Daily update check: the explicit quick-answer already selected by the user.
   - Wiki: enabled with the selected interface language.
   - User contexts: `general knowledge work` with no additional description.
   - Agent persona: `strict`.
   - Active hosts: the current authenticated host only.
   - Skills: `all` mode with every built-in Skill in the signed catalog.
   - Usage guard: disabled. A remaining-usage threshold is requested only when the user enables it.
   - Selecting expedited authorizes the displayed built-in dependency closure only. It never
     approves a third-party Skill, CodexBar installation, credential access, or destructive action.
8. Ask exactly one custom-setup question at a time in the required order below.
   - Explain the available values from the signed user-setup catalog.
   - Do not infer a preference, host selection, custom description, Skill approval, usage-guard opt-in, or fallback consent.
   - Present every user-facing choice, Skill list, dependency list, and write list as one complete
     Markdown list or table entry per line. Never combine independently selectable items into a
     comma-separated paragraph.
9. Write the resolved answers to one session-scoped operating-system temporary YAML file matching `user-setup.schema.json`. Update it atomically and delete it after success, failure, or cancel.
   - Do not include provider credentials, tokens, cookies, account identifiers, or raw usage data.
10. After every completed setup answer, save the complete current non-secret
    quick-answer set and the next step with `hive setup --progress save --scope user --step <step>
    --quick-answers <quick-answers.yml> --user-root <user-root> --output json`.
    - Hive stores no OAuth token, webhook URL, raw prompt, transcript, or absolute path in this
      progress record.
    - On an integration error or interrupted conversation, stop safely. The next configuration
      request must offer the three choices above.
11. Preview the resolved setup.
   - Run `hive setup --scope user --quick-answers <quick-answers.yml> --user-root <user-root> --dry-run --output json`.
   - Show selected hosts and Skills, mandatory Skills, dependency closure, skipped components, marker edits, and conflicts.
   - A preview that identifies an authenticated Hive-owned incomplete activation, Hive marketplace mismatch, or structurally valid ownership manifest that lacks an authenticated release match is a recovery plan, not a user-visible compatibility problem. For the manifest-mismatch case, automatically use the preserving uninstall then reinstall path above. Keep the preview successful, preserve canonical knowledge and saved preferences, and continue to automatic apply without requesting host-configuration steps from the user.
12. Apply automatically after a conflict-free built-in-only preview. The explicit global setup request already authorizes this Hive-owned apply. Ask only for a conflict, third-party Skill, external installation, secret access, or destructive action.
   - Run `hive setup --scope user --quick-answers <quick-answers.yml> --user-root <user-root> --apply --output json`.
   - Preserve foreign bytes and third-party marker blocks.
   - Before activation, automatically recover the exact authenticated Hive-owned host entry when the preview planned recovery. The recovery may remove and reinstall Hive-owned marketplace or plugin state, but never deletes canonical knowledge, saved preferences, or foreign host entries. Do not report internal marketplace, manifest, transaction, or compatibility details unless the user asks for diagnostics.
13. Validate with the same quick-answers.
   - Run `hive setup --scope user --quick-answers <quick-answers.yml> --user-root <user-root> --validate --output json`.
   - Report the canonical user setup path, active hosts, active Skills, Wiki state, usage-guard state, and any unsupported host capability.
   - Clear completed progress only after successful apply and validation with `hive setup --progress
     clear --scope user --user-root <user-root> --output json`.

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
9. **Skills** — every built-in Skill is active by default.
   - Ask whether to keep every built-in Skill active or choose Skills individually.
   - For individual choice, present every built-in Skill as one Markdown list item per line and collect each on/off decision independently.
   - Always include mandatory `user-setup` and preview the full dependency closure.
   - Never derive active Skills from the user profile, persona, or host selection.
   - Existing `recommended` configuration is a legacy saved value. Preserve its exact recorded closure until the user reviews and approves a new `all` or `individual` preview.
   - The signed catalog's `optional_third_party_skills` list is empty in this release. Do not offer or activate a third-party Skill until a later release defines its explicit consent contract.
10. **Usage guard** — explicit opt-in; default disabled.
   - When enabled, ask the user to choose an integer remaining threshold from `1` through `99`.
   - Do not propose a percentage preset, use a percentage in a sample answer, or silently reuse a
     profile-specific value. A new setup has no default usage threshold.
   - A project may later choose its own registered project identity and an equal-or-higher threshold.
     The global value remains the minimum protection for every project; no project category has a preset value.
11. **Discord usage notification** — ask only when the usage guard is enabled.
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
12. **CodexBar fallback** — ask only when the usage guard is enabled and the active-host native sensor is unavailable, unsupported, or malformed.
   - Explain that CodexBar is fallback-only and never overrides a native success or limited decision.
   - Record whether an already-qualified CodexBar fallback may be used.
   - If installation is needed, show the exact fixed command and request separate current-action consent. Never persist installation consent or infer it from the setup quick-answer.
13. **Preview and automatic apply** — show the exact write set and dependency closure, then apply
    a conflict-free built-in-only result without another approval question.

## Reconfiguration

- Do not lead with a technical validation result. Complete any authenticated Hive-only refresh
  automatically, state whether settings are ready or local changes were preserved, then offer the
  relevant next meaningful preference choice.
- Without pending progress, start with `change one setting` or `review everything from the beginning`.
  With pending progress, offer `review everything`, `review selected settings`, or `continue from
  where I left off`; do not infer the choice.
- `Change one setting` and `Review selected settings` must both begin with the full
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
  7. **Built-in Skills** — active built-in Hive Skills.
     - Selection mode: all built-in Skills or individually selected built-in Skills.
     - Individual selection: one enabled/disabled decision per Skill; mandatory `user-setup` remains active.
  8. **Usage guard** — stops new Hive work at a chosen remaining-usage limit.
     - Enablement: enabled or disabled.
     - Stop threshold: integer from `1` through `99` percent remaining.
     - CodexBar fallback: enabled or disabled; available only when the native sensor is unavailable, unsupported, or malformed.
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
- Treat Wiki deletion, host uninstall, Skill data deletion, and provider configuration changes as separate destructive actions outside this Skill.
- Re-run dry-run, apply, and validate with one consistent quick-answer file.

## Clean reinstall

- Use this route for an explicit user request to remove and reinstall Hive's user-scope files, or automatically when an authorized user setup, install, or update detects a structurally valid ownership manifest without an authenticated Hive release match.
- Run `hive uninstall --user-root <user-root> --output json`. This removes Hive-managed host
  activation, projections, packages, indexes, backups, and runtime state while preserving
  `.hive/knowledge/` and saved user preferences.
- Reinstall the selected saved host with `hive install --scope user --host <saved-host> --apply
  --user-root <user-root> --output json`. A valid saved preference file is reused without setup
  questions. Then run the saved-answer `dry-run`, `apply`, and `validate` sequence.
- Hive provides no command to remove the knowledge base or saved preferences. Those files remain
  manual user-owned deletion targets outside this Skill.

## Korean interaction contract

When the selected interface language is Korean, retain product terms and identifiers exactly as
`Aigent Hive`, `Skill`, `Wiki`, `Codex`, `Claude`, `Antigravity`, `CodexBar`, commands,
paths, schema keys, Skill IDs, and versions. Do not translate `Skill` as `기술`.

Use these host-independent question patterns, one question at a time:

1. `계속 사용할 언어를 선택해 주세요: English 또는 한국어.`
2. `사용자 기본 맥락을 선택해 주세요. 여러 항목을 함께 선택할 수 있으며, 이 선택은 프로젝트의 작업 흐름이나 우선순위를 정하지 않습니다.`
   - `웹 개발`: 웹 애플리케이션 관련 배경 또는 관심사
   - `게임 개발`: 게임 관련 배경 또는 관심사
   - `일반 지식 작업`: 소프트웨어 개발 외 배경 또는 관심사
3. `추가로 알려 줄 배경, 관심사, 선호가 있나요? 없으면 건너뛸 수 있습니다.`
4. `Skill 선택 방식을 골라 주세요.`
   - `모든 내장 Skill 사용`
   - `개별 내장 Skill 선택`
5. `Wiki는 이 컴퓨터의 Markdown 파일에 저장합니다. Obsidian 같은 앱으로 열 수 있습니다.`
6. `사용량 한도에 도달했을 때 Discord 알림을 받을까요?`
   - `아니요`
   - `예, 시험 알림도 보내기`
7. `Discord webhook URL을 환경 변수에 저장해 주세요. 예: HIVE_DISCORD_WEBHOOK_URL. URL 자체는
   보내지 말고 환경 변수 이름만 알려 주세요. Hive가 시험 알림을 보내 연결을 확인합니다.`
8. `Discord 알림에 넣을 항목과 순서를 알려 주세요. 기본값은 남은 사용량, 프로젝트, 요청, 진행 상태, 호스트, 계속하기입니다. 시험 알림은 실제 알림과 같은 형식이며 첫 줄에만 시험 안내가 추가됩니다.`

For Korean partial reconfiguration, show this complete catalog before asking which setting to change:

`변경할 전역 설정을 하나 선택해 주세요. 아래는 변경할 수 있는 모든 설정입니다.`

1. **인터페이스 언어** — 이후 Hive 질문과 요약에 사용할 언어
   - 선택: `English` 또는 `한국어`
2. **일일 업데이트 확인** — 24시간에 한 번 업데이트 존재 여부만 확인하며 자동 설치 없음
   - 선택: 켜기 또는 끄기
3. **Wiki** — 이 컴퓨터의 Markdown 지식 Wiki와 작성 언어
   - 사용 여부: 켜기 또는 끄기. 끄더라도 기존 Markdown 보존
   - 작성 언어: `en`, `ko`, 또는 `both`
4. **사용자 기본 맥락** — Hive가 사용자의 배경과 관심사를 이해하기 위한 정보. 프로젝트 작업 흐름·우선순위 결정 없음
   - 맥락: `web-developer`, `game-developer`, `non-developer` 중 복수 선택 가능
   - 추가 설명: 선택 사항인 한 줄 배경·관심사·선호
5. **에이전트 페르소나** — Hive 지원 작업의 기본 대화 방식
   - 선택: `strict`, `balanced`, `friendly`, 또는 `custom`
   - 사용자 지정 설명: `custom` 선택 때만 필수
6. **사용할 호스트** — 전역 Hive 설정을 적용할 subscription host
   - 호스트: `codex`, `claude`, `antigravity` 중 하나 이상
7. **내장 Skill** — 활성화할 내장 Hive Skill
   - 선택 방식: 모든 내장 Skill 사용 또는 개별 내장 Skill 선택
   - 개별 선택: 각 Skill의 켜기·끄기. 필수 `user-setup`는 계속 활성화
8. **사용량 보호** — 남은 사용량이 정한 기준에 도달할 때 새 Hive 작업 중지
   - 사용 여부: 켜기 또는 끄기
   - 중단 기준: 남은 사용량 `1`%부터 `99`%까지
   - CodexBar 대체 수단: 켜기 또는 끄기. 호스트 기본 감지기를 쓸 수 없을 때만 사용
   - Discord 사용량 알림: 켜기 또는 끄기. 사용량 보호를 켠 경우에만 선택 가능하며 Hive에서 Discord로 보내는 알림만 지원
   - Discord webhook 환경 변수: `HIVE_DISCORD_WEBHOOK_URL` 같은 대문자 환경 변수 이름. Hive는 URL 자체를 저장하지 않음
   - Discord 요청 공개 범위: 기본 `summary` 또는 preview·redaction 뒤 명시적으로 선택한 `raw-prompt`
   - Discord 알림 형식: 안전한 항목의 포함 여부와 순서. `remaining-usage`, `project`, `request`, `progress`, `host`, `resume`, `measured-at`, `evidence` 중 선택
   - Discord 알림 언어: 인터페이스 언어와 동일. 시험 알림은 실제 중단 알림과 같은 항목·순서·언어이며 첫 줄에만 시험 안내 추가

Then ask for one numbered parent setting or named child setting. Do not replace this catalog with a
single examples-only sentence.

Do not describe a global user context as a role that prioritizes web, game, non-development, or
any other project workflow. Project setup alone determines project-specific workflow, technical
choices, constraints, and delivery priorities.

## Safety Invariants

- Operate only at user scope. Never add or update a project harness.
- Use `hive install --scope user --host <host>` only for the authenticated Hive-file refresh
  prerequisite; use `hive setup --scope user` for all preference state changes.
- Never call a model-provider API or request, read, store, or forward provider credentials.
- Never read or mutate `.omx/`, `.omc/`, provider runtime state, or host-global configuration.
- Never activate an optional third-party Skill through this workflow.
- Never install CodexBar without exact current-action consent.
- Never commit or push unless the user explicitly requests that Git operation.
