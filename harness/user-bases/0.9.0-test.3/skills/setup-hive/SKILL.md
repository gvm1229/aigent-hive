---
name: setup-hive
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
   - For reconfiguration, run `hive setup --scope user --answers <user-root>/.hive/config/user-setup.yml --user-root <user-root> --validate --output json` first.
   - Preserve existing answers unless the user changes them.
3. For initial setup, ask for interface language first.
   - Offer `English` and `한국어`.
   - After the user chooses, ask every remaining question and explain every preview in
     that language.
4. Ask whether daily update checking should be enabled.
   - This is explicit opt-in and defaults to disabled.
   - Explain the 24-hour successful-check throttle, next-session offline retry, and no-install
     boundary.
5. Ask for setup mode in the selected language.
   - Offer `Expedited — set everything to default` and `Custom`.
   - Initial setup asks for update-check consent next. Expedited performs no further preference
     questions after that consent and uses the fixed defaults below.
   - Custom asks exactly one question at a time in the required order below.
   - Reconfiguration preserves existing answers and asks only for requested changes.
6. Resolve expedited defaults from the signed catalog.
   - Interface language: the language already selected by the user.
   - Daily update check: the explicit answer already selected by the user.
   - Wiki: enabled with the selected interface language.
   - User profile: `custom` with the fixed description
     `General user; infer domain context from each project.`
   - Agent persona: `strict`.
   - Active hosts: the current authenticated host only.
   - Skills: `individual` mode with every built-in Skill in the signed catalog.
   - Usage guard: disabled, stored default remaining threshold `20`, CodexBar fallback disabled.
   - Selecting expedited authorizes the displayed built-in dependency closure only. It never
     approves a third-party Skill, CodexBar installation, credential access, or destructive action.
7. Ask exactly one custom-setup question at a time in the required order below.
   - Explain the available values from the signed user-setup catalog.
   - Do not infer a preference, host selection, custom description, Skill approval, usage-guard opt-in, or fallback consent.
8. Write the resolved answers to a temporary YAML file matching `user-setup.schema.json`.
   - Do not include provider credentials, tokens, cookies, account identifiers, or raw usage data.
9. Preview the resolved setup.
   - Run `hive setup --scope user --answers <answers.yml> --user-root <user-root> --dry-run --output json`.
   - Show selected hosts and Skills, mandatory Skills, dependency closure, skipped components, marker edits, and conflicts.
   - Require explicit approval of the displayed dependency closure before apply.
10. Apply only after preview approval or expedited selection with a conflict-free built-in-only preview.
   - Run `hive setup --scope user --answers <answers.yml> --user-root <user-root> --apply --output json`.
   - Preserve foreign bytes and third-party marker blocks.
11. Validate with the same answers.
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
6. **User profile** — `web-developer`, `game-developer`, `non-developer`, or `custom`.
   - For `custom`, ask the next single question for a non-empty custom description.
7. **Agent persona** — `strict`, `balanced`, `friendly`, or `custom`.
   - For `custom`, ask the next single question for a non-empty custom description.
8. **Active hosts** — select one or more of `codex`, `claude`, and `antigravity`.
9. **Skill selection mode** — a signed recommended suite or individual built-in Skills.
   - Recommended: ask which suite from the signed catalog.
   - Individual: present existing built-ins and collect the selection.
   - Always include mandatory `setup-hive` and preview the full dependency closure.
   - The signed catalog's `optional_third_party_skills` list is empty in this release. Do not offer or activate a third-party Skill until a later release defines its explicit consent contract.
10. **Usage guard** — explicit opt-in; default disabled.
   - When enabled, offer the default remaining threshold `20` before asking for a different integer from `1` through `99`.
11. **CodexBar fallback** — ask only when the usage guard is enabled and the active-host native sensor is unavailable, unsupported, or malformed.
   - Explain that CodexBar is fallback-only and never overrides a native success or limited decision.
   - Record whether an already-qualified CodexBar fallback may be used.
   - If installation is needed, show the exact fixed command and request separate current-action consent. Never persist installation consent or infer it from the setup answer.
12. **Preview approval** — approve or reject the exact write set and dependency closure.

## Reconfiguration

- Show current answers before asking for changes.
- Preserve canonical Wiki Markdown when Wiki is disabled.
- Treat Wiki deletion, host uninstall, Skill data deletion, and provider configuration changes as separate destructive actions outside this Skill.
- Re-run dry-run, apply, and validate with one consistent answer file.

## Safety Invariants

- Operate only at user scope. Never add or update a project harness.
- Use only `hive setup --scope user` for setup state changes.
- Never call a model-provider API or request, read, store, or forward provider credentials.
- Never read or mutate `.omx/`, `.omc/`, provider runtime state, or host-global configuration.
- Never activate an optional third-party Skill through this workflow.
- Never install CodexBar without exact current-action consent.
- Never commit or push unless the user explicitly requests that Git operation.
