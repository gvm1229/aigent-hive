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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:b88eaf08d187d6f83cfac8b9e3a186791f08b71d0d5287f5dafe4d2e7aaa8151"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:c2043678fc1e5ad2e8e2a9cb716e45ec44486b67f3a9af2349c8909e6f4b3a8b"
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
