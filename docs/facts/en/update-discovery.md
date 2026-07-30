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
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:2fb97b133d567155c0f333cbe7a401fc7473e849d88db2e2f9b897d7acecb39e"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:99f39edd08cc4b9d513f073d297bed05e2772c9d"
status: active
---

# Update Discovery

Daily update discovery is explicit opt-in. A successful check throttles the next registry
request for 24 hours; offline or malformed results write no success time and retry in the next
host session. Discovery reports availability but never installs.
