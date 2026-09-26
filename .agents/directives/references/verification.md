# Verification

## Verification Tiers

Match verification cost to the current boundary:

1. **Work loop** — run the changed Rust crate tests and directly related Python tests only.
2. **Pre-commit** — run affected crates plus the nearest black-box, schema, static-contract,
   or regression tests for the changed behavior.
3. **Pre-push** — run the full Rust workspace and full Python conformance suite once for the
   logical milestone being pushed. Do not repeat an unchanged full-suite result for every
   commit in the same milestone.
4. **Release** — run clean-clone CI, every supported OS/architecture, hostile and security
   suites, installer/update recovery, signing, provenance, and publication qualification.

## Test Artifact Lifecycle

- Before a local or CI test that produces source `tests/work/` or `target/debug/` output, use
  `python scripts/test-artifacts.py run --purpose <Korean-summary> --path <owned-path> --command <test-command>`.
- Keep the resulting `tests/results/runs/*.md` record. A passing test is not a cleanup authority
  until its result record is reviewed and committed.
- Use `python scripts/test-artifacts.py check` at a task closure. Resolve every eligible or expired
  item through an explicit review, then use `cleanup --apply` only for the exact reviewed paths.
- A path with a live process, a concrete 72-hour reuse reservation, a failed reproduction, or
  incomplete evidence remains retained. Never use a glob, parent-directory deletion, or age alone.
