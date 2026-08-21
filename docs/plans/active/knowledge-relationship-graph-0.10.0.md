# 지식·Code 관계 graph `0.10.0`

> Checklist owner: `SCP10-001`, `KRG10-*`
> 승인 근거: 2026-08-22 유지보수자 범위 확정
> 연구 근거: [`AI_Learning` 적용 후보](../../research/ai-learning-hive-application-candidates-2026-08-21.md)

## 역할

- Markdown: 유일한 지식 정본
- SQLite FTS: 직접 사실·본문 검색
- Hive-native graph: 명시적 Markdown 관계
- Graphify: 선택형 source·project code 관계 추출
- 모든 graph: 삭제·재생성 가능한 파생 상태

## Checklist

- [x] [SCP10-001] 제품 범위 승인: Hive-native Markdown 관계 graph, optional Graphify full-rebuild code-only adapter, FTS·relation query planner, metadata-first retrieval, graph scope 격리·drift gate
- [ ] [KRG10-001] engine·node·edge·generation·query 결과 schema와 Markdown·SQLite·Graphify ownership 계약
- [ ] [KRG10-002] `sources|source_links|links|related_concepts|duplicate_of|contradictions|topics|tags|replacement`의 결정론적 Markdown edge 추출
- [ ] [KRG10-003] source·project·user-root·shared·private·confidential collection별 물리적 graph generation 격리
- [ ] [KRG10-004] Hive-native Markdown graph의 전체·digest 기반 증분 rebuild와 정규화 동등성
- [ ] [KRG10-005] 사실·본문은 FTS, 문서 관계는 native graph, code 영향은 Graphify로 보내는 query planner와 결합 결과
- [ ] [KRG10-006] metadata 최대 10개 우선 반환과 선택 `item_id` 본문 조회, 전체 검색 최대 50개·본문 자동 확장 `0건`
- [ ] [KRG10-007] `active|contradicted|superseded|expired|revoked` 생명주기, 기본 retrieval 제외, build·query·검토 비용 receipt
- [ ] [KRG10-008] `graphifyy==0.9.47`·Python·dependency exact digest preview와 사용자 승인형 격리 환경
- [ ] [KRG10-009] Graphify `extract --force --code-only` full rebuild, project-relative locator와 Hive edge schema 정규화
- [ ] [KRG10-010] source commit·입력 digest·extractor version drift 검출, staging 검증 뒤 atomic generation activation
- [ ] [KRG10-011] Graphify 미설치·손상·schema mismatch·build 실패의 native graph·FTS 정상 대체와 rollback
- [ ] [KRG10-012] 관계 경로·scope·locator·digest·`EXTRACTED` evidence의 JSON·HTML export
- [ ] [KRG10-013] provider API·API key·query log·watcher·Git hook·자동 MCP 등록·upstream `global|update` 호출 `0건`
- [ ] [KRG10-014] 의미 유사성 질문 기준선과 FTS·alias 결손 측정, vector database·embedding runtime product dependency `0건`
- [ ] [KRG10-015] pre-`0.10.0` canonical Markdown·collection registry·기존 FTS 결과 byte 보존 upgrade·disable·rollback
- [ ] [KRG10-016] 30개 관계 질문·직접 사실 무회귀·성능·격리·Windows x64·macOS arm64·Linux musl 공개 시험 수용

## Graphify 금지 경로

- `graphify update`
- `graphify global`
- Markdown LLM extraction
- `--watch`
- provider backend·credential
- 소비자 project의 `graphify-out/`

## 공개 명령 후보

```text
hive knowledge graph preview|enable|status|rebuild|query|export|disable
hive source-wiki graph status|rebuild|query|export
```

새 Skill ID 추가 없음. 기존 `knowledge-recall`·`knowledge-maintain`의 routing 확장.
