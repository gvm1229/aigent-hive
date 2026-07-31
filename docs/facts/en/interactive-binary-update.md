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
  - "repo:README.md#sha256:4367530f3f660e786c271b1e9f3d26768b3bf5fa4f5f607becdc46e1d490aca8"
links: [test-distribution, update-discovery, update-transaction]
reviewed_revision: "git:bd6d9249b8641590269d32deb97d13b2816ba75e"
status: active
---

# Interactive Binary Update

Bare `hive update` requires an interactive terminal and uses the selected interface language.
It resolves the npm `test` package, authenticates the running npm package manifest or direct
receipt, previews the exact owner adapter, installs only after explicit acceptance, and
revalidates the activated owner and version. Decline, EOF, and noninteractive calls make no
installation change.
