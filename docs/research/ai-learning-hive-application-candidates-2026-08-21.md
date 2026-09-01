# `AI_Learning` 지식의 Hive 적용 후보

> 분석일: 2026-08-21, 추가 scan: 2026-08-22
> 외부 자료: Obsidian `AI_Learning` 프로젝트
> 목적: Hive `0.10.0` 결함·제품 범위 검토
> 변경 경계: 외부 프로젝트 읽기 전용, 원문·비공개 지식의 source workspace 복사 `0건`

## 결론

권장 방향: Hive-native Markdown 관계 graph와 선택형 Graphify code graph의 결합.

- Markdown 정본의 명시적 관계: Hive가 직접 추출
- Source code의 구조 관계: Graphify `--code-only` 사용
- 직접 사실·본문 검색: 기존 SQLite FTS 유지
- 세 결과의 선택·결합: Hive query planner 소유
- Graph 자료: 삭제·전체 재생성 가능한 파생 상태
- 등록된 하위 폴더 scan 결함: `0.10.0` 필수 수정

Graphify 전면 폐기 또는 전체 지식 graph 위임 모두 비권장. Graphify의 검증된 code 관계
추출만 사용하고, 검증 실패 영역은 Hive의 결정론적 계약으로 대체하는 구성.

## 분석 근거

### 자료 규모

- 전체 파일: `339개`, 약 `527KiB`
- 현재 분석 대상 Markdown: `59개`
- `Knowledge/`: 개념 노트 `18개`
- `Maps/`: 분류·통합 색인 `2개`
- `Sources/`: 출처 기록 `19개`
- 북마크 통합 색인: 입력 `88개`, 고유 URL `87개`

### 근거 품질

| 확인 수준 | 범위 | 적용 한계 |
| --- | --- | --- |
| 직접 본문 확인 | 공개 웹페이지 `2건` | 설계 패턴의 제한적 근거 |
| 제한 확인 | 제목·검색 결과 중심 `8건` | 세부 구현·성과 판정 불가 |
| X snippet | 다수 북마크 | dependency 채택 근거 사용 금지 |
| 프로젝트 실제 검증 | Hive setup·lint·scan 기록 | 해당 Windows·프로젝트 경계만 증명 |

외부 도구 이름만으로 dependency 채택 금지. 반복 확인된 설계 패턴과 프로젝트 실제 검증을
우선 적용 후보로 사용.

### 2026-08-22 추가 지식

- 벡터 데이터베이스와 파일·관계형·graph 기억의 역할 비교
- 기억 구축·질의·유지 관리 비용의 분리
- 중복·모순·만료·철회·대체·삭제를 포함한 기억 생명주기
- Architecture Map과 실제 source의 drift 검사
- Codebase Memory MCP·Deepsec·Qdrant·Codex Profile·Hook 등 도구 후보

판정:

- 벡터 검색: 관계 graph 대체물 아님, 의미 유사성 후보 검색용 후속 adapter
- Qdrant·Codebase Memory MCP·Deepsec: 직접 dependency 채택 전 별도 source·license·보안 검증 필요
- Architecture Map drift 검사: 현재 graph 권장안의 수락 기준으로 즉시 적용 가능
- 비공식 Codex Profile·Hook 제안: 공식 기능 대조 전 제품 범위 제외

## 적용 후보

### 1. 등록된 하위 폴더의 지식 scan

분류: `0.10.0` 필수 bugfix.

Release 결정: `0.9.5`로 `0.9.x` 종료, `0.9.6` 미출시. 해당 수정의 `0.10.0` 편입.

확인된 문제:

- 큰 Git Vault 아래에 등록한 하위 프로젝트의 repository-root discovery 실패
- `Sources/**`, `Knowledge/**`, `Maps/**`의 의미 색인 scan 중단
- 독립 `.git` 초기화와 Raw/Wiki ingest 우회 필요

적용 방법:

- 등록된 project root를 scan 권한 경계로 사용
- 상위 Git repository root와 등록 project root의 불일치 허용
- 등록 project root 밖 sibling read 금지
- 전역 `safe.directory` mutation 금지
- symlink·junction·reparse point·`..` 탈출 차단
- 상위 Vault의 sibling sentinel 불변 회귀 시험
- 허용 path pattern 기반 project collection scan

