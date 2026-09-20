---
schema_version: 1
pair_id: native-host-qualified-scope
topic_slug: native-host-qualified-scope
language: en
counterpart: ../ko/native-host-qualified-scope.md
title: "Qualified Native File Hook Scope"
summary: "Native hook acceptance is limited to measured file paths and failure handling."
tags: [acceptance, hooks, scope]
aliases: []
sources:
  - "repo:docs/research/native-host-qualification-0.11.0.md#sha256:16e1a415e894e0502f46715f63d14d9b662f4c7c1fc445aecd0677e3e50f4aa2"
links: [antigravity-file-hook-acceptance, codex-native-error-observation]
reviewed_revision: "git:6c7a6485479b009b74d7ddac93c05291a061eef4"
status: active
---

# Qualified Native File Hook Scope

0.11.0 qualifies trusted loaded format-3 file guards in the tested Windows Codex and Antigravity versions. Both allow normal samples, deny protected samples, and deny edits when the registered checker is absent. Six-case pairs show three protected edits blocked with hooks and allowed with instructions alone. Native timeout, malformed response/configuration, and unmatched tools are not mandatory enforcement paths. Codex child scope is one tested same-target child; Antigravity inheritance and other OS-native hosts remain unverified.
