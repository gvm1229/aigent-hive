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
  - "repo:docs/guides/test-cleanup.md#sha256:ff6a2ce93c809bf773968f020f432235b43e0ee07a0251a78708839fe3332272"
  - "repo:docs/guides/test-lanes.md#sha256:ac5e2863835c6c3605986ff71600d6b6a626676dc745a781cc2d0e18a28fa451"
  - "repo:scripts/test-lanes.py#sha256:5bc7694c5e1f399880069d16edbde37b85c741dadc5d6252892ebd5142cea8b1"
  - "repo:scripts/test_artifacts.py#sha256:df00889b7e966f0b5613ee9f079e35dc21c2ba52bc55084f311c042e4af50a6d"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:aadd1bc3e37924cf0869cb520cbf4401583682c3b005d19a4f119d3b381b6524"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:43ff866b01e2ff98234a12225e2278b1fd18752fa78ae3c34b91d1afefc6685d"
  - "repo:tests/conformance/lanes.toml#sha256:7a19fd3fe4dcf636dc7691665e3072e61c8354320141b13a7eb85e2c5fd85739"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Test Lane Inventory

Purpose packages replace phases. `tests/conformance/lanes.toml` assigns recursive `test_*.py` to documentation/security/contract/integration/release. Missing/duplicates fail; selects changed lanes; logs JSON timings. Stability/upgrade tests and fixtures stay.

`daily` requires reviewed paths and Git evidence. Unknown/expired records or unique storage >20GiB need attention. Tests default `CARGO_INCREMENTAL=0`; overrides stay. Processes, links and changed files block deletion. Inventory v2 excludes directory size, checks names/types/times/identity/file sizes; old formats need review. Review deps/build/incremental separately to keep frozen files in place. Cleanup creates no tests/ documents or commits; audits use `.agents/work/test-artifacts/cleanup/`.
