# Atomic fact 안내

`docs/` Wiki의 검색용 원자 knowledge 계층.

## 경로

- `en/`: English fact
- `ko/`: 같은 `pair_id`의 Korean fact

## Page 원칙

- Primary fact 1개
- 직접 필요한 context만 포함
- Exact bilingual pair와 reciprocal counterpart
- Current repository source locator·digest
- Reviewed Git revision
- Related fact는 body 병합 대신 link
- Raw transcript·tool output·runtime state 수집 금지

## Fact catalog

| Fact | English | 한국어 |
| --- | --- | --- |
| Product purpose | [en](en/product-purpose.md) | [ko](ko/product-purpose.md) |
| Product non-goal | [en](en/product-non-goals.md) | [ko](ko/product-non-goals.md) |
| Supported host | [en](en/supported-hosts.md) | [ko](ko/supported-hosts.md) |
| Artifact boundary | [en](en/artifact-boundaries.md) | [ko](ko/artifact-boundaries.md) |
| Crate ownership | [en](en/crate-ownership.md) | [ko](ko/crate-ownership.md) |
| Orchestration owner | [en](en/orchestration-ownership.md) | [ko](ko/orchestration-ownership.md) |
| Docs Wiki architecture | [en](en/docs-wiki-architecture.md) | [ko](ko/docs-wiki-architecture.md) |
| 응답 언어 일관성 | [en](en/language-consistency.md) | [ko](ko/language-consistency.md) |
| 검증 결과 명확성 | [en](en/verification-result-clarity.md) | [ko](ko/verification-result-clarity.md) |
| 사용자 인계 전 자동 처리 | [en](en/automated-user-handoff.md) | [ko](ko/automated-user-handoff.md) |
| Knowledge preservation | [en](en/knowledge-preservation.md) | [ko](ko/knowledge-preservation.md) |
| Knowledge storage | [en](en/knowledge-storage.md) | [ko](ko/knowledge-storage.md) |
| Shared index | [en](en/shared-index.md) | [ko](ko/shared-index.md) |
| Global knowledge RAG | [en](en/global-knowledge-rag.md) | [ko](ko/global-knowledge-rag.md) |
| Knowledge portability·scan | [en](en/knowledge-portability-scan.md) | [ko](ko/knowledge-portability-scan.md) |
| 공유 색인 대상 경로 안전 | [en](en/shared-index-target-safety.md) | [ko](ko/shared-index-target-safety.md) |
| Global onboarding | [en](en/global-onboarding.md) | [ko](ko/global-onboarding.md) |
| Project onboarding | [en](en/project-onboarding.md) | [ko](ko/project-onboarding.md) |
| Plugin update merge | [en](en/plugin-update-merge.md) | [ko](ko/plugin-update-merge.md) |
| Skill routing | [en](en/skill-routing.md) | [ko](ko/skill-routing.md) |
| Role state | [en](en/role-state.md) | [ko](ko/role-state.md) |
| Run recovery | [en](en/run-recovery.md) | [ko](ko/run-recovery.md) |
| Usage sensor | [en](en/usage-sensor-policy.md) | [ko](ko/usage-sensor-policy.md) |
| Automatic dispatch guard | [en](en/automatic-dispatch-guard.md) | [ko](ko/automatic-dispatch-guard.md) |
| Source usage guard | [en](en/source-usage-guard.md) | [ko](ko/source-usage-guard.md) |
| Source watcher process replacement | [en](en/source-watcher-process-replacement.md) | [ko](ko/source-watcher-process-replacement.md) |
| Windows source watcher identity | [en](en/windows-watcher-identity.md) | [ko](ko/windows-watcher-identity.md) |
| Judge verification | [en](en/judge-verification.md) | [ko](ko/judge-verification.md) |
| Release verification | [en](en/release-verification.md) | [ko](ko/release-verification.md) |
| Linux musl qualification | [en](en/linux-musl-qualification.md) | [ko](ko/linux-musl-qualification.md) |
| Test fault isolation | [en](en/test-fault-isolation.md) | [ko](ko/test-fault-isolation.md) |
| Windows namespace gate timeout | [en](en/windows-namespace-gate-timeout.md) | [ko](ko/windows-namespace-gate-timeout.md) |
| Update transaction | [en](en/update-transaction.md) | [ko](ko/update-transaction.md) |
| Update discovery | [en](en/update-discovery.md) | [ko](ko/update-discovery.md) |
| Interactive binary update | [en](en/interactive-binary-update.md) | [ko](ko/interactive-binary-update.md) |
| Windows PowerShell module isolation | [en](en/windows-powershell-module-isolation.md) | [ko](ko/windows-powershell-module-isolation.md) |
| Version policy | [en](en/version-policy.md) | [ko](ko/version-policy.md) |
| npm `0.8.0` distribution | [en](en/test-distribution.md) | [ko](ko/test-distribution.md) |
| Source development | [en](en/source-development.md) | [ko](ko/source-development.md) |
| Marketing deck record | [en](en/marketing-deck-record.md) | [ko](ko/marketing-deck-record.md) |
| v0.9 Skill suite 계획 | [en](en/v0-9-skill-suite-plan.md) | [ko](ko/v0-9-skill-suite-plan.md) |

## 정본 관계

Fact: current source·ADR·architecture의 reviewed retrieval projection.
Source와 불일치 시 source·ADR 우선, stale fact 갱신 필요.

Derived SQLite와 advisory lock:

```text
.agents/work/source-wiki/index.sqlite3
.agents/work/source-wiki/.index.lock
```

둘 다 Git 제외 상태. Explicit `hive source-wiki index`만 rebuild authority 보유.
