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
  - "repo:README.md#sha256:ac7b9cc92c876e73c7731f685482a15ef8ba9bc4a1ec9c1ff081e8dc2d14e089"
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
