# 한국어 언어 core `0.10.0`

> Checklist owner: `KOR10-*`
> Upstream: [`epoko77-ai/im-not-ai`](https://github.com/epoko77-ai/im-not-ai)
> 연구 기준: `im-not-ai 2.3.2@0ac1e84`
> 목표: Hive가 만드는 한국어 응답·문서에 자연스러운 후편집 논리를 기본 적용하고, 기존 글 윤문에는 `humanize-kor`를 명시적으로 제공

## 제품 계약

- 자동 한국어 core와 `humanize-kor` Skill: 동일한 rule pack·profile·검사 engine 사용
- 일반 한국어 생성: 작고 안전한 생성 규칙의 상시 적용과 완성 초안의 결정적 검사·필요 구간 국소 재작성
- `humanize-kor`: 사용자가 붙여넣거나 지정한 기존 한국어 글을 빠르게 윤문하는 명시적 진입점. 일반 응답 자동 적용의 대체 수단 아님
- 보호 대상: 원문·직접 인용·코드·명령·경로·URL·수치·날짜·단위·version·고유명사
- 우선순위: 정확한 원본이 어색한 윤문본보다 우선. 검사 실패 후보 채택 금지
- 실행 경계: Hive의 provider API·credential·model process 소유 금지. 실제 진단·재작성은 활성 host 소유

## 적용 profile

| Profile | 대상 | 보존 경계 |
| --- | --- | --- |
| `response` | 일반 한국어 응답 | 쉬운 말·구체적 의도·필요한 예시, 짧은 응답의 과윤문 금지 |
| `release-note` | Discord·release 요약 | main list와 example sublist, version·명령·Skill ID 보존 |
| `documentation` | README·guide·Wiki | 제목·링크·계약·경고·사실 보존 |
| `technical` | CLI·오류·schema 설명 | code·field·경로·정확한 화면 문구 byte 보존 |
| `verbatim` | 인용·법률·사용자 원문 요청 | 검사만 수행하고 재작성 금지 |

## Host 적용 수준

- Codex: 상시 생성 정책과 final self-review. 공식 final-response replacement hook 부재 시 한계 명시
- Claude: `Stop`에서 `last_assistant_message` 검사, 실패 시 bounded rewrite 요청. Hook의 직접 응답 교체 표현 금지
- Antigravity·Gemini CLI: `AfterAgent` final 검사와 bounded retry 사용. Streaming `AfterModel` chunk 교체의 문서 전체 빈도 검사 사용 금지
- Hive-owned 문서·공지·CLI 문자열: host와 무관한 결정적 gate를 commit·게시 전에 적용

## Watermark·출처 계약

- 자연스러운 한국어 작성과 검증 가능한 출처 은폐의 목적 분리
- 통계적 watermark나 탐지기 회피율의 측정·최적화·광고 금지
- 숨은 문자 삽입·후보 단어 교란·반복 재작성으로 detector를 속이는 기능 금지
- 출처 표시·인용·저자·기관·링크·AI 사용 고지처럼 원문에 존재하는 provenance 삭제와 거짓 인간 작성 주장 추가 금지
- 사용자의 detector 우회나 의무 고지 회피 요청: 해당 목적 수행 금지, 출처와 의미를 보존하는 일반 문체 개선만 제안
- `sanitize`의 zero-width·bidi·NFC 처리: text hygiene. 통계적 watermark 제거 표현 금지
- 정상적인 윤문에 따른 탐지 점수 변화: 부수 효과일 수 있으나 성공 기준·품질 지표에서 제외

## Upstream update 계약

- 제품 실행 금지: floating `main`, upstream `install.sh|update.sh`, symlink, cron, 자동 `git pull`
- Manifest 고정 항목: upstream version·commit·tree digest·license digest·선별 파일 digest·Hive 변환 version
- `rules-data`: taxonomy·quick rules·diagnosis rules·baseline·profile 자료. 격리 build와 corpus gate 통과 뒤 독립 language pack 후보 승격 가능
- `engine-code`: metrics·sanitize·gate·chunking 코드. Rust·Python·보안·세 운영체제 검증과 번호 시험판 필요
- `host-surface`: Skill·agent·hook·manifest 변경. 세 host capability·projection·upgrade 수용 필요
- Upstream check: update 가능성만 보고. staging·검증·preview·명시 승인 뒤 원자 활성화
- 활성 pack과 이전 pack 동시 보존. 실패·schema mismatch·품질 저하 때 즉시 rollback
- Upstream 삭제·force-push·license 변경 시에도 기존 검증 pack의 사용 권한과 digest 기록 유지

## Checklist

- [x] [KOR10-001] 자동 한국어 core·명시적 `humanize-kor`·지속 가능한 upstream update를 `0.10.0` 제품 범위로 유지보수자 승인
- [x] [KOR10-002] `im-not-ai 2.3.2@0ac1e84` MIT license·111-file tree·symlink `0건`·host version drift·retired agent·runtime 경계를 provenance manifest로 고정 — `eaed3203`
- [x] [KOR10-003] `response|release-note|documentation|technical|verbatim` profile·보호 span·예시 sublist·쉬운 설명 계약 정의
- [x] [KOR10-004] 번역투·반복·대구·피동·명사화·상투구·리듬 검사와 bigram change rate·touch rate·서법·수치·인용·링크 보존 Rust core 구현
- [x] [KOR10-005] 자동 draft→inspect→host-owned 국소 rewrite→verify와 exact draft fallback directive·CLI 구현
- [x] [KOR10-006] Codex self-review, Claude `Stop`, Antigravity `AfterAgent`의 fresh capability·exact consent·retry 1회 adapter 구현, 미검증 event instruction-only
- [x] [KOR10-007] Hive-owned text용 profile 정적 gate·digest receipt와 한국어 gold corpus 구현
- [x] [KOR10-008] `humanize-kor` Skill·light/standard/heavy/redo·원본 보존·preview·projection parity 구현
- [x] [KOR10-009] Watermark·detector·출처·거짓 저자 금지와 zero-width·bidi·Hangul NFD hygiene 경계를 schema·Skill·회귀 시험에 적용
- [x] [KOR10-010] Upstream version check와 commit·tree·license·선별 source·변환 version·세 class preview manifest 구현 — live check current/latest `2.3.2`
- [x] [KOR10-011] 세 파일 한정 staging·generation·atomic pointer·history rollback, raw install·floating ref·자동 update 차단 구현
- [ ] [KOR10-012] 한국어 gold corpus·blind 평가·의미·수치·명령·list·link·인용 무회귀·세 host·세 운영체제·direct upgrade·rollback 수용

## 출시 경계

- `0.10.0-test.1`: 이 기능 제외. 안정판 수용 근거로 재사용 불가
- 구현 뒤 `0.10.0-test.2` 이상을 새 product bytes로 게시·설치·세 운영체제 수용
- 안정판 `0.10.0`: 새 번호 시험판 수용과 유지보수자의 버전명 포함 명시 승인 전 계속 금지
