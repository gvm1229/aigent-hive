---
schema_version: 1
pair_id: usage-threshold-provenance
topic_slug: usage-threshold-provenance
language: en
counterpart: ../ko/usage-threshold-provenance.md
title: "Threshold declaration provenance"
summary: "0.14.0 changes the threshold without reconciling its declaration record; 0.14.1 reconciles it before publication."
tags: [preferences, setup, usage]
aliases: []
sources:
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:800de1795d32bd05d22a67f768913f40a0717782d5ba5bcee039245282a71ce2"
  - "repo:crates/hive-core/src/preferences.rs#sha256:838005e60b3255422c2ee143d76e66ef3ef877d7f3a8ff196eb446c0dfdc888d"
  - "repo:docs/research/usage-threshold-provenance-0.14.0/reproduce.py#sha256:5460ffb967d9381497534988fd6b20f2f89b9faff8c6fa5f2f220df8fd5128c5"
links: [usage-guard-thresholds]
reviewed_revision: "git:dd581c30013516a6649e74e3c0d80bf7d46cb278"
status: active
---

# Threshold declaration provenance

In 0.14.0, the dedicated threshold setter changes the value but leaves its valid
decision digest unchanged. Reload reconciles that record, so full comparison fails
and activation rolls back. Synthetic Windows cases reproduce this without pending
setup, for explicit and legacy-unknown records. General answer patches update the
record; clearing pending setup alone is not a product fix. The requested diagnosis
and additive-plan review keep this defect separate:
[evidence](../../research/usage-threshold-provenance-0.14.0/README.md). 0.14.1 applies the
same reconciliation to the candidate before publication ([fix](../../research/overhaul/OV141-002.md)).
