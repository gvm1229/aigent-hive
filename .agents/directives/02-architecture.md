# 02. Architecture Directive

This directive protects Aigent Hive's product boundaries.

## Artifact Classes

Maintain three distinct artifact classes:

1. **Hive source workspace** — this repository's Rust source, templates, schemas, tests, directives, and documentation.
2. **Release bundle** — reproducible binaries plus versioned template/projection data and verification metadata.
3. **Installed harness** — files generated inside an independent consumer project.

Never import installed consumer state back into source, copy source-development directives into a consumer harness, or store user data in a plugin cache or release directory.

## Runtime Boundary

- Hive runs locally on top of an already authenticated subscription host.
- Hive must not call Anthropic, OpenAI, Google, or other model APIs.
- Hive must not request, read, store, or forward provider API keys.
- The active host or external orchestration layer owns model calls, subagent processes, session continuation, and retries.
- Hive owns setup, deterministic projection, Markdown data contracts, SQLite indexing, validation, migration, and update safety.

## Orchestration Boundary

- Do not implement Hive equivalents of plan, Ralph, team, swarm, or provider session runtimes.
- Use host-native capabilities when available.
- If the user selects OMX or OMC, preserve their namespaces and let that layer own orchestration.
- If a required capability is unavailable, report it as unsupported. Do not silently emulate it or switch backends.

## Canonical Data

- Knowledge, role identity, run state, plans, status, and evidence manifests are canonical Markdown.
- Setup answers, typed configuration, approval ledgers, and suppression fingerprints are canonical tracked YAML/TOML.
- Raw source objects may retain their original non-Markdown format when small, non-confidential, and Git-suitable.
- SQLite is a derived local index and may be deleted at any time.
- No durable fact may exist only in SQLite.
- A clean checkout containing the canonical tracked Markdown/YAML/TOML and source objects must be sufficient to rebuild the index without a model call or network request.

## Source Layout

- Root `.agents/` governs Hive development and is never a shipping source.
- `harness/` is the only canonical source for consumer templates, portable skills, and host projections.
- Host-specific files must be thin projections from provider-neutral contracts.
- Do not create empty crates or adapters that imply unsupported functionality.
