---
name: knowledge-transfer
description: (knowledge-transfer) Move the user's existing portable Hive knowledge safely between computers without re-extracting it from source folders.
---

# Transfer Hive Knowledge (`knowledge-transfer`)

Use this Skill only for an existing Hive knowledge base moving between computers. Use
`knowledge-scan` when the user wants new knowledge extracted from a repository or folder.

1. Resolve the installed user root and operating system. Show the selected portable collections,
   exclusions, bundle path, SHA-256, and destination free-space check before creating a bundle.
2. Run `hive knowledge transfer export --preview --user-root <root> --scope all-portable
   --bundle <file.hivekb> --output json`. Show counts, exclusions, size, and digest. Use the same
   command with `--apply` to export. Preserve its `archive_sha256` with the file. Never include
   FTS, vector indexes, models, host settings, credentials, or confidential collections.
3. On the destination, run `hive knowledge transfer import --preview --user-root <root>
   --bundle <file.hivekb> --expected-sha256 <sender-digest> --output json`. Show `conflict_paths`.
   Offer keeping local content and excluding conflicts, or cancellation; never force overwrites.
   Apply with `--apply --preview-digest <transfer_preview_digest> --expected-sha256 <sender-digest>`.
   Add `--exclude-conflicts` only for that explicit choice. A changed preview requires review again.
4. Check `transfer.complete` and query representative restored knowledge through FTS. Report
   excluded or detached collections separately. Record the returned transfer ID and receipt digest;
   an optional vector issue cannot undo completed canonical/FTS restoration.
5. When `vector_rebuild.state` is `question-required`, ask whether to regenerate now. Record the
   answer with `hive knowledge transfer vector --user-root <root> --id <transfer-id>
   --receipt-digest <digest> --answer yes|no|cancel --output json`.
   `no` persists `deferred` for this transfer only; `cancel` stores no refusal. Neither changes
   vector preferences. Do not repeat a deferred question on a retry.
6. `yes` immediately rebuilds already-approved eligible partitions. Inspect each scope result.
   For an unfinished build, use `transfer status --user-root <root> --id <id> --output json` and
   repeat `transfer vector --answer yes` with its fresh receipt digest without asking again.
   Reuse completed indexes. Missing Python/runtime consent, new scope approval, or private-project
   attachment requires its existing separate setup/authorization procedure; never install or enable
   implicitly. Confidential content remains excluded. Preserve completed transfer status on failure.

Treat imported project-private collections as detached until explicit destination-project attachment.
Do not use a cloud account, move a bundle automatically, or infer approval for confidential data.


## Merge multiple bundles

When the user supplies two or more existing `.hivekb` files for one destination, use one
merge operation instead of repeated import. Never modify the input bundles.

1. Run `hive knowledge transfer merge preview --bundle <a.hivekb> --bundle <b.hivekb>
   --user-root <root> --output json`. Keep `merge_preview_digest`, `merge_input_digest`,
   every input archive digest, conflict path, and semantic candidate.
2. Exact duplicate bytes are safe to merge automatically. If semantic candidates are present, the
   active host reviews their kind, summary, body, conditions, numbers, dates, negation, and sources.
   Write a temporary review JSON bound to `merge_input_digest` with exactly one decision per
   semantic candidate: `separate` or `equivalent` plus one candidate path as `primary_path`.
   For a `conflicts` entry, use `choose` with its exact `path` and one listed `selected_sha256`.
3. Run `merge review` with `merge_input_digest` and the review JSON. The host may make this review
   without asking the user when the evidence proves equivalence. It must keep candidates separate
   when evidence is insufficient. A divergent canonical path or claim identity remains a user
   conflict; show all such conflicts together and do not apply a partial merge.
4. Run `merge apply` with the same input bundles, `merge_input_digest`, review JSON, and returned
   review digest. It rechecks inputs and destination state, applies one canonical transaction,
   rebuilds FTS once, and retains collapsed originals as portable merge provenance.
5. After a completed merge, use the existing transfer and vector status flow. Ask the optional
   vector rebuild question once for the whole merge only when the saved preference is `yes`.

The review JSON is input data, never a provider credential or a source of authority beyond the
current merge digest. Do not merge across collection visibility boundaries, install a model, enable
a new collection, or include confidential knowledge.
