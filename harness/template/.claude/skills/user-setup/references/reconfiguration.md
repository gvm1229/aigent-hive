# Named changes and structured choices

Inspect the same user root once with `--show --summary --output json`. For terminal input, use
`hive setup --scope user --user-root <user-root> --section <catalog-section>` for the requested area.
The CLI owns the question text and preview. Do not ask an already answered choice again in chat.

When the user already supplied choices, a small local JSON/YAML transport can contain:

```json
{"schema_version":1,"kind":"hive.setup-answer-patch","answers":{"interface_language":"ko"}}
```

This is an example, not a default or authority to change language. Include only actual user choices.
The CLI merges them with stored settings and preserves other values and provenance. Never put
schema, provenance, root binding or operation metadata inside `answers`. For a fresh user root,
`interface_language` and `selected_hosts` must be supplied; do not invent them. Factory defaults are
appropriate only for a requested quick setup, not as inferred custom preferences.

If a requested setting's field name or shape is unknown, read the matching
`--describe --section <section> --output json` using the same user root. Select the section's
exact ID from `available_sections` in the initial status summary; never invent a section name.
Copy `question.key`
exactly into a nested patch object: a key `/a/b` means `{"a":{"b":value}}`. Use only the
requested sections, batch independent lookups, and never fetch the full schema or ask the user
to repeat supplied choices. Keep labels separate from stored field names.

Save that JSON to a regular local file. Do not pipe it into Hive or omit `--answers`.
Apply an already authorized ordinary change through the same engine:

```text
hive setup --scope user --user-root <user-root> --answers <patch-file> --input-channel agent --apply --summary --output json
```

User-supplied complete answer files remain supported. Agent input records its channel, not a direct
human confirmation. For an operation needing separate consent, obtain the full `--dry-run` preview
without `--summary` and use only its exact approved flags/digest. Do not infer downloads, index
rebuild scope, monitor activation, interruption, host removal or external delivery from this example.

After failure or an interrupted AI response, inspect and resume the stored operation. Do not rewrite
the same full YAML or retry apply blindly. A stored preference is separate from a verified runtime
state. Use the CLI's monitor review flow for consent withdrawal. Preserve Wiki Markdown and user
notes; program removal is a separate explicit request covered by recovery.md.

## Standalone vector feature compatibility

Use this route only when an installed instruction or an explicit request calls for the standalone
vector-search feature API. Normal CLI setup already owns its vector question; do not ask twice.
Run `hive setup feature claim --id vector-search --user-root <user-root> --output json`.
Ask only if `question_required` is true. Explain that exact search remains available and first
preparation takes time; the measured Windows runtime is about 376MB. If another session holds the
unanswered claim, wait until that claim expires; never store a host session identifier.
For an explicit yes, run `hive setup feature answer --id vector-search --answer yes --user-root
<user-root> --output json`, then `hive setup feature prompt --id vector-search --user-root
<user-root> --output json`. Give its `prompt` field for a new session. Preserve the fixed collection
list and `setup_request_digest`; changed scope needs a new preview. Download and index preparation
retain separate consent. For an explicit no, save `--answer no` and do not ask again unless requested.
Silence, cancellation and an interrupted response are never a no answer or activation consent.