### 2. Hive-native Markdown 관계 graph

분류: `0.10.0` 핵심 제품 후보.

외부 프로젝트의 명시적 관계:

| Markdown metadata | Hive edge |
| --- | --- |
| `sources`, `source_links` | `SOURCED_BY` |
| `links`, `related_concepts` | `RELATED_TO` |
| `duplicate_of` | `DUPLICATE_OF` |
| `contradictions` | `CONTRADICTS` |
| `topics`, `tags` | `CLASSIFIED_AS` |
| replacement·superseded 상태 | `SUPERSEDED_BY` |

Node·edge 필수 정보:

- canonical locator와 content digest
- collection ID와 visibility
- source scope와 relation type
- `EXTRACTED` evidence
- extractor·schema version

초기 범위: 명시적 관계만 추출. `INFERRED`·`AMBIGUOUS` 관계 생성 보류.

구현 위치:

- `crates/hive-wiki`: Markdown metadata 정규화와 관계 추출
- 파생 SQLite: `graph_nodes`, `graph_edges`, `graph_generation`
- 새 결과 schema: 관계·경로·근거·범위 표현
- 파일 digest 기반 증분 갱신
- 증분 결과와 전체 재생성 결과의 정규화 동등성 검증

### 3. 선택형 Graphify code graph

분류: `0.10.0` 조건부 채택 후보.

확인된 통과 범위:

- Windows Python `3.12` 격리 설치
- Rust code graph 생성
- 반복 전체 생성 결과 동일
- code 관계 경로와 locator 반환
- 작은 graph cold query p95 약 `376ms`
- provider API·API key 없는 `--code-only`
- Graphify runtime dependency의 알려진 보안 finding `0건`

적용 방법:

- `graphifyy==0.9.47`과 dependency digest 고정
- `extract --force --code-only`만 허용
- `graphify update`·upstream `global`·`--watch` 금지
- scope마다 별도 HOME·output·graph generation
- staging 생성 → normalize → validate → atomic activation
- source locator의 project-relative 정규화
- Graphify schema를 Hive 공개 계약으로 직접 노출 금지
- Graphify 부재·손상·schema mismatch 때 FTS·native graph 유지

Graphify 관계 예:

- `DEFINES`
- `CALLS`
- `IMPORTS`
- `IMPLEMENTS`
- `REFERENCES`
- `TESTS`

### 4. Metadata-first 이단계 검색

분류: `0.10.0` 지원 기능 후보.

검색 순서:

1. 제목·요약·tag·scope·locator·digest 최대 `10개`
2. 사용자 또는 host가 선택한 항목만 본문 retrieval
3. 전체 검색도 metadata 최대 `50개`, 본문 자동 확장 금지
4. 중복 판정 기준: `canonical_url`, `content_hash`

적용 방법:

- 기존 `knowledge query`: metadata-first 경로로 명확화
- 기존 `knowledge retrieve`: 선택한 `item_id`의 bounded content 조회
- 기존 `knowledge-recall`: 두 단계 routing 통합
- 새 shipping Skill ID 추가 금지
- retrieval receipt에 후보 수·선택 수·반환 byte·generation 기록

### 5. 출처 ingest와 중복 관리

분류: `0.10.x` 또는 버전 비종속 backlog 후보.

Source record 계약:

- `source_url`: 입력 URL
- `canonical_url`: 최종 도착 URL
- `content_hash`: 핵심 본문 digest
- `duplicate_of`: 실질적 동일 자료 연결
- `verification`: 직접 확인·제한 확인·snippet·미검토
- `last_checked`: 마지막 확인 시점

처리 규칙:

- 같은 URL·같은 hash: no-op
- 같은 URL·다른 hash: source revision 갱신
- 다른 URL·같은 내용: 독립 출처 유지와 `duplicate_of` 연결
- 같은 주제·다른 주장: 기존 concept에 증분 병합
- 접근 실패: 추측 대신 유형화된 failure record

Host 역할: 원문 접근과 합법적 수집. Hive 역할: bounded receipt 검증, canonical Markdown 저장,
파생 index 갱신. Hive CLI의 provider API·credential·직접 model call `0건`.

### 6. Maturity·verification 검색 필터

