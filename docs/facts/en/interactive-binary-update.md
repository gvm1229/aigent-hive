---
schema_version: 1
pair_id: interactive-binary-update
topic_slug: interactive-binary-update
language: en
counterpart: ../ko/interactive-binary-update.md
title: "Interactive Binary Update"
summary: "Bare hive update delegates an exact confirmed package to the authenticated current install owner."
tags: [installation, update]
aliases: ["Install-owner update"]
sources:
  - "repo:README.md#sha256:dcc7bb5f7428cf45304a94596cbf734c738b8b7aa300980df821829b652eeb0f"
links: [test-distribution, update-discovery, update-transaction]
reviewed_revision: "git:19f6326521d7f963fca8e41aaf716c5fc987e975"
status: active
---

# Interactive Binary Update

Bare `hive update` requires an interactive terminal and uses the selected interface language.
It resolves npm `latest`, authenticates the running npm package manifest or direct receipt,
previews the exact owner adapter, installs only after explicit acceptance, and revalidates the
activated owner and version. A legacy `0.8.0-test.N` owner remains valid and sorts below exact
stable `0.8.0`, allowing an explicit update without losing ownership evidence. Decline, EOF,
and noninteractive calls make no installation change.
