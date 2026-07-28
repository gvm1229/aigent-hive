---
schema_version: 1
pair_id: product-intent
topic_slug: product-intent
language: en
counterpart: ../ko/product-intent.md
title: "Product Intent and Direction"
summary: "Target users, onboarding, knowledge, host integration, and preview-release direction."
tags: [intent, onboarding, product, release]
aliases: ["Hive product direction"]
sources:
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:15dbcb1c9e294078dc641d0c51c3655bd047cdf1c57629cb4158e7d047097f1b"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:aa1f7e4271db8f3e1ceac5e0b54ed7451405513f37d65571b3e0df899930a8c0"
  - "repo:docs/decisions/ADR-0013-preview-release-scope.md#sha256:eb5f53e2cc1168888bb5117fdd91ede7016312ee79f8894581017e2e1b1976c5"
links: [boundaries, knowledge, plugin-lifecycle, security-release, skill-routing, usage-hosts]
reviewed_revision: "git:d46e9b7deb5c54fc7cec00c38483388ce563ff1d"
status: active
---

# Product Intent and Direction

Aigent Hive is for developers and non-developers who use subscription-authenticated Codex, Claude
Code, or Antigravity hosts and want durable, project-aware assistance without provider API keys.
Hive supplies local setup, selected Skills, directives, memory, validation, and safe upgrades while
the host continues to own model execution.

## User lifecycle

- First install enters mandatory user setup.
- Expedited setup means all defaults: English conversation, enabled English Wiki, strict persona,
  every built-in Skill installed, and usage guard disabled until explicit opt-in.
- Custom setup records language, Wiki languages, user profile, active hosts, persona, Skills, and
  optional usage-guard threshold.
- The LLM Wiki is default-on and can be disabled or re-enabled without deleting canonical Markdown.
- Material completed work receives agent-reviewed task-fact capture; raw transcripts, hidden
  prompts, credentials, and runtime payloads never become knowledge.

## Project lifecycle

- Auto onboarding inherits global preferences, reads repository evidence, and asks only unresolved
  essential questions.
- Each project owns `AGENTS.md`, `.agents/`, project configuration, and its canonical Markdown Wiki.
- Projects do not own SQLite databases. One user-root SQLite index covers global and project
  Markdown so reusable knowledge can be found across projects.
- Upgrade uses signed historical base bytes: unmodified files accept incoming replacement;
  modified files keep overlapping user edits and receive non-conflicting incoming changes.

## Interaction strategy

- Approved Skills route automatically from narrow descriptions, with explicit-only long-tail
  capabilities kept out of default context.
- Clear prompt-writing intent can invoke refinement; ambiguous or detail-poor prompts receive one
  optional refinement suggestion without automatic rewrite.
- Usage guards prefer qualified native sensors and use CodexBar only as fallback.
- OMX and OMC are useful current compatibility dependencies, not durable product owners. Their
  eventual retirement must not require Wiki migration.

## `0.8.0` direction

The first public target is a `Claude-unverified preview`. Codex and Antigravity need live evidence;
Claude remains theory-supported and visibly unverified until a subscription-backed test is
possible. Real Windows-machine acceptance, clean current CI, bounded Skill metadata, SHA-256, and
GitHub artifact attestation remain release gates. Platform signing and external TUF production
quorum are deferred to a future hardened channel.

Hive does not become a model runtime, scheduler, provider API client, credential store, or
replacement for host-native orchestration.
