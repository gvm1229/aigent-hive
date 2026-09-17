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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:5327d6c3417a62069df8eda30e76fe907c48418806023847eb16189cbe3041ef"
  - "repo:docs/decisions/product-release-decisions.md#sha256:9f234ef3fede8030ab6ad57fa4b560a71f30f468622c4e4ad56b83864d0e05ce"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:6b2ee66b721493ad6c69f2f22d56ed16a6b47886e7609a4be42bff9aea768c57"
  - "repo:scripts/accept-verified-workflow.py#sha256:ad4bfb4f5c2b477a5900f0e28161ce1baee155af1b96cb73e93a0ec871a149a5"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Verified Workflow Skill

`verified-workflow` combines evidence-gated graphs, bounded retry, independent verification, and
exact recovery. Natural continuation selects it only with at least two declared workflow signals;
task length and bare continue do not qualify. A disposable acceptance verified normalized routing,
canonical run creation, intentional failure then successful retry, a separate host-owned Judge
receipt, fresh-process/session recovery, and terminal cancellation in one receipt. It proves CLI
process recovery, not a Codex desktop restart, and gives no quorum authority to one Judge receipt.
