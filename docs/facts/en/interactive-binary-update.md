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
  - "repo:README.md#sha256:165c9867d908d6b3774380a621952156e343989ad88f24e942663f552ba85077"
links: [test-distribution, update-discovery, update-transaction]
reviewed_revision: "git:9c98d50fda7f2ad4b3011f5697cee950a1b2847f"
status: active
---

# Interactive Binary Update

Bare `hive update` requires an interactive terminal and uses the selected interface language.
It resolves npm `latest`, authenticates the running npm package manifest or direct receipt,
previews the exact owner adapter, installs only after explicit acceptance, and revalidates the
activated owner and version. A `0.9.0-test.N` owner remains valid for a later exact stable
`0.9.0` update without losing ownership evidence. Decline, EOF, and noninteractive calls make no
installation change.
