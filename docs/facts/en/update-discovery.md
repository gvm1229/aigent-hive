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
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:a958146a8ea6d747fa485cf5ba0ec81f0471567723589fe82a6aa90815cece06"
links: [global-onboarding, test-distribution]
reviewed_revision: "git:a7be86f2558442c2cec3596abe2f481dd91d268f"
status: active
---

# Update Discovery

Daily update discovery is explicit opt-in. A successful check throttles the next registry
request for 24 hours; offline or malformed results write no success time and retry in the next
host session. Discovery reports availability but never installs.
