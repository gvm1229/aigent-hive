---
schema_version: 1
pair_id: global-knowledge-bundle-transfer
topic_slug: global-knowledge-bundle-transfer
language: en
counterpart: ../ko/global-knowledge-bundle-transfer.md
title: "Global Knowledge Bundle Transfer"
summary: "A global .hivekb transfer uses the current shell home, SHA-256 verification, dry-run, then explicit apply."
tags: [bundle, global, knowledge, portability]
aliases: [".hivekb transfer", "knowledge export import"]
sources:
  - "repo:README.md#sha256:ac7b9cc92c876e73c7731f685482a15ef8ba9bc4a1ec9c1ff081e8dc2d14e089"
  - "repo:docs/archive/plans/releases/0.9.5/knowledge-bundle-portability-0.9.5.md#sha256:78721fbbaf589353a17fdee534e5c86f1406283cf546eb32acd9996e84adb3c3"
  - "repo:docs/hive-install-guide.ko.html#sha256:fb6dbc2e5752fe9693c7ac791841b2e2f9995ef03a2d5c3ef4ce07b6f068800c"
links: [knowledge-portability-scan, knowledge-storage]
reviewed_revision: "git:1b755a995d91739d758830210d93cdc012e9e61b"
status: active
---

# Global Knowledge Bundle Transfer

`--user-root` is the user home directory, not `.hive`. Use the macOS/Linux or Windows
shell example that matches the host. Verify SHA-256 and a conflict-free `--dry-run` before
`--apply`. Bundles contain portable Markdown only: no SQLite index, runtime state,
project-private knowledge, credentials, or absolute paths.
