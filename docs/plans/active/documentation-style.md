# 사람용 문서 style 완료 계획

> Checklist owner: `DOC-*`
> Load condition: 문서 style 교정·검증·semantic review

### 실행 기록 — 전체 문서 말투 정리

- 상태: final semantic correction·verification 진행 중
- 범위: 사전 선별 없는 workspace 문서 후보 전수 inventory·read·disposition
- 수정 대상: Korean·mixed-language의 사람용 authored prose
- 보호 대상: AI directive·Skill 원문, license, exact quote·UI·protocol, hostile fixture,
  generated/runtime output와 third-party bytes
- 수정 원칙: 의미 중심 문장·문단 재작성, 일괄 suffix 치환 금지
- Source/generated 원칙: canonical producer 우선, output-only patch 금지
- Product version: `0.7.0` 유지

현재 evidence:

| 항목 | 결과 |
| --- | --- |
| Fresh inventory | Checker가 current candidate 전수 산출 |
| 전수 disposition | `reviewed_count == inventory_count`, `unreviewed_count=0` |
| Checker | `scripts/check-human-documentation-style.py` |
| Regression | `tests/conformance/test_human_documentation_style.py` 16/16 PASS |
| Projection parity | Source/template contract와 compiled marker test PASS |
| Markdown | Local link·heading anchor `130 files`, `123 links`, 오류 0건 |
| 규칙 | 비제한 sentence-form, 기계적 nominal ending, authored blockquote 검사 |
| Exact literal | path·line·reason·line SHA-256 결합 allowlist |
| Semantic review | 자동 검사 밖의 잘린 명사·조사 오류·stale 상태 발견, 교정 진행 중 |
| 외부 dependency | 추가 0개 |

Completion gate:

- [x] [DOC-001] Fresh checker finding 0건, stale exception 0건
- [x] [DOC-002] Source/template/compiled projection parity
- [x] [DOC-003] Local Markdown link·heading anchor PASS
- [ ] [DOC-004] Changed human docs independent semantic review PASS
- [ ] [DOC-005] 전체 workspace·conformance 검증 PASS
