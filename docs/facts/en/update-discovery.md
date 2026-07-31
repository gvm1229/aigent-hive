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
  - "repo:crates/hive-cli/src/update_discovery.rs#sha256:9841e0c913da22987396f488e22bc0459062aa41f3508d25269ef55a277c6c29"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:4e3fb80b77c2e105029c9d6794922c4ce1b2fdeb"
status: active
---

# Update Discovery

Daily update discovery is explicit opt-in. A successful check throttles the next registry
request for 24 hours; offline or malformed results write no success time and retry in the next
host session. Discovery reports availability but never installs.
