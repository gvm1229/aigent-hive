# ADR-0003: Markdown 정본과 SQLite projection

- 상태: accepted
- 날짜: 2026-07-23
- 부분 대체: Consumer Notion mode의 정본·rebuild는 [`ADR-0018`](ADR-0018-notion-wiki-backend.md)

## 결정

지식, role identity, run plan/status와 evidence manifest는 Markdown에 저장. Setup answers, typed config, approval ledger와 suppression fingerprint는 tracked YAML/TOML에 저장. SQLite는 모든 canonical tracked source에서 재구축 가능한 local search projection으로만 사용.

## SQLite 용도

- FTS5 전문 검색
- tag·alias 정규화
- backlink와 source 관계
- incremental indexing용 content hash
- query ranking cache

## 금지

- SQLite에만 존재하는 durable fact
- SQLite file hash를 migration 호환성 기준으로 사용
- DB를 Git 또는 Git LFS에 포함

## 재구축

동일 Git checkout에서 model call과 network 없이 `hive index rebuild` 실행. DB byte hash가 아니라 page ID, content digest, tag, link와 query result equivalence 검증.
