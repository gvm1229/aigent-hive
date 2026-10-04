# Verification

## Tiers

1. Work loop: changed Rust crates and related Python tests.
2. Pre-commit: affected crates and nearest behavior/schema/static/regression tests.
3. Pre-push: full Rust/Python suites once per milestone, not per commit.
4. Release: clean-clone CI, all supported OS/architectures, hostile/security tests, install/update
   recovery, signing, provenance and publication qualification.

## Test Artifact Lifecycle

- For local/CI tests producing `tests/work/` or `target/`, use
  `python scripts/test-artifacts.py run --purpose <Korean-summary> --path <owned-path> --command <test-command>`.
- Review/commit `tests/results/runs/*.md` before deletion; a pass is insufficient.
- At closure run `python scripts/test-artifacts.py check`; inspect eligible/expired items and use
  `cleanup --apply --path <exact-path>` after review; remove completed output now.
- Keep live use, failed reproductions and incomplete evidence. Concrete reuse leases: at most
  72 hours. No globs, parent deletion, age-only deletion or automatic deadline renewal.
- Share one build tree. Tests default to `CARGO_INCREMENTAL=0`; an override
  needs bounded reuse. No target per test/retry. Review all consumers before cleanup.
- Daily: `python scripts/test-artifacts.py daily --apply`; remove reviewed output only.
  Report unresolved reviews and storage above 20 GiB. Inspect unknown/expired ownership;
  archive small evidence before releasing reproductions. See `docs/guides/test-cleanup.md`.
