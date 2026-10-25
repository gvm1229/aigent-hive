# Already supplied project choices

Use the same explicit target and user root. A trusted supplied capability file or the project's
saved capability material is required for the existing file input. Never invent host support,
create a permission digest from guesses, or silently replace a pinned owner. Without the needed
material, use the visible CLI terminal, or provide that one command to the user.

A small local transport contains only known user choices:

```json
{"schema_version":1,"kind":"hive.setup-answer-patch","answers":{"project_name":"user-provided-name"}}
```

The CLI merges stored choices and preserves untouched values and provenance. Metadata and scope
bindings are not patchable answers. A fresh project requires `setup_mode`, `project_kind`,
`project_name` and `primary_host`; do not infer them. Custom overrides follow the existing schema
and mode rules. Never invent a role, optional Skill, hook, knowledge scope or higher threshold.

```text
hive setup --target <project-root> --user-root <user-root> --answers <patch-file> --capabilities <trusted-capabilities-file> --input-channel agent --apply --summary --output json
```

Complete supplied answer files remain supported. If new consent is required, use the full
`--dry-run` preview without `--summary`; summaries never replace an approval scope. Preserve role
bodies and existing assignments. After an AI error inspect state and use the exact resume command,
not a new apply. A successful configuration is separate from actual host execution qualification.
