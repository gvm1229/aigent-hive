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
  - "repo:docs/guides/test-lanes.md#sha256:2d9ea96838ebef0f85ad3bdc2549163fe8b01fcfb5d206ce2bef4a7be7763ee6"
  - "repo:scripts/test-lanes.py#sha256:08d6ee2113e301f836a217733539f9e01b96f4c6569f4f71c4e02635fab0bfa8"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:096b1b193472079e7f44e94aa0381dd399046120fae3a103772993e45f2a7fd7"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:316c4057978fb4b928618c41fb37fb596f9d8b8d9e6e4f08fe85cdfa8756ada0"
  - "repo:tests/conformance/lanes.toml#sha256:e489bbf237207fd643f36a4e95324c977de54368f87cc74b03646ee19549f693"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:f752a59f5bec8d07071789ef1ef109aa8381d7fa"
status: active
---

# Test Lane Inventory

Python tests and fixtures use purpose-based packages instead of phase directories.
`tests/conformance/lanes.toml` assigns every recursive `test_*.py` module once to documentation,
security, contract, integration, or release. The runner rejects omissions and duplicates, selects
lanes from changed paths, and can write module timing JSON. Stability and historical upgrade tests
remain; no test or fixture was deleted during the reorganization.
