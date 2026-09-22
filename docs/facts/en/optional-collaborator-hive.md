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
  - "repo:docs/plans/active/project-directives-collaboration-0.11.0.md#sha256:910ba889f836e4a7e90623b81d09e7293222c02d021cd5fd2b03d1671bba3696"
links: [foundation-refactor]
reviewed_revision: "git:dd4e2773e98d7acb13fa0dc77ad11167faaf23fb"
status: active
---

# Optional Hive for Collaborators

The maintainer added PDC-001–003 to 0.11.0: separate common rules from Hive procedures, verify absent/working/failing installations, and preserve user and third-party bytes during upgrades. Keep ordinary development available without Hive; never treat protection failures as absence. Reopen RFR-001 for 0.11.0-test.2. Keep the example read-only; after product verification, prepare its repair plan with the fixed Hive version. Scope documentation is complete; implementation and behavioral acceptance remain unverified.
