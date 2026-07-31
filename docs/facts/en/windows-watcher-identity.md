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
  - "repo:tests/conformance/test_source_usage_guard.py#sha256:e6d29a38db42c18e49763da9013171caf990bad1c054590ada8a939a699f36bf"
links: [source-usage-guard]
reviewed_revision: "git:a1fb6e848117b83354144540df01474e68d25aa8"
status: active
---

# Windows Source Watcher Identity

On Windows, the source usage watcher starts through the base CPython executable
instead of a virtual-environment launcher. The recorded process PID therefore matches
the lease owner, startup waits for the lease with a bounded retry, and a failed startup
reaps its process. Concurrent atomic state replacement also retries only transient
Windows permission conflicts within the same bounded window.
