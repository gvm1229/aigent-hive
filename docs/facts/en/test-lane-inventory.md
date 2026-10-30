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
  - "repo:docs/guides/test-cleanup.md#sha256:12d06ecf62144cec1394e4cff8762ce8b7942211941077f1d232728182c0294f"
  - "repo:docs/guides/test-lanes.md#sha256:ac5e2863835c6c3605986ff71600d6b6a626676dc745a781cc2d0e18a28fa451"
  - "repo:scripts/test-lanes.py#sha256:5bc7694c5e1f399880069d16edbde37b85c741dadc5d6252892ebd5142cea8b1"
  - "repo:scripts/test_artifacts.py#sha256:779fd37acac29ad2165234120e7eb942b3c0d121b75a084658e30440f30f5bc7"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:aadd1bc3e37924cf0869cb520cbf4401583682c3b005d19a4f119d3b381b6524"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:43ff866b01e2ff98234a12225e2278b1fd18752fa78ae3c34b91d1afefc6685d"
  - "repo:tests/conformance/lanes.toml#sha256:bc515db1c2bbb782a72a4d1b5ab659967f471b829b85d8b193248717d8532dd8"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Test Lane Inventory

Python packages replace phases by purpose. `tests/conformance/lanes.toml` assigns every recursive `test_*.py` module to documentation, security, contract, integration or release. The runner rejects omissions/duplicates, selects changed lanes with JSON timings. Stability and historical upgrade tests/fixtures remain intact.

`daily` cleans reviewed paths with existing committed evidence. Unresolved reviews, expiry or nonoverlapping storage over 20GiB need attention. Tests default to `CARGO_INCREMENTAL=0`, preserving overrides. Live processes, changed inventories and links block deletion. The maintainer forbids cleanup-created `tests/` documents and result commits; audits use ignored `.agents/work/test-artifacts/cleanup/`. Existing test evidence stays in Git.
