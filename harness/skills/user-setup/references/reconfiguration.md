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
interface language and selected hosts must be supplied; do not invent them. Factory defaults are
appropriate only for a requested quick setup, not as inferred custom preferences.

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
