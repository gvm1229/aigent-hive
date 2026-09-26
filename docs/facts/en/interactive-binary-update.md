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
  - "repo:README.md#sha256:ef0e22aa026b267338f193ae7a9ebe5ff341609a9f10410f21d0dbb54fe7f853"
links: [test-distribution, update-discovery, update-transaction]
reviewed_revision: "git:1b755a995d91739d758830210d93cdc012e9e61b"
status: active
---

# Interactive Binary Update

Bare `hive update` requires an interactive terminal and uses the selected interface language.
It resolves npm `latest`, authenticates the running npm package manifest or direct receipt,
previews the exact owner adapter, installs only after explicit acceptance, and revalidates the
activated owner and version. A `0.9.0-test.N` owner remains valid for a later exact stable
`0.9.0` update without losing ownership evidence. Decline, EOF, and noninteractive calls make no
installation change.
