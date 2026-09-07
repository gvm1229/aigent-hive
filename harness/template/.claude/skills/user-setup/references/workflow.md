## Workflow

Read only the branch required by the request: initial setup uses steps 1–3 and Question Order;
an explicitly named setting uses Reconfiguration for that setting only; interrupted setup uses
saved progress; authenticated repair uses Clean reinstall. Shared apply and validation rules
apply to each branch. Do not load every question catalog for a named single-setting change.

1. Locate the installed CLI before asking any preference question. Version output establishes
   executable discovery, not cryptographic authentication; require the CLI's release ownership
   checks before trusting an installed generation for mutation.
   - On Windows, run this Skill's `../scripts/resolve-hive.ps1`. It tries `Get-Command hive`, then
     `where.exe hive`, then `(npm prefix -g) + '\\hive.cmd'`, and verifies the selected exact
     executable with its own `--version` call. Use the returned absolute executable path for the
     remaining commands; do not require a `PATH` refresh or a copied path from the user.
   - On other systems, use `command -v hive` and `hive --version`.
   - If CLI discovery or authentication fails, diagnose the exact issue read-only before asking
     preference questions. Use an already authorized repair when the CLI can prove its scope;
     otherwise report the exact missing user action. Never recursively search npm folders or
     reproduce setup writes manually.
2. Read `hive setup --scope user --describe --output json` and use only its schema, localized catalog, question order, and answer example. Never guess a YAML key, Skill ID, or default.
3. Detect whether this is initial setup or reconfiguration.
   - Resolve the exact `<user-root>` selected during user installation.
   - For reconfiguration, read the saved quick-answers and run
     `hive setup --scope user --quick-answers <user-root>/.hive/config/user-setup.yml --user-root <user-root> --validate --output json` before offering writes.
   - Do not show raw path, hash, manifest, projection, or drift diagnostics by default.
   - If validation finds an authenticated Hive-file refresh, run the smallest matching
     `hive install --scope user --host <host> --dry-run --output json`, then apply that exact
     safe Hive-owned refresh automatically. If release ownership is unverified, preserve files and
     follow [recovery](recovery.md); structural validity alone never permits uninstall. If the saved-quick-answer validation reports an outdated
     user projection, run `hive setup --scope user --quick-answers <saved-quick-answers> --user-root
     <user-root> --dry-run --output json`, then apply that exact projection refresh automatically.
     Rerun both validations. The explicit global setup request already authorizes these
     deterministic prerequisites; do not ask whether to review or continue.
   - During that authenticated refresh, remove a retired Hive Skill only when its retired-name
     ledger entry and historical Hive digest prove ownership. Preserve a same-named modified or
     foreign Skill, and remove empty Hive-owned parent directories after a successful deletion.
   - State a short plain-language result, then begin the next meaningful setup question. If the
     preview preserves local edits, state that they were preserved; do not ask a review-only
     question. Diagnose authentication failures first. Ask only if diagnosis needs a user action,
     the preview needs a material user choice, or a
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
4. For a valid reconfiguration without pending progress or a named setting, start with this one question in the saved interface language:
   `Your Hive settings are ready. Would you like to change one setting or review everything from the beginning?`
   - `Change one setting` without a named setting: first show the full partial-reconfiguration catalog below in the saved
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
   - Update-check consent was collected before setup mode; do not repeat it. Expedited uses the
     fixed defaults below, except the separately required new-feature question in step 15.
   - Custom asks exactly one question at a time in the required order below.
   - Reconfiguration preserves existing quick-answers and asks only for requested changes.
7. Resolve expedited defaults from the signed catalog.
   - Interface language: the language already selected by the user.
   - Daily update check: the explicit quick-answer already selected by the user.
   - Wiki: enabled with the selected interface language.
   - User contexts: `general knowledge work` with no additional description.
   - Agent persona: `strict`.
   - Active hosts: the current authenticated host only.
   - Judge invocation: `explicit`.
   - Skills: `all` mode with every built-in Skill in the signed catalog.
   - Usage guard: enabled with a `20%` remaining threshold.
   - Selecting expedited authorizes the displayed built-in dependency closure only. It never
     approves a third-party Skill, CodexBar installation, credential access, or destructive action.
