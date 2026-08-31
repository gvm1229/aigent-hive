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
  - "repo:docs/guides/test-lanes.md#sha256:f411a47fa291833172ecf56219e0446b806b84c49206ceed926279bf27d17141"
  - "repo:scripts/test-lanes.py#sha256:5bc7694c5e1f399880069d16edbde37b85c741dadc5d6252892ebd5142cea8b1"
  - "repo:scripts/test_artifacts.py#sha256:d5aa3c82a7d7aaf76eee072ac675ed8d04990aed61451e329bfdf798f9e88785"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:42a2bacfb28c1ee12a73765d027cfb74b7ad786b659d12dbffc098337e09d62c"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:f74ae9ecf4d442e4171b4f0b28bb4d2a7ad75167858d8cba436e9710021e12ab"
  - "repo:tests/conformance/lanes.toml#sha256:2fa749a5fa9fec1ddff3f4e547f317235d6e9bdf4df355cbe9e4ffacf142e160"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:571467bb776b86bed509a06cdb6744434b067993"
status: active
---

# Test Lane Inventory

Python tests and fixtures use purpose-based packages instead of phase directories.
`tests/conformance/lanes.toml` assigns every recursive `test_*.py` module once to documentation,
security, contract, integration, or release. The runner rejects omissions and duplicates, selects
lanes from changed paths, and can write module timing JSON. Test commands record durable Markdown
evidence before their generated output can become eligible for cleanup. Stability and historical
upgrade tests remain; no test or fixture was deleted during the reorganization.
