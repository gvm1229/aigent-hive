---
schema_version: 1
pair_id: usage-hosts
topic_slug: usage-hosts
language: en
counterpart: ../ko/usage-hosts.md
title: "Usage Guard and Host Sensors"
summary: "Native-first quota sensing, CodexBar fallback boundaries, and source-session enforcement."
tags: [guard, hosts, usage]
aliases: ["usage guard hosts"]
sources:
  - "repo:.github/workflows/ci.yml#sha256:60945a807302e95f64373efc5d91ff11269e0f2609dc1d21c707c41e5a79db09"
  - "repo:docs/decisions/ADR-0010-native-first-usage-sensors.md#sha256:141e8070b475ee2b0d81e93a69217093e07af9a9ca61c16dcbb31f111ea1d0f4"
  - "repo:docs/plans/active/native-usage-sensor.md#sha256:d2452d3622f65125e8e84261c33c4ff87aa823bb232a65aa6e22baffffd327d8"
  - "repo:tests/conformance/test_source_usage_guard.py#sha256:df19f5b1000469f30711a8624e8359761a85e52072eee69f5b3be73e2952ba3b"
links: [plugin-lifecycle, security-release, workflow]
reviewed_revision: "git:51f40e24316e9f776626ddf73676f7719b020a42"
status: active
---

# Usage Guard and Host Sensors

The usage guard checks the configured inclusive remaining-quota threshold before automatic work
boundaries. It records only sanitized sensor identity, window, timing, and decision data. Account
payloads, provider credentials, and raw quota responses are outside durable state.

Sensor order is native first, then CodexBar only as a fallback. Codex uses the local app-server
rate-limit method. Claude can use an explicitly configured, sanitized status-line capture.
Antigravity has no qualified official structured output yet, so its native sensor remains
unsupported. CodexBar is fallback-only for all three providers and requires explicit consent before
installation.

The source-development Python watcher and boundary gate are separate from the shipped one-shot
dispatch guard. A transient `unknown` waits three seconds and retries once; a repeated short glitch
remains visible but creates no new halt marker. Confirmed quota exhaustion and filesystem, session,
or sensor-integrity failures remain fail-closed.

Source-session identity never reads `.omx/`. The current Codex thread key is optional; when absent,
the guard derives a process-scoped identity from the live Codex host PID and mandatory process
creation digest. Controls bind to the thread or process identity plus that creation digest. A clean
clone initializes only Hive-owned state under `.agents/work/usage-guard/`, and a new thread never
inherits another thread's bypass.

Recovery controls do not initialize quota sensors. `status` reports the last observation with
explicit freshness, while a disabled gate skips quota sensing and returns `session_bypass`.
Watchers pin the host creation digest, retire a prior thread watcher on transition, and require a
matching locked child lease before any stop signal.

State writes are portable across supported hosts. Descriptor permission hardening uses
`os.fchmod` only when callable, and every pre-stream failure closes the descriptor before temporary
path cleanup. Windows watcher lease checks skip the locked first byte and reconstruct the guaranteed
JSON object prefix. Full source guard conformance runs on Linux and macOS; focused Windows
compatibility regressions cover state writes, lease reads, clean-clone `gate`, `session-disable`,
and disabled `gate` before the broader Phase 1 corpus.

Regression acceptance covers clean-clone gate and disable, disabled gate allowance, non-transfer to
a new thread or recreated process, unchanged malformed OMX bytes, prior-watcher retirement, and
refusal to signal an unrelated process even when watcher state contains its genuine PID and process
creation digest. The originating requirement was removal of the bootstrap deadlock caused by
treating optional OMX runtime state as mandatory source-guard authority.
