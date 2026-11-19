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
  - "repo:README.md#sha256:e3b2bf7f1fa08e168811133d0e9992ff9e7477cb9a31cf24658a29a403a9b2dc"
  - "repo:docs/archive/plans/releases/0.9.5/knowledge-bundle-portability-0.9.5.md#sha256:78721fbbaf589353a17fdee534e5c86f1406283cf546eb32acd9996e84adb3c3"
  - "repo:docs/hive-install-guide.ko.html#sha256:dd0758ce13884e75e4caa160fde742f8af781dc18616f28c519d2e825dc8fa74"
links: [knowledge-portability-scan, knowledge-storage]
reviewed_revision: "git:c5b13bc26c4b2485dcad7509bc8590f97d0831fe"
status: active
---

# Global Knowledge Bundle Transfer

`--user-root` is the user home directory, not `.hive`. Use the macOS/Linux or Windows
shell example that matches the host. Verify SHA-256 and a conflict-free `--dry-run` before
`--apply`. Bundles contain portable Markdown only: no SQLite index, runtime state,
project-private knowledge, credentials, or absolute paths.
