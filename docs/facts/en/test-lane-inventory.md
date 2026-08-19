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
  - "repo:scripts/test-lanes.py#sha256:08d6ee2113e301f836a217733539f9e01b96f4c6569f4f71c4e02635fab0bfa8"
  - "repo:tests/conformance/contracts/test_run_role_contracts.py#sha256:5da00009c7146f04088445cfeda29641f7edc22757f9a9c9e37b5ea0a612bcf0"
  - "repo:tests/conformance/integration/test_connected_setup_lifecycle.py#sha256:316c4057978fb4b928618c41fb37fb596f9d8b8d9e6e4f08fe85cdfa8756ada0"
  - "repo:tests/conformance/lanes.toml#sha256:e489bbf237207fd643f36a4e95324c977de54368f87cc74b03646ee19549f693"
links: [release-verification, test-fault-isolation]
reviewed_revision: "git:3b4d6d23c679eec9e23f334dc60a2678b657345e"
status: active
---

# Test Lane Inventory

`tests/conformance/lanes.toml` assigns every `test_*.py` module once to documentation, security,
contract, integration, or release. `scripts/test-lanes.py` rejects omissions and duplicates,
runs selected lanes, and records elapsed time. Consumer fixtures use ignored `tests/work/` roots;
Phase 4 no longer creates `tests/hive-phase4-*` beside tracked tests.
