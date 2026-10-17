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
  - "repo:scripts/test_artifacts.py#sha256:832d67312bf54707ea0e3acfce37189cc733180819e41fdc9ca9c21bf04957d0"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:d6b780ea86a3b69a0b822f9b227a5a3e0004ca0d36f7da3e87d401b15b4e973a"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:81d38458c1fb4e2b0ad406bac350d06b5df34b57de31d729509f726402e9b319"
  - "repo:tests/conformance/lanes.toml#sha256:ebb52408e72acc7e3dffface4f5694e14749eb204c882eda821a5392408b111f"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:6cee0fc8998d4b1b25eaed092f1a252b7144b210"
status: active
---

# Test Lane Inventory

Purpose-based packages replace phase directories. One `tests/conformance/lanes.toml` inventory assigns every recursive `test_*.py` module: documentation, security, contract, integration, release. The runner rejects missing/duplicate modules, selects changed lanes and records JSON module timings. Stability and historical upgrade tests/fixtures remain intact.

The user's storage request adds `daily`: reviewed paths with committed evidence only; unresolved reviews, expiry and nonoverlapping storage above 20 GiB require attention. Child tests default to `CARGO_INCREMENTAL=0`, preserving overrides. Live processes, changed inventories and links block deletion. Small Markdown results remain in Git.
