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
  - "repo:README.md#sha256:a8326c3d7cf53451e09dcca9bc54f34b00b0428cf3d606e8e0c40ff3adf7b845"
links: [test-distribution, update-discovery, update-transaction]
reviewed_revision: "git:1144b25b9653cbb3e2a39bc6716acd13239f4ac7"
status: active
---

# Interactive Binary Update

Bare `hive update` requires an interactive terminal and uses the selected interface language.
It resolves npm `latest`, authenticates the running npm package manifest or direct receipt,
previews the exact owner adapter, installs only after explicit acceptance, and revalidates the
activated owner and version. A legacy `0.8.0-test.N` owner remains valid and sorts below exact
stable `0.8.0`, allowing an explicit update without losing ownership evidence. Decline, EOF,
and noninteractive calls make no installation change.
