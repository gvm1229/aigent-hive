---
schema_version: 1
pair_id: historical-project-base-coverage
topic_slug: historical-project-base-coverage
language: en
counterpart: ../ko/historical-project-base-coverage.md
title: "Historical Project Base Coverage"
summary: "A declared project upgrade source range requires exact authenticable full bases and matrix acceptance."
tags: [migration, project-upgrade, regression, release]
aliases: ["Historical base parity"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:162954ace665a9f30166cf241abe18b5e1168ebd8e862c106819a142d496bd46"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:c5dba7810327a88235025ea62ba2b77387a072c8e76b044b661ddb911aa26220"
  - "repo:crates/hive-render/src/lib.rs#sha256:71a3eba58eab1195bc5f6dc5411d81fefd547ab73ef09070479edc0bbe67b091"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:0fd5ea87fa377dc584dcfa6ad93ae9ee74eb4e97"
status: active
---

# Historical Project Base Coverage

For the future `0.9.5` candidate, the complete project-base source set is `0.9.1` through `0.9.4`.
The coverage checker derives a digest-bound report and rejects a same-major source range below `0.9.1`.
The compiled CLI and signed release-update matrices pass for `0.9.1` through `0.9.4`. Their full
base projection is authenticated before mutation; missing or tampered bases stop before apply and
preserve project and foreign bytes.
