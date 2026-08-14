---
schema_version: 1
pair_id: test-lane-inventory
topic_slug: test-lane-inventory
language: en
counterpart: ../ko/test-lane-inventory.md
title: "Test Lane Inventory"
summary: "One executable manifest assigns every Python conformance module to one named release lane."
tags: [release, test, verification]
aliases: ["conformance lanes", "test inventory"]
sources:
  - "repo:scripts/test-lanes.py#sha256:a5733a0e84b484c06f89b7a5f55d09053de3153a38e444bbf0188ff4c319fa4d"
  - "repo:tests/conformance/lanes.toml#sha256:0938e7e898dec62f527b152b571ff989b3c5b09c03a3c188120243dab9dfb7fa"
  - "repo:tests/conformance/test_connected_setup_lifecycle.py#sha256:3209668a474ee06f54bb75cc383d34e64ad3c5fee15f2662b5a4163fff7e510e"
  - "repo:tests/conformance/test_phase4_contracts.py#sha256:931a18a69a2f065109133c25ad954e8214f4635ee2685f412343354a8f34e396"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:3b4d6d23c679eec9e23f334dc60a2678b657345e"
status: active
---

# Test Lane Inventory

`tests/conformance/lanes.toml` assigns every `test_*.py` module once to documentation, security,
contract, integration, or release. `scripts/test-lanes.py` rejects omissions and duplicates,
runs selected lanes, and records elapsed time. Consumer fixtures use ignored `tests/work/` roots;
Phase 4 no longer creates `tests/hive-phase4-*` beside tracked tests.
