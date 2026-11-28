# Code checklist

Read only when the target is code. Bundled detail: `upstream/omc-ai-slop-cleaner/SKILL.md`
(workflow and slop classes), with [hive-overrides.md](hive-overrides.md) applied.

## Baseline

Run the nearest regression check before editing and classify it as `runnable`, `unavailable`,
`pre-existing-failure` or `out-of-scope-expensive`. For a non-runnable baseline, record the
smallest truthful fallback: a focused test, compile or type check, lint, snapshot or bounded probe.

## Smells and examples

| Class | Look for |
| --- | --- |
| Repetition | copy-paste branches, duplicate helpers, the same validation twice |
| Padding | comments that restate the code, debug leftovers, dead branches, stale flags |
| Needless layers | pass-through wrappers, single-use helpers, speculative configuration |
| Generic defaults | broad `catch` that hides errors, defensive checks for impossible states |
| Unsupported claims | comments or names that promise behavior the code lacks |

## Passes

1. Dead code deletion.
2. Duplicate removal.
3. Naming and error-handling consistency.
4. Missing regression tests for preserved behavior.

After each pass, read that pass's diff and rerun its check. Keep public interfaces, data formats
and timing-sensitive behavior unchanged. Add no dependency.
