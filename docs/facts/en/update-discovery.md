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
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:7631f6a1b322510cf6b9b1d6e826681362cb494d250cf7137b4a29c446402b35"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:4e3fb80b77c2e105029c9d6794922c4ce1b2fdeb"
status: active
---

# Update Discovery

Daily update discovery is explicit opt-in. A successful check throttles the next registry
request for 24 hours; offline or malformed results write no success time and retry in the next
host session. Discovery reports availability but never installs.
