---
schema_version: 1
pair_id: optional-collaborator-hive
topic_slug: optional-collaborator-hive
language: en
counterpart: ../ko/optional-collaborator-hive.md
title: "Optional Hive for Collaborators"
summary: "0.11.0 includes generator changes and verification before example-project repair planning."
tags: [collaboration, directives, version]
aliases: []
sources:
  - "repo:docs/plans/active/project-directives-collaboration-0.11.0.md#sha256:6b3e9a93fde9f1017e3bf185f6e248a4a292a2e6c19406b5f007ab5753b75985"
links: [foundation-refactor]
reviewed_revision: "git:a4414f0b4daf1cc0dabff770f7a3410a0be254e0"
status: active
---

# Optional Hive for Collaborators

The maintainer added PDC-001–003 to 0.11.0: separate common rules from Hive procedures, verify absent/working/failing installations, and preserve user and third-party bytes during upgrades. Keep ordinary development available without Hive; never treat protection failures as absence. RFR-001 and 0.11.0-test.2 public acceptance are complete. The example repair plan is prepared after verification; actual application remains prohibited. Eight Windows native cases and three-OS public checks passed within their documented limits.
