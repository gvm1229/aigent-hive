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
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:c77febdf50b689937897ea1848ae0f38468d14843dbeda5486678eb523447902"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:f74ae9ecf4d442e4171b4f0b28bb4d2a7ad75167858d8cba436e9710021e12ab"
  - "repo:tests/conformance/lanes.toml#sha256:bb7f8713a1b1c63428cdde197f68028868126cd8436fa9cde0afc1b155fa192e"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# Test Lane Inventory

Python tests and fixtures use purpose-based packages instead of phase directories.
`tests/conformance/lanes.toml` assigns every recursive `test_*.py` module once to documentation,
security, contract, integration, or release. The runner rejects omissions and duplicates, selects
lanes from changed paths, and can write module timing JSON. Stability and historical upgrade tests
remain; no test or fixture was deleted during the reorganization.
