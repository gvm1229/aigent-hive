---
schema_version: 1
pair_id: test-lane-inventory
topic_slug: test-lane-inventory
language: en
counterpart: ../ko/test-lane-inventory.md
title: "Test Lane Inventory"
summary: "Purpose-based test packages preserve every stability regression under one executable lane inventory."
tags: [release, test, verification]
aliases: ["conformance lanes", "test inventory"]
sources:
  - "repo:docs/guides/test-cleanup.md#sha256:b53790d8dba15644bdb0086d405b4b05412edc7b1d2c99db0d8d66edc9b049ba"
  - "repo:docs/guides/test-lanes.md#sha256:ac5e2863835c6c3605986ff71600d6b6a626676dc745a781cc2d0e18a28fa451"
  - "repo:scripts/test-lanes.py#sha256:5bc7694c5e1f399880069d16edbde37b85c741dadc5d6252892ebd5142cea8b1"
  - "repo:scripts/test_artifacts.py#sha256:a117cd65552ffd38cf02a52a19e8a154067cbdfff24ac91697f5fa6b54581157"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:df8aa9994a9fa02a4ee782567f646f664d7414ca244aa679e49498a7832b041f"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:81d38458c1fb4e2b0ad406bac350d06b5df34b57de31d729509f726402e9b319"
  - "repo:tests/conformance/lanes.toml#sha256:28e9d1ab7c0edb9325c4f923708982f21be0f963395f8cc27ca67df38abd065a"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:a7766827a25ced9deec765503bc5c46290dbdf8d"
status: active
---

# Test Lane Inventory

Purpose-based packages replace phase directories. One `tests/conformance/lanes.toml` inventory assigns every recursive `test_*.py` module: documentation, security, contract, integration, release. The runner rejects missing/duplicate modules, selects changed lanes and records JSON module timings. Stability and historical upgrade tests/fixtures remain intact.

The user's storage request adds `daily`: reviewed paths with committed evidence only; unresolved reviews, expiry and nonoverlapping storage above 20 GiB require attention. Child tests default to `CARGO_INCREMENTAL=0`, preserving overrides. Live processes, changed inventories and links block deletion. Small Markdown results remain in Git.