분류: `0.10.x` 품질 후보.

권장 값:

- `maturity`: `concept|research|prototype|production`
- `verification`: `direct-source|limited|snippet|unreviewed`
- 기존 `assertion_status`·promotion 상태와 결합

효과:

- 관련 자료 검색과 제품 결정 근거 검색의 분리
- snippet-only 결과의 dependency 채택 근거 제외
- 오래된 version 정보의 낮은 신뢰도 표시

### 7. 관계 graph JSON·HTML export

분류: `0.10.0` 제품 가시성 후보.

- JSON: agent·CLI·시험용 node·edge·locator·digest·scope·evidence
- HTML: 사람의 구성 요소·의존성·영향 경로 탐색
- 생성 source commit·graph digest·stale 상태 표시
- canonical Markdown 대체 금지

예상 명령:

```text
hive knowledge graph query --format json
hive knowledge graph export --format html
hive source-wiki graph query --format json
hive source-wiki graph export --format html
```

### 8. Failure-domain 분류

분류: 후속 run 진단 후보.

권장 유형:

```text
model
context
memory
retrieval
tool
permission
evaluator
environment
product
```

기존 failure fingerprint와 결합. 반복 failure-domain의 자동 수정 금지, 회귀 시험 또는 backlog
후보 전환.

### 9. 벡터 검색의 측정형 검토

분류: `0.10.0` 연구·확장 계약 후보, product dependency 제외.

새 지식의 핵심:

- 벡터 검색의 강점: 표현이 다른 의미 유사 문서의 후보 선별
- 벡터 검색의 약점: 식별자·날짜·수치·부정·모순·여러 단계 관계
- 적합한 구성: keyword·metadata·relation·vector의 혼합 검색

`0.10.0` 적용 방법:

- FTS·Markdown relation·Graphify code relation을 먼저 query planner lane으로 고정
- 의미 유사성 질문 corpus와 현재 FTS·alias 기준선 작성
- 기준선 결손이 측정된 경우에만 vector adapter backlog 승격
- Qdrant·embedding runtime·model dependency 추가 `0건`
- Hive의 직접 embedding model 실행 `0건`

### 10. 기억 생명주기와 비용 receipt

분류: `0.10.0` 지식 품질 후보.

권장 상태:

- `active`
- `contradicted`
- `superseded`
- `expired`
- `revoked`

권장 측정:

- canonical 입력 byte와 생성 graph node·edge 수
- 전체·증분 build 시간
- query p50·p95와 반환 byte
- stale·expired·revoked 항목 수
- 사람 검토·모순 해결 건수

적용 경계:

- 원시 transcript 전체 자동 기억 금지
- 만료·철회 항목의 기본 retrieval 제외
- audit 요청의 locator·상태·replacement 제한 반환
- 자동 회고·Dreaming workflow: 실제 수요·host receipt 검증 전 backlog 유지

### 11. Architecture Map drift gate

분류: `0.10.0` graph 수락 기준.

적용 방법:

- Graph generation의 source commit·입력 digest·extractor version 기록
- 현재 source와 generation digest 불일치 때 `stale`
- stale graph의 현재 결과 승격 금지
- JSON·HTML export와 canonical source의 drift 검사
- 공개 시험판 CI의 rebuild → normalize → compare
- 작은 저장소 통과만으로 대규모 source 품질 주장 금지

## 권장 `0.10.0` 제품 범위

1. Nested registered-project knowledge scan 수정
2. Hive-native Markdown relationship graph
3. Optional Graphify code-only adapter
4. FTS·Markdown 관계·code 관계 query planner
5. Metadata-first retrieval
6. 기억 만료·철회·대체 상태와 비용 receipt
7. JSON·HTML graph export와 drift gate
8. Scope별 물리적 graph 격리
9. Graphify full rebuild only
10. Vector 검색 결손 기준선과 후속 adapter 경계
11. pre-`0.10.0` canonical Markdown·SQLite 결과 무손실 upgrade

### 권장 이유

