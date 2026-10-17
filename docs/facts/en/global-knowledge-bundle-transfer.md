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
  - "repo:README.md#sha256:165c9867d908d6b3774380a621952156e343989ad88f24e942663f552ba85077"
  - "repo:docs/archive/plans/releases/0.9.5/knowledge-bundle-portability-0.9.5.md#sha256:78721fbbaf589353a17fdee534e5c86f1406283cf546eb32acd9996e84adb3c3"
  - "repo:docs/hive-install-guide.ko.html#sha256:dfec0894dac0e14eab09ee5da9e7eb5a5c3febe418efd02b0614c711d5fd938d"
links: [knowledge-portability-scan, knowledge-storage]
reviewed_revision: "git:9c98d50fda7f2ad4b3011f5697cee950a1b2847f"
status: active
---

# Global Knowledge Bundle Transfer

`--user-root` is the user home directory, not `.hive`. Use the macOS/Linux or Windows
shell example that matches the host. Verify SHA-256 and a conflict-free `--dry-run` before
`--apply`. Bundles contain portable Markdown only: no SQLite index, runtime state,
project-private knowledge, credentials, or absolute paths.
