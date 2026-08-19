# Graphify `0.9.47` 도입 가능성

> 조사일: 2026-08-20
> 판정: 제품 도입 중단, 버전 비종속 backlog 이전
> 실행 환경: Windows 11 x64, Python `3.12.0`, 격리 `tests/work/graphify-research/`

## 대상

- Repository: [Graphify-Labs/graphify](https://github.com/Graphify-Labs/graphify)
- Release: [`v0.9.47`](https://github.com/Graphify-Labs/graphify/releases/tag/v0.9.47)
- Package: `graphifyy==0.9.47`
- Wheel SHA-256: `2a8b13ccd53d507d16dcc12aebe488517c369afa547938464474fd3e772938ab`
- License: Apache-2.0

## 유지보수·채택 신호

| 항목 | 확인값 |
| --- | --- |
| Repository 생성 | 2026-04-03 |
| 최근 push·release | 2026-08-19 |
| GitHub stars | 108,286 |
| Forks | 10,510 |
| Open issues·PR | 1,009 |
| 최신 upstream CI | run `32289286798` 성공 |

높은 채택 신호와 매우 빠른 `0.9.x` 변경 속도의 동시 존재. 안정된 adapter schema 근거로 사용 금지.

## 설치·Dependency

- Windows Python `3.12.0` 격리 설치 성공
- 기본 설치: Graphify·NetworkX·NumPy·RapidFuzz·tree-sitter와 25개 언어 grammar, 총 30 package
- 확인 license: Apache-2.0, BSD-3-Clause, MIT, 0BSD, Zlib, CC0-1.0
- Upstream CI: Linux Python 3.10·3.12와 security scan 성공
- macOS·Windows upstream 설치 CI: 확인 불가
- 격리 venv 기본 pip `23.2.1`: `pip-audit` advisory 8건
- Graphify와 기본 runtime dependency: audit finding `0건`

기본 pip 취약점은 안전한 installer의 pip `>=26.1.2` 고정으로 회피 가능. 현재 upstream 설치 안내만으로 Hive sidecar 하드 게이트 충족 불가.

## 구조 graph 반복성

Synthetic Rust corpus:

- 전체 build 1: `9 nodes`, `11 edges`, SHA-256 `cbc88d90b2c72c0ec72212d1f359a17fc12d56a0c830ba6979579e18a9ed5e4e`
- 전체 build 2: 같은 node·edge·SHA-256
- 결과: 구조 전체 build의 byte 결정성 확인

## Query

질문: `what connects main to Store?`

- BFS depth 2: 8 nodes
- `main() → persist() → Store` 경로 반환
- `EXTRACTED`·`INFERRED` 표시와 `file:line` locator 제공
- Windows cold CLI 20회: min `287.98ms`, median `309.40ms`, p95 `375.84ms`, max `423.39ms`
- `GRAPHIFY_QUERY_LOG_DISABLE=1`: 사용자 cache query log 생성 `0건`

작은 code graph의 query 성능 기준 통과. 50,000 chunk global knowledge 성능의 대체 근거 아님.

## 증분 갱신 실패

한 Rust 함수 추가 뒤 비교:

| 경로 | Nodes | Edges | SHA-256 |
| --- | ---: | ---: | --- |
| `graphify update` | 12 | 15 | `acd2462494e2f9933e592620e4f672ad671223bdbe68f76cffd8add7a0931129` |
| 전체 `extract --force --code-only` | 10 | 14 | `e0fcc1475382de8565f305b30187fe491bf99c025edde991897c22e35c7292f9` |

차이:

- 증분 경로의 README node 2개 추가
- 증분 source locator: scan root 포함 경로
- 전체 source locator: corpus 상대 경로
- 같은 canonical corpus의 graph 동등성 실패

판정: `GPH10-004` 하드 게이트 실패.

## Global knowledge 격리

Upstream `global` 동작:

- 단일 `~/.graphify/global-graph.json`
- repository tag 기반 node prefix
- source 없는 external node의 label 기반 병합
- Hive collection visibility·confidential authorization 계약 없음

판정: shared·project-private·confidential collection의 존재·label·edge 격리 미지원. Upstream `global` command 사용 금지.

## Markdown 의미 추출

- `--code-only`: provider API·API key 없이 성공
- Markdown 의미 추출: `gemini|kimi|claude|openai|deepseek|ollama` backend 또는 host agent 작업 필요
- Hive provider API·credential 경계 안의 adapter 부재
- 품질·비용·private collection 격리 검증 미실행

판정: source·project·global 각 10개 의미 질문의 수용 근거 부재.

## 관계 질문 기준 30개

### Hive source

1. CLI knowledge action과 owning Rust module의 연결
2. user setup schema와 renderer·fixture의 연결
3. release workflow와 compatibility report의 연결
4. usage guard control과 session binding의 연결
5. Source Wiki fact와 source digest verifier의 연결
6. project upgrade와 historical base registry의 연결
7. Skill catalog와 세 host projection의 연결
8. run checkpoint와 evidence predicate의 연결
9. Judge receipt와 external trust root의 연결
10. changed crate와 영향 받는 Python lane의 연결

### Consumer project

1. `AGENTS.md` marker와 Hive ownership manifest의 연결
2. setup answer와 생성된 directive의 연결
3. selected host와 설치된 plugin projection의 연결
4. local Skill edit와 upgrade merge 결과의 연결
5. project collection과 user-root index의 연결
6. usage threshold와 effective policy의 연결
7. run plan과 checkpoint·handoff의 연결
8. update owner와 binary activation의 연결
9. foreign byte와 conflict result의 연결
10. disabled Wiki와 제거되는 파생 파일의 연결

### Global knowledge

1. user-root fact와 source project의 연결
2. shared fact와 reusable claim의 연결
3. project-private claim과 owning collection의 연결
4. confidential claim과 authorization의 연결
5. superseded claim과 replacement의 연결
6. contradiction과 current-truth winner의 연결
7. imported detached collection과 local mapping의 연결
8. bundle entry와 canonical Markdown의 연결
9. stale index와 rebuild source의 연결
10. 한 project 결정과 다른 project의 명시적 retrieval 연결

질문 목록 작성 완료. 제품 adapter 부재와 격리 실패로 source·project·global 전체 수용 실행 중단.

## 최종 판정

- 전체 구조 build: 가능
- 작은 code graph query: 가능
- 증분 동등성: 실패
- global visibility 격리: 미지원
- Markdown host-owned 의미 추출: 미검증
- macOS·Linux·50,000 chunk 수용: 미검증
- `0.10.0` 제품 도입: 중단
- 다음 경로: [`graphify-knowledge-graph.md`](../plans/backlog/graphify-knowledge-graph.md)
