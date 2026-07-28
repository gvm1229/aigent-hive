# ADR-0011: Source Wiki 독립성

- 상태: accepted
- 날짜: 2026-07-27
- 범위: Aigent Hive source workspace의 durable LLM knowledge

## 결정

Source Wiki 정본:

- English: `llm-wiki/en/`
- Korean: `llm-wiki/ko/`
- Derived index: ignored `.agents/work/source-wiki/index.sqlite3`
- Coordination marker: ignored persistent `.agents/work/source-wiki/.index.lock`
- 문서 계약: exact bilingual pair, reciprocal counterpart, reviewed source locator와 digest
- 재구축 계약: tracked Markdown만으로 model call·network 없이 SQLite index 재생성

Source workspace의 `omx_wiki/`, `.omx/wiki/`, `.hive/knowledge/` 사용 금지.

## Agent-reviewed task fact autocapture

Default-on source Wiki의 completion 계약:

- Material source task 종료 전 reusable outcome·artifact·decision·workflow·criterion 판정
- Current authorized task와 reviewed local artifact만 source
- 기본 기록: outcome, tool 또는 external project, creation·acceptance criteria,
  bounded originating request summary
- Exact request: 사용자 retention intent와 credential·confidentiality·private-path review
  이후에만 허용
- External artifact: source corpus import 금지, tracked safe handoff의 Hive-relevant fact만 인용
- 동일 fact: no-op 또는 current-truth pair 갱신
- English·Korean pair 동시 기록과 explicit index rebuild

금지 자동화:

- Raw transcript·complete conversation·hidden prompt capture
- Hook payload·tool output·cache·database·runtime state ingestion
- `UserPromptSubmit`, `Stop`, `PostToolUse` 기반 memory recorder
- Consumer knowledge 또는 installed state의 source corpus import

Autocapture 의미: agent-reviewed completion step. Background watcher·hook ingestion 의미 없음.

Index publication 계약:

- `.index.lock`: 정본·index data가 아닌 persistent coordination marker
- Writer serialization: persistent marker의 exclusive OS advisory lock
- Reader serialization: `lint`·`query`의 shared OS advisory lock과 writer 완료 대기
- Reader visibility: bounded read·in-memory 검증 종료까지 shared lock 유지, in-flight
  claim gap 관찰 0건
- SQLite build: ambient target SQLite path 없는 in-memory 생성
- Verification: serialized bytes의 in-memory deserialize와 logical digest 대조
- Publication: pinned source-root capability 내부 recoverable two-phase CAS
- CAS phase 1: expected live identity 검증 뒤 unique Hive-owned claim으로 이동
- CAS phase 2: synced temporary bytes를 live index로 이동한 뒤 exact prior claim 정리
- Crash residue: missing live index와 exact Hive-owned orphan claim·temporary 가능
- Recovery authority: 다음 explicit rebuild만 canonical Markdown에서 index 재생성 후 exact
  regular Hive-owned claim·temporary path 정리
- Reader failure: missing·stale·corrupt·crash-interrupted index에서 `lint` finding 또는
  `query` fail-closed, implicit repair 0건
- Recovery: lock marker 보존, disposable `index.sqlite3` 삭제·재생성

## OMX Wiki Skill 제외 이유

제외 판단은 OMX Wiki의 품질·유용성과 무관.

기억할 의도:

- 현재 source 개발 orchestration과 실행 보조에는 OMX/OMC 적극 활용
- OMX Wiki Skill 제외
- Durable source knowledge의 소유권은 Aigent Hive에 유지
- Durable source knowledge의 수명: 교체 가능한 orchestration 도구보다 장기
- OMX Wiki 경로·명령·설정 lifecycle의 정본 계약 채택 금지
- 향후 OMX/OMC retirement 조건: 도구 교체만 필요, knowledge migration 0건

제외 근거:

- 고정 저장 경로 `omx_wiki/`
- 고정 명령 surface `omx wiki`
- `.omx-config.json` 기반 lifecycle·auto-capture 계약
- Durable source knowledge와 replaceable compatibility layer 사이의 불필요한 결합

장기 방향:

- Provider-neutral 또는 host-native 대체 capability 확보
- 이후 OMX/OMC compatibility dependency 단계적 제거
- 제거 시 source knowledge 경로·schema·pair identity·index authority migration 0건

성공 기준: OMX/OMC retirement 시 source knowledge migration 0건.

## 재사용 경계

재사용 대상:

- `hive-wiki`의 Markdown parser, lint, SQLite index, query core
- Consumer knowledge capture·maintenance·query의 secret 검증, review, current-truth 원칙

재사용 제외:

- Installed consumer `.hive/` layout
- Consumer runtime·approval·knowledge state
- OMX/OMC storage namespace와 lifecycle ownership

현재 OMX/OMC 역할: source 개발 orchestration 보조.

## 결과

- Source knowledge의 provider·orchestrator 독립성
- English·Korean 동시 검토와 source digest 추적
- SQLite 손실·corruption 후 pinned capability 기반 deterministic rebuild
- Persistent shared/exclusive OS advisory lock marker와 disposable index lifecycle 분리
- Recoverable two-phase CAS와 explicit-rebuild-only orphan cleanup
- Consumer product와 source knowledge의 ownership 분리
- OMX/OMC 교체 시 durable knowledge 이동 비용 제거
