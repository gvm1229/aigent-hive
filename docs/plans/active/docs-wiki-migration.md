# `docs/` Wiki 전환 계획

> Checklist owner: `DWK-*`
> Decision: [`ADR-0014`](../../decisions/ADR-0014-docs-wiki-architecture.md)
> 기준 자료: 간소화 직전 `README.md`, 현재 `docs/`, DuckSoul Obsidian Wiki 구조

## 목표

- 간결한 root README 유지
- 기존 README의 유효한 knowledge 전부를 `docs/`에 보존
- `docs/`에 home·index·topic MOC·atomic fact 계층 적용
- 별도 `llm-wiki/` 제거
- Source Wiki CLI·Skill·SQLite를 `docs/facts/` 정본에 연결
- 유효한 knowledge의 삭제 대신 이동을 AI 편집 기본값으로 고정

## 구조

```text
docs/
├── 00-home.md
├── 01-index.md
├── architecture/
├── decisions/
├── guides/
├── research/
├── plans/
├── state/
├── readme/
└── facts/
    ├── README.md
    ├── en/
    └── ko/
```

각 topic directory의 `README.md`: 범위·정본·주요 문서·관련 topic 안내.
`docs/facts/{en,ko}`: exact bilingual pair와 primary fact 1개.

## README knowledge 이동 표

| 기존 README 범위 | 대상 |
| --- | --- |
| 지원 범위·핵심 원칙·제품 기능 | `docs/overview/` topic document + atomic product fact |
| 아키텍처·저장소 구조 | 기존 `docs/architecture/` 보강 |
| 기술 stack·dependency | `docs/guides/development.md`와 dependency fact |
| 개발·검증 command | `docs/guides/development.md` |
| Source Wiki | `docs/00-home.md`, `docs/facts/README.md`, ADR-0014 |
| Source usage safeguard | 기존 `docs/guides/source-usage-guard.md` |
| User onboarding·project harness | onboarding guide + atomic lifecycle fact |
| Canonical knowledge·index | architecture document + atomic storage fact |
| Subscription usage guard | usage guide + atomic policy fact |
| Clean-context judge | 기존 architecture·guide |
| Signed release·update | 기존 architecture·guide |
| Git workflow | 기존 branching·commit guide |
| 상태·version·license | CURRENT, ADR-0006, licensing |

## Checklist

- [x] [DWK-001] Source AI directive에 valid knowledge 이동 우선, deprecated knowledge
  제거 예외, 간소화 전 inventory·replacement 확인 completion gate 추가
- [ ] [DWK-002] `docs/00-home.md`·`docs/01-index.md`·topic MOC와 old README knowledge
  mapping을 구현하고 누락 지식을 topic document로 복원
- [ ] [DWK-003] 기존 13개 bilingual page를 primary fact 1개 단위의
  `docs/facts/{en,ko}` exact pair로 분할·이동하고 cross-link·source digest 갱신
- [ ] [DWK-004] Source Wiki CLI·Skill·directive·test·state·architecture 경로를
  `docs/facts/{en,ko}`로 전환하고 `llm-wiki/` 제거, lint·index·query·clean-copy 검증

## 완료 기준

- 간소화 직전 README heading별 disposition 100%
- 유효한 knowledge의 대체 locator 존재
- `docs/00-home.md`에서 모든 top-level topic 접근 가능
- `docs/01-index.md`의 tracked Markdown 누락 0건
- Fact page별 primary fact 1개, unrelated section 0건
- English·Korean exact pair와 reciprocal link 100%
- `llm-wiki` tracked path·canonical reference 0건
- Source Wiki lint finding·warning 0건
- English·Korean query와 index 삭제 뒤 explicit rebuild equivalence PASS
