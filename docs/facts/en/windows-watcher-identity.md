---
schema_version: 1
pair_id: windows-watcher-identity
topic_slug: windows-watcher-identity
language: en
counterpart: ../ko/windows-watcher-identity.md
title: "Windows Source Watcher Identity"
summary: "Windows source watchers bypass virtual-environment launchers so PID ownership matches the lease."
tags: [guard, source, windows]
aliases: ["Windows watcher PID"]
sources:
  - "repo:tests/conformance/test_source_usage_guard.py#sha256:6b3a36c7bf6d41839d463f8239ef701221fc49c6303f5c849d2e589e8eadca7d"
links: [source-usage-guard]
reviewed_revision: "git:2c3485e6442d06871c9d61aec6c896c8fc93db11"
status: active
---

# Windows Source Watcher Identity

On Windows, the source usage watcher starts through the base CPython executable
instead of a virtual-environment launcher. The recorded process PID therefore matches
the lease owner, startup waits for the lease with a bounded retry, and a failed startup
reaps its process. Concurrent atomic state replacement also retries only transient
Windows permission conflicts within the same bounded window.
