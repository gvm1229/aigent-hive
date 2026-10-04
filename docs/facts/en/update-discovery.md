---
schema_version: 1
pair_id: update-discovery
topic_slug: update-discovery
language: en
counterpart: ../ko/update-discovery.md
title: "Update Discovery"
summary: "Opt-in update discovery checks once per successful day without installing."
tags: [discovery, update]
aliases: ["Daily update check"]
sources:
  - "repo:crates/hive-cli/src/update_discovery.rs#sha256:ba624251e8c106ec72cb0de4eb1ca4b1ecb7f67a70bb4486cc8eef35f01862e3"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:bd6d9249b8641590269d32deb97d13b2816ba75e"
status: active
---

# Update Discovery

Daily update discovery is explicit opt-in. A successful check throttles the next registry
request for 24 hours; offline or malformed results write no success time and retry in the next
host session. Discovery reports availability but never installs.
