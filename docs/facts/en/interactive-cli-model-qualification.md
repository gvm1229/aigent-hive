---
schema_version: 1
pair_id: interactive-cli-model-qualification
topic_slug: interactive-cli-model-qualification
language: en
counterpart: ../ko/interactive-cli-model-qualification.md
title: Interactive CLI Model Qualification
summary: Actual model measurements and persistent setup regression checks have separate
  evidence scopes.
tags:
- interactive-cli
- model-evaluation
- preservation
- regression
aliases: []
sources:
- repo:docs/research/cli-model-validation-0.14.0.md#sha256:13d1c983971e95ec5b8a7b927451a4272f6215c63ab86f9d8da3b2ed8652807e
- repo:tests/conformance/contracts/test_setup_summary.py#sha256:67090516ace0ed798c6f1388ddc7fc9ab35ce4815fdd801435f126aa7a96f401
- repo:tests/fixtures/interactive-cli/model-fixed-choices.json#sha256:89b5913b061ecb20908a1ea47c0fe7cd61206c3f49c6fb7e4077735f52cc8f38
- repo:tests/fixtures/interactive-cli/model-project-receipt.py#sha256:2a3c75ff8e8b01df54958cd5718def0ab6786e2b483ad057362eb7456370dee3
links:
- consumer-session-coordination
- project-onboarding
reviewed_revision: git:cd1a9dd8df64e42e6fcc8bdd3e24d4941987afa7
status: active
---

# Interactive CLI Model Qualification

Windows Codex fixtures for0.14.0 measured two runs per version/scope: Hive input-byte medians fell60.7038% globally and50.0094% for projects. Direct CLI setup uses no model. These are not token, price or throughput measurements; the project median clears its target by only3 bytes.

The maintainer requires persistent fixed-input CLI regressions for develop integration and at most five runs per distinct parameter set. Failures and pending reservations count. Temporary paths or documentation commits do not reset a case; actual model/input changes do. Aggregate trial totals are not approval limits.
