# Verification

## Tiers

1. Work loop: changed Rust crates and related Python tests.
2. Pre-commit: affected crates and nearest behavior/schema/static/regression tests.
3. Pre-push: full Rust/Python suites once per milestone, not per commit.
4. Release: clean-clone CI, all supported OS/architectures, hostile/security tests, install/update
   recovery, signing, provenance and publication qualification.

## Test Artifact Lifecycle

- For tests producing `tests/work/` or `target/debug/`, local or CI, use
  `python scripts/test-artifacts.py run --purpose <Korean-summary> --path <owned-path> --command <test-command>`.
- Review and commit `tests/results/runs/*.md` before deletion; passing alone is insufficient.
- At closure run `python scripts/test-artifacts.py check`; inspect eligible/expired items and use
  `cleanup --apply --path <exact-path>` after review. Remove completed output in the same task.
- Preserve live processes, failed reproductions, incomplete evidence and concrete reuse for at most
  72 hours. No globs, parent deletion, age-only deletion or automatic deadline renewal.
- Reuse one shared build tree. Child tests default to `CARGO_INCREMENTAL=0`; an explicit override
  needs bounded reuse. No separate Cargo target per test/retry. Review all consumers before cleanup.
- Daily: `python scripts/test-artifacts.py daily --apply`; only reviewed output is removed.
  Report unresolved reviews and storage above 20 GiB. Inspect unknown/expired ownership;
  archive small evidence before releasing reproductions. See `docs/guides/test-cleanup.md`.
