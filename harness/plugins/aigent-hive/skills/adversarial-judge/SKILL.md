---
name: adversarial-judge
description: (adversarial-judge) Prepare an explicit clean-context request for an independent host-owned Judge. Use only for an explicitly requested Judge step or a verified-workflow Judge node; do not use for ordinary continuation, verdict production, signing, or task-agent self-review.
---

# Adversarial Judge

Prepare the smallest clean-context Judge request for an explicitly selected subject.

## Workflow

1. Require one explicit user request or one verified-workflow node that names independent adversarial review.
2. Keep only the subject, risk tier, acceptance criteria, artifact digests, verification evidence digests, and known constraints.
3. Exclude task-agent reasoning, preferred outcomes, self-score, prior Judge results, transcripts, and unrelated project memory.
4. Run `hive judge package` to validate the clean-context package. Do not recreate package validation in prose.
5. Require the active host to launch a separate Judge only after a schema-valid assignment and capability receipt exist. Unsupported hosts return `unsupported`; do not substitute a process, provider API, watcher, or another agent.
6. Treat findings as diagnostic. Completion authority remains with the existing authenticated quorum contract.

## Boundaries

- Hive never calls a provider API, reads provider credentials, spawns a process, signs an artifact, or writes a Judge result.
- The task agent, requester, and any prior Judge result never become Judge context or approval authority.
- User cancellation and unavailable host capability stop the step without retry or hidden fallback.