- `0.9.5`의 안정적인 Markdown 정본·SQLite 검색 보존
- `0.9.x` 종료 뒤 발견한 nested project scan 결함의 다음 minor 수정
- 외부 프로젝트가 이미 가진 명시적 관계의 provider-free 활용
- Graphify 실제 통과 범위인 code 관계 추출의 선택적 사용
- Graphify 실패 범위인 증분 갱신·global visibility·Markdown LLM 추출의 제품 경계 제외
- 직접 사실·문장·관계·code 영향 질문의 역할 분리
- Graphify 장애와 설치 거부 때 기존 기능 유지

## `0.9.5` 대비 적용 전후

| 사용자 경험 | `0.9.5` 현재 | 권장 `0.10.0` 적용 뒤 |
| --- | --- | --- |
| 상위 Git 저장소의 하위 Hive project scan | repository-root discovery 실패 가능 | 등록 project root 안에서 안전한 scan |
| 지식 정본 | Markdown | Markdown 유지 |
| 직접 사실·본문 검색 | SQLite FTS | SQLite FTS 유지 |
| 문서 관계 질문 | 검색된 본문의 agent 해석 | 명시적 Markdown edge의 경로 조회 |
| Code 관계 질문 | `rg`·파일 확인·agent 추론 | 선택형 Graphify code path 조회 |
| 영향 범위 확인 | 여러 파일 수동 탐색 | `CALLS`·`IMPORTS`·`TESTS` 경로 탐색 |
| 근거 표시 | locator·digest·source | locator·digest·source·edge evidence |
| 검색 문맥 사용량 | 본문 chunk 중심 | metadata 선별 뒤 선택 본문 조회 |
| 오래된 기억 | superseded·contradiction 중심 | 만료·철회·대체와 기본 검색 제외 |
| 시각화 | 정형 graph export 없음 | JSON·HTML 관계 graph export |
| Graph 최신성 | 별도 관계 graph 없음 | source digest drift 때 stale 차단 |
| 범위 격리 | collection visibility | collection별 별도 graph까지 확대 |
| Graphify 필요성 | 없음 | code graph를 활성화한 scope만 필요 |
| Graphify 장애 | 영향 없음 | native graph·FTS로 정상 대체 |
| Upgrade | 기존 canonical 자료 | canonical byte 보존, graph 신규 생성 |

간단한 예:

```text
0.9.5
"이 변경이 어디에 영향을 주나?" → 검색 결과를 agent가 파일별로 해석

권장 0.10.0
"이 변경이 어디에 영향을 주나?" → 관계 경로 조회 → 관련 파일 본문만 선택 확인
```

## 채택 제외

- Puppetmaster dependency: Hive의 host-owned 실행 경계와 중복
- Semantica dependency: snippet 중심 근거로 license·보안·구조 판정 불가
- LangGraph 계열 orchestration dependency: canonical run graph와 역할 중복
- Graphify `global`: collection visibility와 충돌
- Graphify Markdown LLM extraction: provider·credential 경계와 충돌
- Graphify incremental update: 전체 재생성 동등성 실패
- SQLite 정본 승격: Markdown 정본 원칙과 충돌
- 검색용 shipping Skill 추가: 기존 `knowledge-recall` 확장으로 대체 가능

## 수락 기준

- Markdown 전체·증분 graph 정규화 결과 동등
- Graphify 전체 build 두 번의 정규화 결과 동일
- Graphify incremental command 호출 `0건`
- shared·project-private·confidential graph 누출 `0건`
- canonical Markdown·collection registry 변경 `0건`
- 기존 SQLite 직접 사실 질문 결과 저하 `0건`
- Graphify 미설치·손상·schema mismatch의 기존 기능 영향 `0건`
- 상위 Git 저장소 sibling read·write `0건`
- 등록 nested project scan의 전역 Git 설정 mutation `0건`
- expired·revoked 지식의 기본 retrieval `0건`
- graph source digest drift의 current 승격 `0건`
- vector database·embedding runtime product dependency `0건`
- provider API·API key·query log·watcher·Git hook·자동 MCP 등록 `0건`
- Windows x64·macOS arm64·Linux musl 공개 시험 수용

## 범위 결정

이 문서: 적용 후보와 권장안. Nested registered-project scan 수정의 `0.10.0` 편입과
`0.9.6` 미출시는 유지보수자 확정. Graphify 제한 채택·Markdown 관계 graph·기억 생명주기의
전체 `SCP10-001` 승인과 제품 구현은 별도 결정 필요.
