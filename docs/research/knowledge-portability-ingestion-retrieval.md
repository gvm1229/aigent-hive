# Knowledge 이식·directory 수집·검색 조사

> 확인일: 2026-08-01
> 범위: Hive user-root·project knowledge의 machine 간 이식, directory bulk scan,
> automatic retrieval

## 저장소 기준선

| 영역 | 현재 구현 | 판정 |
| --- | --- | --- |
| 이식 | 별도 `export|import` 없음 | 미충족 |
| 수집 | 선택 source 1개 + reviewed Wiki draft `ingest` | bulk scan 미충족 |
| Project 구분 | 단일 SQLite `pages.source_project` | normalized collection 기반 보유 |
| 전역 승격 | Exact page의 `fact|preference|workflow` 승인형 `promote` | 재사용 가능 |
| 검색 | Explicit-only `hive-knowledge-query` | automatic retrieval 미충족 |

## 외부 근거

- [SQLite Backup API](https://www.sqlite.org/backup.html): 실행 중 DB의 일관된 snapshot
  지원. 단순 file copy의 lock·중단 손상 한계와 derived DB 이식의 부적합성 근거
- [SQLite `VACUUM INTO`](https://www.sqlite.org/lang_vacuum.html#vacuuminto): 최소 크기
  live snapshot 지원. 정본·schema·machine binding 이식 문제의 해결 수단은 아님
- [RFC 8493 BagIt](https://www.rfc-editor.org/rfc/rfc8493.html): payload 전체 목록,
  상대 path와 checksum 검증 기반의 reliable transfer 구조
- [`git ls-files`](https://git-scm.com/docs/git-ls-files): tracked file과
  `--others --exclude-standard`를 이용한 standard ignore 적용
- [SQLite FTS5](https://www.sqlite.org/fts5.html): weighted BM25, 빠른 `rank` 정렬,
  비용이 큰 전체 `optimize`와 단계적 merge 선택지
- [OWASP Secrets Management](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html):
  secret 최소 권한·노출 제한·log 평문 기록 금지
- [W3C PROV-O](https://www.w3.org/TR/prov-o/): Entity·Activity·Agent와
  `wasDerivedFrom|wasRevisionOf|hadPrimarySource` 중심 provenance 최소 model
- [Anthropic Contextual Retrieval](https://www.anthropic.com/engineering/contextual-retrieval):
  document context를 보존한 chunk + BM25의 retrieval failure 감소 보고. Provider-neutral
  benchmark 재현 전 제품 성능 근거로 단정 금지
- [Lost in the Middle](https://arxiv.org/abs/2307.03172): 긴 context의 중간 evidence 활용
  저하 관찰. Corpus 전체 주입보다 bounded top-k retrieval 필요성

## 권고

1. SQLite file 대신 canonical Markdown·portable config·suppression·provenance를 담은
   checksummed `.hivekb` bundle 이식
2. `collection_id` 기반 normalized row와 stable logical identity 사용, directory basename은
   display alias로 한정
3. Deterministic inventory → agent-reviewed claim → project collection upsert → reusable
   candidate promotion의 2단계 scan
4. 기존 `hive-knowledge-query`를 단일 automatic retrieval Skill로 확장,
   별도 find Skill 중복 금지
5. Internal knowledge 우선 조회 뒤 freshness 부족 시 external research handoff

## Claim 분류

| `claim_kind` | 최소 evidence | Root promotion |
| --- | --- | --- |
| `project-profile` | README·manifest·등록 identity | 기본 제외 |
| `decision` | ADR·intention 문서·user statement | Portable decision만 후보 |
| `convention` | 구성·source·test의 반복 증거 | 적용 조건 포함 후보 |
| `preference` | User-authored instruction·statement | Secret 검사 뒤 후보 |
| `workflow` | 문서화된 단계 + 실행 evidence | Portable workflow만 후보 |
| `dependency-evidence` | Manifest·lock의 exact version | 단독 promotion 제외 |
| `outcome` | Test·build·acceptance receipt | Version·revision 한정 후보 |
| `question` | 미해결 TODO·명시적 의문 | Fact promotion 금지 |

`assertion_status`: `user-stated|observed|verified|inferred|conflicted|superseded`.
Dependency 존재만으로 성공 판정 금지. 예: Next.js 16 manifest·lock은 사용 evidence,
test·build receipt는 해당 revision의 성공 evidence, convention 재사용은 별도 조건부 후보.

## 적대적 검토

| Pass | Finding | 계획 반영 |
| --- | --- | --- |
| 1 — overlap | `scan`과 기존 `capture|promote`, 새 find Skill의 기능 중복 | Scan은 inventory·candidate 생성, capture는 canonical write, promote는 root 승격, query는 retrieval 단일 owner |
| 1 — schema | Directory별 table의 schema 폭증·rename collision | 고정 table + `collection_id` row |
| 2 — transfer | Hash를 authenticity로 오인, SQLite·absolute path의 machine binding | Hash는 내부 무결성만 표시, DB·absolute path 제외, destination trust 별도 |
| 2 — hostile input | Zip slip·symlink·casefold·Unicode·archive bomb | Extraction 전 manifest·path·count·size 검증과 staging confinement |
| 2 — poisoning | Secret·third-party code·prompt injection의 knowledge 승격 | Bounded excerpt·license·secret gate, retrieved text의 untrusted data 처리 |
| 3 — truth | Dependency 사용을 성공·best practice로 과장 | `assertion_status`, version·revision·test evidence와 조건부 promotion |
| 3 — portability | 다른 machine의 project path 부재 | Detached collection import + explicit local mapping |
| 3 — context | 매 turn 과다 retrieval·외부 최신성 대체 | 1회 top-k·byte budget, no-hit handoff, current external research 분리 |
| 4 — scope | `all-portable`의 private 포함 여부 불명확 | Scope별 포함 집합 명시, confidential 전면 제외 |
| 4 — routing | Query Skill과 task Skill의 동시 body load 가능성 | Retrieval 종료 뒤 sequential handoff, 동시 body 1개 |

사용자 결정 잔여: 없음. Bundle signing·encryption·network sync는 v0.9 비목표.
구현 전 deterministic ZIP library의 license·maintenance·size 검토만 잔여.
