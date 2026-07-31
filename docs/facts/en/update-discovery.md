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
  - "repo:crates/hive-cli/src/update_discovery.rs#sha256:650d13734e29745a8ae4634c0dd7f3f8477222e144875b111ce6925fccd86a19"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:bf9e28d8af36ef8d672694fc3c23fdd1a39233ee"
status: active
---

# Update Discovery

Daily update discovery is explicit opt-in. A successful check throttles the next registry
request for 24 hours; offline or malformed results write no success time and retry in the next
host session. Discovery reports availability but never installs.
