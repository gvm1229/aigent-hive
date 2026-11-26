# Already supplied project choices

Use the same explicit target and user root. Without `--capabilities`, the CLI resolves host
capabilities exactly as terminal setup does: the project's saved capability data when the primary
host is unchanged, otherwise local host detection that leaves execution support unverified. Pass
`--capabilities` only with a trusted supplied file. Never invent host support, create a permission
digest from guesses, or silently replace a pinned owner.

A small local transport contains only known user choices:

Copy `answer_patch_template` from the initial status response. Keep its `schema_version` and
`kind` unchanged and fill only `answers`. Do not invent a format name or use a different wrapper.

```json
{"schema_version":1,"kind":"hive.setup-answer-patch","answers":{"project_name":"user-provided-name"}}
```

The CLI merges stored choices and preserves untouched values and provenance. Metadata and scope
bindings are not patchable answers. A fresh project requires `setup_mode`, `project_kind`,
`project_name` and `primary_host`; take them from the user's confirmation or delegation of the
proposal built from your interview or scan and the status `agent_setup` values. Custom overrides
follow the existing schema and mode rules. Never invent a role, optional Skill, hook, knowledge
scope or higher threshold.

```text
hive setup --target <project-root> --user-root <user-root> --answers <patch-file> --input-channel agent --apply --summary --output json
```

Complete supplied answer files remain supported. If new consent is required, use the full
`--dry-run` preview without `--summary`; summaries never replace an approval scope. Preserve role
bodies and existing assignments. After an AI error inspect state and use the exact resume command,
not a new apply. A successful configuration is separate from actual host execution qualification.
