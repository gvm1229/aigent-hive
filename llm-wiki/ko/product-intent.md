---
schema_version: 1
pair_id: product-intent
topic_slug: product-intent
language: ko
counterpart: ../en/product-intent.md
title: "제품 의도와 방향"
summary: "대상 사용자, onboarding, knowledge, host integration과 0.8.0 release 방향."
tags: [intent, onboarding, product, release]
aliases: ["Hive 제품 방향"]
sources:
  - "repo:docs/decisions/ADR-0011-source-wiki-independence.md#sha256:15dbcb1c9e294078dc641d0c51c3655bd047cdf1c57629cb4158e7d047097f1b"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:aa1f7e4271db8f3e1ceac5e0b54ed7451405513f37d65571b3e0df899930a8c0"
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:3b5b29532bd3e353aaaea9b95637a6582bf6c6ab5dab01ebfc61bc7967ecd613"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a06b937b8b5b71ee7515e474e96bcd0f07ba66e6977a197380fb42fa88a035ac"
links: [boundaries, knowledge, plugin-lifecycle, security-release, skill-routing, usage-hosts]
reviewed_revision: "git:51f40e24316e9f776626ddf73676f7719b020a42"
status: active
---

# 제품 의도와 방향

대상 사용자: Subscription-authenticated Codex, Claude Code 또는 Antigravity host를 사용하는
developer와 non-developer. Provider API key 없이 durable project-aware assistance를 원하는
사용자. Hive 소유 범위: local setup, selected Skill, directive, memory, validation, safe
upgrade. Model execution 소유권은 host에 유지.

## User lifecycle

- 최초 설치 뒤 mandatory user setup
- Expedited setup의 모든 기본값: English 대화, enabled English Wiki, strict persona,
  모든 built-in Skill 설치, explicit opt-in 전 usage guard disabled
- Custom setup 항목: language, Wiki language, user profile, active host, persona, Skill,
  optional usage threshold
- LLM Wiki default-on, canonical Markdown 삭제 없는 disable·re-enable
- Material task completion의 agent-reviewed fact capture
- Raw transcript·hidden prompt·credential·runtime payload의 knowledge 유입 금지

## Project lifecycle

- Global preference 상속, repository evidence 분석과 unresolved essential question만 사용하는
  auto onboarding
- Project별 `AGENTS.md`, `.agents/`, project config와 canonical Markdown Wiki
- Project별 SQLite 0개, global·project Markdown를 포괄하는 user-root SQLite 1개
- Signed historical base 기반 upgrade: unmodified file의 incoming replace, modified file의
  overlapping user edit 우선과 non-conflicting incoming change 추가

## Interaction strategy

- Narrow description 기반 approved Skill automatic routing
- Long-tail capability의 explicit-only metadata
- 명시적 prompt 작성 intent의 refinement, 모호하거나 detail 부족 prompt의 optional
  refine 제안 1회, automatic rewrite 0회
- Qualified native usage sensor 우선, CodexBar fallback-only
- OMX·OMC의 current compatibility dependency 활용과 durable ownership 금지
- 향후 OMX·OMC 제거 시 Wiki migration 0건

## `0.8.0` 방향

첫 public target: `Aigent Hive 0.8.0`. Pre-1.0 SemVer를 성숙도 신호로 사용하고 별도
preview label·npm dist-tag 제외. 주 설치 명령: `npm install -g aigent-hive`. 병렬 channel:
GitHub Release, 검증된 Unix·PowerShell·CMD installer. 단일 native artifact 계보:
macOS arm64·Intel, Linux musl x86_64·arm64, Windows x86_64.

Codex·Antigravity live evidence 필수. Claude는 subscription-backed test 전
theory-supported·unverified 표시. 5개 platform SHA-256·GitHub artifact attestation,
npm binary identity, clean install·recovery는 release gate 유지. Platform signing과
external TUF production quorum은 future hardened channel로 deferred.

Hive non-goal: model runtime, scheduler, provider API client, credential store, host-native
orchestration 대체.
