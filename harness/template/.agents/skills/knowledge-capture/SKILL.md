---
name: knowledge-capture
description: (knowledge-capture) At the end of a Wiki-enabled turn, keep one useful fact, preference, or workflow that will help later work; never save secrets, raw conversations, or uncertain guesses.
---

# Capture One Knowledge Claim (`knowledge-capture`)

For each eligible turn, write at most one atomic knowledge claim.

In a consumer project, apply `.agents/directives/00-project-harness.md`'s availability gate
before this command-backed workflow. Ordinary collaboration skips capture and leaves canonical
knowledge unchanged; never imitate a successful write manually. Installed errors are not
non-installation. Source-workspace and already activated user-level contracts remain unchanged.

## Mandatory memory gate

1. When Wiki is enabled, review every user turn and completed task before the final response.
   Select only a durable, reusable `preference`, `workflow`, `decision`, `convention`,
   `project-profile`, or verified `outcome`. A normal question or quick-answer is not a fact candidate.
   The selected host's user-level guidance applies in every folder immediately after Hive
   installation. Project setup, a Hive harness, a project marker, or an attached collection is not
   a prerequisite. Never skip this gate only because the current project is unregistered. Store a
   safe user-global fact at `user-root`; keep ambiguous project-specific scope fail-closed.
2. If the target contains `hive-source.json`, use `hive source-wiki` for the material
   source-task fact. Never use consumer knowledge paths in the source workspace.
3. For consumer knowledge, reject secret, credential, confidential, ephemeral, ambiguous,
   speculative, private-path, raw transcript, complete conversation, hook payload, tool output,
   cache, database, and runtime content with canonical write count zero.
4. For a safe explicit `user-root` user statement, normalize one atomic fact and use a stable
   `claim_key` plus `project-profile|decision|convention|preference|workflow`; do not create a
   request JSON or a provenance digest. Use the strict request schema only for reviewed artifacts,
   verified outcomes, replacements, or another supported scope. Do not retain the raw turn.
5. Run exactly one write-through request. Prefer the simple user-statement route:

   ```text
   hive knowledge remember --user-root <user-root> --user-statement <normalized-fact> --claim-key <stable-key> --kind <preference|workflow|decision|convention|project-profile> --output json
   ```

   For a reviewed artifact or another supported scope, use:

   ```text
   hive knowledge remember --user-root <user-root> --request <request.json> --output json
   ```

6. Require a schema-valid canonical Markdown and derived-index receipt before the final response.
   After a successful write, route lint by target class. A valid `hive-source.json` selects `hive
   source-wiki lint --target <source-root> --output json`. An enabled registered consumer project
   selects `hive knowledge lint --target <current-project-root> --user-root <user-root> --output
   json`. An unregistered consumer project selects `hive knowledge lint --target <user-root>
   --user-root <user-root> --output json`, which validates the canonical user-root store and its
   derived shared index. Missing project setup, a project marker, or an attached collection never
   skips lint. Identical input is a no-op. A contradiction, ambiguous scope, failed secret gate,
   or stale replacement digest stops the write and preserves current truth.

## Explicit source ingest

For an explicitly selected source or a source created by the authorized task, read [source ingest](references/ingest.md). Ordinary one-fact capture does not load that procedure.

## Safety

- Do not capture when Wiki is disabled.
- Never ingest a raw session, hidden prompt, secret, credential, or unbounded file.
- Keep Raw and Wiki Markdown canonical; treat SQLite as disposable derived state.
- Do not reproduce CLI mutation logic or write knowledge files directly when the command is unavailable.