8. Ask exactly one custom-setup question at a time in the required order below.
   - Explain the available values from the signed user-setup catalog.
   - Do not infer a preference, host selection, custom description, Skill approval, usage-guard disablement, or fallback consent.
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
    - On an integration error, pause only that integration, preserve progress, diagnose within
      existing authority, and continue independent setup work. Ask only for an exact unresolved
      user action. On user cancellation or conversation interruption, preserve progress; the next
      configuration request uses the saved-progress choices above.
11. Preview the resolved setup.
   - Run `hive setup --scope user --quick-answers <quick-answers.yml> --user-root <user-root> --dry-run --output json`.
   - Show selected hosts and Skills, mandatory Skills, dependency closure, skipped components, marker edits, and conflicts.
   - An authenticated incomplete activation can enter the CLI recovery path. An unverified
     manifest is a conflict, never a successful preview or automatic uninstall authority.
     Use [recovery](recovery.md), preserving canonical knowledge and saved preferences.
12. Apply automatically after a conflict-free built-in-only preview. The explicit global setup request already authorizes this Hive-owned apply. Ask only for a conflict, third-party Skill, external installation, secret access, or destructive action.
   - Run `hive setup --scope user --quick-answers <quick-answers.yml> --user-root <user-root> --apply --output json`.
   - Preserve foreign bytes and third-party marker blocks.
   - Before activation, recover an interrupted operation only through its authenticated journal
     within existing authority. Any uninstall/reinstall follows [recovery](recovery.md), including
     its removal-plan and destructive-scope approval requirements. Preserve canonical knowledge,
     saved preferences, and foreign entries. Explain unresolved risks even without a diagnostics request.
13. Validate with the same quick-answers.
   - Run `hive setup --scope user --quick-answers <quick-answers.yml> --user-root <user-root> --validate --output json`.
   - Report the canonical user setup path, active hosts, active Skills, Wiki state, usage-guard state, and any unsupported host capability.
   - Clear completed progress only after successful apply and validation with `hive setup --progress
     clear --scope user --user-root <user-root> --output json`.
14. When the usage guard is enabled, probe native usage availability without invoking a fallback.
   - For an active Codex host, run `hive usage probe-native --host codex --output json` after
     successful apply and validation. Do not mention, ask about, inspect, or invoke CodexBar before
     this command reports `hive.usage-native-fallback-eligible`.
   - `hive.usage-native-available` and all native limited decisions complete setup with no
     CodexBar question or invocation.
   - For Claude and Antigravity, defer the native probe until the first active-host usage check.
     Do not turn the lack of an initialization-time host session into fallback consent.
   - After `hive.usage-native-fallback-eligible`, explain the exact native failure and separately
     ask whether the qualified CodexBar fallback may be used. Installation still requires a second,
     current-action consent for the exact fixed `hive usage fallback-install` command.
   - `hive.usage-native-failed-closed` is an integrity or safety failure. Do not offer or invoke a
     fallback for that result.

15. Ask the vector-search question last for both expedited and custom setup.
   - Run `hive setup feature claim --id vector-search --user-root <user-root> --output json` first.
   - Ask only when `question_required` is true. Explain that exact search remains available, first preparation can take time, and the measured Windows runtime is about 376MB.
   - When another session already holds an unanswered claim, do not ask a duplicate question. A later session may claim again after the short local claim expires; never store a host session identifier.
   - For yes, save the answer with `hive setup feature answer --id vector-search --answer yes --user-root <user-root> --output json`, then return the `prompt` field from `hive setup feature prompt --id vector-search --user-root <user-root> --output json` as a new-session prompt. Preserve its fixed collection list and `setup_request_digest`; changed collections require a new preview rather than a quiet scope expansion.
   - For no, save `--answer no` and never ask again unless the user explicitly requests vector-search setup. Do not save a no answer after silence, cancellation, or an interrupted setup.
