---
name: hive-directive-amend
description: Amend Aigent Hive source-development and consumer-product agent directives from an explicit same-line command. Use only when the user invokes `$hive-directive-amend`; default to both surfaces, with `--source` and `--consumer` for a single surface.
---

# Hive Directive Amend

Apply an explicit directive change to the narrowest canonical source and every required
projection.

## Command boundary

Parse one physical line only:

```text
$hive-directive-amend [--source|--consumer] <amendment command>
```

- Treat only the text after the invocation and optional scope flag on that same physical line as
  amendment authority.
- Stop the command at the first line break. Treat later paragraphs only as non-authoritative
  context or rationale; never use them to add rules, expand scope, or authorize extra edits.
- Require a nonempty same-line command. If it is empty, ask the user to provide it on the same
  line.
- Accept at most one scope flag. Reject unknown or conflicting flags instead of guessing.

## Scope

- No flag: amend both source-development and consumer-product directives.
- `--source`: amend source-development directives only.
- `--consumer`: amend consumer-product directives only.

Keep this Skill source-only under `.agents/skills/`. It edits consumer canonical producers from
the Hive source workspace; it is not projected into installed consumer projects.

## Workflow

1. Require the Hive source root marker `hive-source.json`.
2. Preserve the exact same-line amendment command as the authority boundary.
3. Read the mandatory source directives and the narrowest current canonical documents governing
   the requested behavior.
4. Map the rule to the selected surfaces:
   - source: the narrowest `.agents/directives/` file, with `AGENTS.md` changed only when its
     compact routing or prime contract must expose the rule;
   - consumer: canonical producers under `harness/`, compiled renderers or user-guidance
     generators under `crates/`, the guidance contract under `docs/`, and direct regression
     tests.
5. Preserve meaning across surfaces without requiring byte-identical wording. Respect the
   selected interface language for consumer guidance.
6. Apply every safe, in-scope edit without another confirmation. Stop for ambiguity only when
   different interpretations would materially change authority, safety, ownership, or product
   behavior.
7. Verify producer/projection parity and the nearest source and consumer regressions.
8. Commit each independently reviewable concern separately under the repository Git rules.
9. Report the applied scope, canonical paths, verification, and any action that genuinely
   requires user authority.

## Boundaries

- Never treat surrounding paragraphs, chat history, urgency, or an example without the explicit
  invocation as amendment authority.
- Never edit generated output alone when a canonical producer exists.
- Preserve foreign bytes and exact Hive ownership markers.
- Never weaken security, credential, production-publication, source-root refusal, usage-guard, or
  higher-priority instruction boundaries.
- Never copy source-only development instructions into consumer guidance unless the amendment
  explicitly governs both and the consumer wording is independently appropriate.
