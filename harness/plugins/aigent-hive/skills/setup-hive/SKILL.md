---
name: setup-hive
description: Configure or reconfigure provider-neutral Aigent Hive preferences at user scope through the signed Hive CLI. Use after minimal user installation; when setup is required; or when changing interface language, Wiki language or enablement, user profile, agent persona, active hosts, built-in Skill selection, or usage-guard preferences.
---

# Setup Hive

Configure user-scope Hive preferences without modifying a project harness or provider credentials.

## Workflow

1. Check the signed CLI with `hive setup --scope user --help`.
   - If unavailable, report that user setup is unsupported in the installed release.
   - Never reproduce setup writes manually.
2. Detect whether this is initial setup or reconfiguration.
   - Resolve the exact `<user-root>` selected during user installation.
   - For reconfiguration, run `hive setup --scope user --answers <user-root>/.hive/config/user-setup.yml --user-root <user-root> --validate --output json` first.
   - Preserve existing answers unless the user changes them.
3. Ask exactly one question at a time in the required order below.
   - Explain the available values from the signed user-setup catalog.
   - Do not infer a preference, host selection, custom description, Skill approval, usage-guard opt-in, or fallback consent.
4. Write the collected answers to a temporary YAML file matching `user-setup.schema.json`.
   - Do not include provider credentials, tokens, cookies, account identifiers, or raw usage data.
5. Preview the resolved setup.
   - Run `hive setup --scope user --answers <answers.yml> --user-root <user-root> --dry-run --output json`.
   - Show selected hosts and Skills, mandatory Skills, dependency closure, skipped components, marker edits, and conflicts.
   - Require explicit approval of the displayed dependency closure before apply.
6. Apply only after preview approval.
   - Run `hive setup --scope user --answers <answers.yml> --user-root <user-root> --apply --output json`.
   - Preserve foreign bytes and third-party marker blocks.
7. Validate with the same answers.
   - Run `hive setup --scope user --answers <answers.yml> --user-root <user-root> --validate --output json`.
   - Report the canonical user setup path, active hosts, active Skills, Wiki state, usage-guard state, and any unsupported host capability.

## Question Order

Ask only one question, wait for the answer, then continue.

1. **Interface language** — `en` or `ko`.
2. **Wiki language** — `en`, `ko`, or `both`.
3. **Wiki enablement** — default `enabled`; offer explicit opt-out without deleting canonical Markdown.
4. **User profile** — `web-developer`, `game-developer`, `non-developer`, or `custom`.
   - For `custom`, ask the next single question for a non-empty custom description.
5. **Agent persona** — `strict`, `balanced`, `friendly`, or `custom`.
   - For `custom`, ask the next single question for a non-empty custom description.
6. **Active hosts** — select one or more of `codex`, `claude`, and `antigravity`.
7. **Skill selection mode** — a signed recommended suite or individual built-in Skills.
   - Recommended: ask which suite from the signed catalog.
   - Individual: present existing built-ins and collect the selection.
   - Always include mandatory `setup-hive` and preview the full dependency closure.
   - The signed catalog's `optional_third_party_skills` list is empty in this release. Do not offer or activate a third-party Skill until a later release defines its explicit consent contract.
8. **Usage guard** — explicit opt-in; default disabled.
   - When enabled, offer the default remaining threshold `20` before asking for a different integer from `1` through `99`.
9. **CodexBar fallback** — ask only when the usage guard is enabled and the active-host native sensor is unavailable, unsupported, or malformed.
   - Explain that CodexBar is fallback-only and never overrides a native success or limited decision.
   - Record whether an already-qualified CodexBar fallback may be used.
   - If installation is needed, show the exact fixed command and request separate current-action consent. Never persist installation consent or infer it from the setup answer.
10. **Preview approval** — approve or reject the exact write set and dependency closure.

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
