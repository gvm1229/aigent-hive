---
name: configure
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

1. Check the signed CLI with `hive setup --scope user --help`.
   - If unavailable, report that user setup is unsupported in the installed release.
   - Never reproduce setup writes manually.
2. Detect whether this is initial setup or reconfiguration.
   - Resolve the exact `<user-root>` selected during user installation.
   - For reconfiguration, read the saved answers and run
     `hive setup --scope user --answers <user-root>/.hive/config/user-setup.yml --user-root <user-root> --validate --output json` before offering writes.
   - Do not show raw path, hash, manifest, projection, or drift diagnostics by default.
   - If validation finds an authenticated Hive-file refresh, run the smallest matching
     `hive install --scope user --host <host> --dry-run --output json`, then apply that exact
     safe Hive-owned refresh automatically. If the saved-answer validation reports an outdated
     user projection, run `hive setup --scope user --answers <saved-answers> --user-root
     <user-root> --dry-run --output json`, then apply that exact projection refresh automatically.
     Rerun both validations. The explicit global setup request already authorizes these
     deterministic prerequisites; do not ask whether to review or continue.
   - State a short plain-language result, then begin the next meaningful setup question. If the
     preview preserves local edits, state that they were preserved; do not ask a review-only
     question. Ask only if authentication fails, the preview needs a material user choice, or a
     separate authority boundary applies.
   - Explain the underlying file or digest only after the user asks `Why?` or requests diagnostics.
3. For initial setup, ask for interface language first.
   - Offer `English` and `한국어`.
   - After the user chooses, ask every remaining question and explain every preview in
     that language.
   - Start with this one question only: `Welcome to Aigent Hive. Would you like to continue in English or Korean?`
4. For a valid reconfiguration, start with this one question in the saved interface language:
   `Your Hive settings are ready. Would you like to change one setting or review everything from the beginning?`
   - `Change one setting`: show the current answer for each requested setting, preserve every
     other answer, and ask one question at a time.
   - `Review everything`: ask the interface-language question first, using the saved language as
     the default, then ask every remaining setup question one at a time with saved answers as
     defaults.
5. Ask whether daily update checking should be enabled.
   - This is explicit opt-in and defaults to disabled.
   - Explain the 24-hour successful-check throttle, next-session offline retry, and no-install
     boundary.
6. Ask for setup mode in the selected language.
   - Offer `Expedited — set everything to default` and `Custom`.
   - Initial setup asks for update-check consent next. Expedited performs no further preference
     questions after that consent and uses the fixed defaults below.
   - Custom asks exactly one question at a time in the required order below.
   - Reconfiguration preserves existing answers and asks only for requested changes.
7. Resolve expedited defaults from the signed catalog.
   - Interface language: the language already selected by the user.
   - Daily update check: the explicit answer already selected by the user.
   - Wiki: enabled with the selected interface language.
   - User contexts: `general knowledge work` with no additional description.
   - Agent persona: `strict`.
   - Active hosts: the current authenticated host only.
   - Skills: `all` mode with every built-in Skill in the signed catalog.
   - Usage guard: disabled, stored default remaining threshold `20`, CodexBar fallback disabled.
   - Selecting expedited authorizes the displayed built-in dependency closure only. It never
     approves a third-party Skill, CodexBar installation, credential access, or destructive action.
8. Ask exactly one custom-setup question at a time in the required order below.
   - Explain the available values from the signed user-setup catalog.
   - Do not infer a preference, host selection, custom description, Skill approval, usage-guard opt-in, or fallback consent.
   - Present every user-facing choice, Skill list, dependency list, and write list as one complete
     Markdown list or table entry per line. Never combine independently selectable items into a
     comma-separated paragraph.
9. Write the resolved answers to a temporary YAML file matching `user-setup.schema.json`.
   - Do not include provider credentials, tokens, cookies, account identifiers, or raw usage data.
10. Preview the resolved setup.
   - Run `hive setup --scope user --answers <answers.yml> --user-root <user-root> --dry-run --output json`.
   - Show selected hosts and Skills, mandatory Skills, dependency closure, skipped components, marker edits, and conflicts.
   - Require explicit approval of the displayed dependency closure before apply.
11. Apply only after preview approval or expedited selection with a conflict-free built-in-only preview.
   - Run `hive setup --scope user --answers <answers.yml> --user-root <user-root> --apply --output json`.
   - Preserve foreign bytes and third-party marker blocks.
12. Validate with the same answers.
   - Run `hive setup --scope user --answers <answers.yml> --user-root <user-root> --validate --output json`.
   - Report the canonical user setup path, active hosts, active Skills, Wiki state, usage-guard state, and any unsupported host capability.

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
   - Always include mandatory `configure` and preview the full dependency closure.
   - Never derive active Skills from the user profile, persona, or host selection.
   - Existing `recommended` configuration is a legacy saved value. Preserve its exact recorded closure until the user reviews and approves a new `all` or `individual` preview.
   - The signed catalog's `optional_third_party_skills` list is empty in this release. Do not offer or activate a third-party Skill until a later release defines its explicit consent contract.
10. **Usage guard** — explicit opt-in; default disabled.
   - When enabled, offer the default remaining threshold `20` before asking for a different integer from `1` through `99`.
11. **CodexBar fallback** — ask only when the usage guard is enabled and the active-host native sensor is unavailable, unsupported, or malformed.
   - Explain that CodexBar is fallback-only and never overrides a native success or limited decision.
   - Record whether an already-qualified CodexBar fallback may be used.
   - If installation is needed, show the exact fixed command and request separate current-action consent. Never persist installation consent or infer it from the setup answer.
12. **Preview approval** — approve or reject the exact write set and dependency closure.

## Reconfiguration

- Do not lead with a technical validation result. Complete any authenticated Hive-only refresh
  automatically, state whether settings are ready or local changes were preserved, then offer the
  relevant next meaningful preference choice.
- Start with `change one setting` or `review everything from the beginning`; do not assume which
  preference the user wants to change.
- During a full review, language remains the first question and all saved answers remain defaults.
- Preserve canonical Wiki Markdown when Wiki is disabled.
- Treat Wiki deletion, host uninstall, Skill data deletion, and provider configuration changes as separate destructive actions outside this Skill.
- Re-run dry-run, apply, and validate with one consistent answer file.

## Korean interaction contract

When the selected interface language is Korean, retain product terms and identifiers exactly as
`Aigent Hive`, `Skill`, `Wiki`, `Codex`, `Claude`, `Antigravity`, `CodexBar`, `Notion`, commands,
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
