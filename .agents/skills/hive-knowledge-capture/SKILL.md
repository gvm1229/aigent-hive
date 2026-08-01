---
name: hive-knowledge-capture
description: Review every Wiki-enabled user turn for one durable fact, preference, or workflow and write only an agent-reviewed canonical claim; also ingest reviewed sources. Reject secret, confidential, ephemeral, ambiguous, and raw-session content.
---

# Hive Knowledge Capture

Run the mandatory memory gate, then preserve the existing explicit source-ingest path.

## Mandatory memory gate

1. When Wiki is enabled, review every user turn and completed task before the final response.
   Select only a durable, reusable `preference`, `workflow`, `decision`, `convention`,
   `project-profile`, or verified `outcome`. A normal question or answer is not a fact candidate.
2. If the target contains `hive-source.json`, route a material source-task fact to
   `$hive-source-wiki`. Never use consumer knowledge paths in the source workspace.
3. For consumer knowledge, reject secret, credential, confidential, ephemeral, ambiguous,
   speculative, private-path, raw transcript, complete conversation, hook payload, tool output,
   cache, database, and runtime content with canonical write count zero.
4. Normalize one atomic claim. Bind `collection_id`, stable `claim_key`, portable `locator`, kind,
   status, visibility, normalized fact, and reviewed provenance exactly as required by
   `knowledge-remember-request.schema.json`. Use `user-stated` only for explicit user intent,
   `observed` only for a reviewed artifact, and `verified` only with acceptance evidence. Do not
   retain the raw turn.
5. Run exactly one strict write-through request:

   ```text
   hive knowledge remember --user-root <user-root> --request <request.json> --output json
   ```

6. Require a schema-valid canonical Markdown and derived-index receipt before the final response.
   Identical input is a no-op. A contradiction, ambiguous scope, failed secret gate, or stale
   replacement digest stops the write and preserves current truth.

## Explicit source ingest

1. Confirm the source is explicitly selected or created by the current authorized task, bounded,
   non-confidential, and suitable for tracking.
2. Prepare an agent-reviewed Wiki Markdown draft that follows the installed knowledge schema and
   includes bounded outcome, criteria, and normalized provenance.
3. Run:

   ```text
   hive knowledge ingest --target <project-root> --user-root <user-root> --source <source-file> --wiki <reviewed-wiki-draft> --output json
   ```

4. Require a schema-valid success result and report its changed paths and evidence digest.
5. Run `hive knowledge lint --target <project-root> --user-root <user-root> --output json`.

## Safety

- Do not capture when Wiki is disabled.
- Never ingest a raw session, hidden prompt, secret, credential, or unbounded file.
- Keep Raw and Wiki Markdown canonical; treat SQLite as disposable derived state.
- Do not reproduce CLI mutation logic or write knowledge files directly when the command is unavailable.
