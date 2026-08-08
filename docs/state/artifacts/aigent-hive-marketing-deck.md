# Aigent Hive marketing deck

## Outcome

- 60분 발표용 deck과 presenter notes 생성 완료
- Slide 수: 91
- 구성: 1부 기능·사용법 58장, 2부 설치 14장, 3부 설계·원리 19장
- 내용 기준: `0.9.0-test.5`, 공개 short Skill name 22개
- 제작 도구: LumaDeck
- LumaDeck repository: `~/Documents/WebProjects/luma-deck`
- Deck project: `projects/aigent-hive-overview/`
- Editable source: `projects/aigent-hive-overview/slides.md`
- Presenter notes: `projects/aigent-hive-overview/PRESENTER_NOTES.md`
- Style source: `projects/aigent-hive-overview/styles/index.css`
- Deck metadata: `projects/aigent-hive-overview/deck.json`
- Validated HTML build:
  `artifacts/aigent-hive-overview/html/0.9.0-test.5-20260808/index.html`
- Logo: honeycomb `H` placeholder; final logo design은 후속 작업
- 검증: Slidev production build, 1280×720 전체 91장 overflow 검사, 대표 화면 육안 검수
- 비차단 경고: dependency `@vueuse/core`의 Rolldown pure-annotation 경고 2건

## Creation criteria

Current presentation targets:

1. 30분 기능·사용법: 특장점과 `AGENTS.md` 단독 구성 대비, core 기능과 공개 Skill별 설명·직후 예시
2. 10분 설치: README 기반 exact test version 설치와 setup/configure 질문·예시 답변·행동 차이
3. 20분 설계·원리: canonical state, projection, transaction, routing, knowledge, run, update, 검증 구조
4. 개발 convention: 저장소 정본 계획, concern 단위 workflow, ADR lifecycle, REL·SIL·TST·NAT·MRA·DNI prefix

Supporting constraints:

- Local, provider-neutral, user-owned positioning
- Codex·Claude Code·Antigravity host coverage
- Global Hive → project harness → host discovery structure
- Markdown canonical knowledge and user-root disposable SQLite index distinction
- Automatic onboarding, safe upgrade, usage guard, prompt refinement, narrow loading
- Product explanation rather than implementation-history chronology
- Apple WWDC형 image-centric 구성과 formal-casual 한국어 발표 톤
- Primary white, complementary honey gold, 게임 고유 배경·색상 제외
- Slide당 한 가지 주장과 발표 대본 중심의 낮은 화면 텍스트 밀도

## Initial request

Original request with the user-specific absolute path normalized to
`~/Documents/WebProjects/luma-deck`:

```text
Take a look at my ppt solution at ~/Documents/WebProjects/luma-deck
Try to automatically install a consumer product harness from our hive (automatically infer the onboarding questions) first. Come to think of it, an automated onboarding is nice. Create a skill for that too. The skill will involve inheriting the global preferences, reading the project source and ask only the truly necessary questions that cannot be automatically resolved. If the global hive preferences is inherited, then the only question left standing would be the project's purpose or goal, but if the README or the AGENTS.md of that project is detailed enough, that question's answer can also be inferred. So the auto onboarding should work.
Then create a deck project that concisely explains:
1. What our hive harness is about
2. Who hive is for (user target)
3. What feature it provides
4. How it's structured and how it functions
5. What kind of optimization strategy it uses
```

## Continuation

1. `~/Documents/WebProjects/luma-deck` 이동
2. `projects/aigent-hive-overview/slides.md`와 `styles/index.css` 수정
3. LumaDeck repository의 `AGENTS.md`와 project harness directive 우선 확인
4. Slidev build·전체 slide overflow 검증·대표 화면 육안 검수
5. 이 source record와 `docs/facts/{en,ko}/marketing-deck-record.md` current truth 동시 갱신
6. 최종 logo가 정해지면 `components/HoneyMark.vue`와 관련 visual만 교체
