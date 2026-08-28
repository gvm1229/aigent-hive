---
schema_version: 1
pair_id: verified-workflow
topic_slug: verified-workflow
language: en
counterpart: ../ko/verified-workflow.md
title: "Verified Workflow Skill"
summary: "0.10.0 routes complex natural-language continuation through verified-workflow and has a disposable acceptance for retry, Judge, recovery, and cancellation."
tags: [orchestration, skills, v0-10]
aliases: ["ralph-loop"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:592d39a3e71369cad9be4a789e7657509193b473893d2ab413d8218634022e1b"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:f88a14b8ddbe260320ddc58feba5c6b953b532e403bb6f7c3af7d0ebe035880f"
  - "repo:scripts/accept-verified-workflow.py#sha256:ad4bfb4f5c2b477a5900f0e28161ce1baee155af1b96cb73e93a0ec871a149a5"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:f050bb65eeed570541346af6dc22c52cdc6dbaf9"
status: active
---

# Verified Workflow Skill

`verified-workflow` combines evidence-gated graphs, bounded retry, independent verification, and
exact recovery. Natural continuation selects it only with at least two declared workflow signals;
task length and bare continue do not qualify. A disposable acceptance verified normalized routing,
canonical run creation, intentional failure then successful retry, a separate host-owned Judge
receipt, fresh-process/session recovery, and terminal cancellation in one receipt. It proves CLI
process recovery, not a Codex desktop restart, and gives no quorum authority to one Judge receipt.
