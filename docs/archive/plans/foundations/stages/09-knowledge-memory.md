# Stage 9. Karpathy Raw/Wiki/Schema memory

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

```text
.hive/knowledge/
├── Raw/
├── Wiki/
│   ├── index.md
│   └── log.md
├── Schema/
│   └── schema.md
└── suppression.yml
```

#### Raw

- 원본·정규화 source
- 기존 source 내용 직접 수정 금지
- source 변경은 새 revision
- 폐기된 source는 active tree에서 삭제
- 기밀정보·credential 저장 금지

#### Wiki

- agent-maintained interlinked Markdown
- source-summary, entity, concept, comparison, synthesis, open-question 단위
- hashtag/tag와 Raw locator
- 새 source를 기존 관련 page에 누적 통합
- contradiction은 양쪽 source를 표시
- deprecated/superseded page는 active tree에서 삭제

#### Schema

- page kind, frontmatter, relation, tag와 operation contract
- provider instruction file과 분리
- 사용자와 agent가 versioned migration으로 공동 진화

#### 삭제와 suppression

삭제 순서:

1. obsolete Raw/Wiki/page/claim을 active tree에서 제거
2. backlink, index와 tag 제거
3. 최소 suppression entry 기록
4. SQLite rebuild 또는 incremental delete
5. stale reference lint

Suppression entry에는 fingerprint, source locator, reason, replacement와 timestamp만 포함.
`reason`은 shipped stable reason-code enum이며 삭제 본문을 복제 금지.

일반 삭제는 Git history에서 복원 가능. Secret/legal erase만 별도 승인된 history rewrite와 backup purge 수행.

#### 병렬 처리

여러 source의 extraction은 독립 read-only agent로 병렬화 가능. Canonical Wiki integration은 한 curator가 serial하게 수행해 duplicate, contradiction, backlink와 index를 한 번에 정산.

#### SQLite

용도:

- FTS5 full-text search
- tag·alias index
- backlink·source graph
- incremental indexing용 content hash
- ranking cache

`hive index rebuild`:

1. canonical Markdown scan
2. frontmatter/tag/link/source parse
3. temp SQLite 생성
4. page count·logical digest·query fixture 검증
5. atomic replace

Model call과 network 불필요. SQLite byte hash 동일성은 요구 금지.

#### 완료 조건

- [x] SQLite 삭제 후 같은 page ID·tag·link·content digest 재구축
- [x] 동일 Markdown checkout에서 query result equivalence
- [x] deprecated/superseded content가 active Wiki와 search 결과에 없음
- [x] suppression ledger에 삭제 본문 없음
- [x] contradiction, orphan, broken link, missing citation, stale index 탐지
- [x] parallel extraction + serial integration에서 lost update 없음
- [x] Git LFS 없이 canonical knowledge 동기화 가능
