---
schema_version: 1
pair_id: steering-preservation
topic_slug: steering-preservation
language: en
counterpart: ../ko/steering-preservation.md
title: "Follow-up instruction preservation"
summary: "Hive 0.12.1 projects additive steering and conflict clarification rules."
tags: [agent, directives]
aliases: []
sources:
  - "repo:crates/hive-cli/src/user_directives.rs#sha256:e3f955e48e354aaac598a5237679689fb7c827366574d9330bc6dd4b661735ea"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7b8afc5527b300e665054ccd0f1fc6f34df02fee75e5e87f7198100cfd55caa9"
  - "repo:harness/directives/00-project-harness.md#sha256:ee4f9721824c4caad76c21b7d43e04f708637f3df96a095bf8058ef46b122ace"
links: [agent-autonomous-continuation]
reviewed_revision: "git:29e5241e0983e13d222703731533b43409327e6a"
status: active
---

# Follow-up instruction preservation

The maintainer requested preservation of earlier steering. Hive 0.12.1 adds the rule to project directives and English/Korean global guidance. New messages add by default; explicit replacement changes only the named scope. An implicit conflict requires a choice before the affected action, while independent work continues. Cancellation is immediate. Existing summaries retain unresolved conditions. Windows tests cover delivery to three host projections with Wiki disabled, not live model adherence or universal tool interception.
