# 한국어 언어 core `0.10.0`

> Checklist owner: `KOR10-*`
> Upstream: [`epoko77-ai/im-not-ai`](https://github.com/epoko77-ai/im-not-ai)
> 연구 기준: `im-not-ai 2.3.2@0ac1e84`
> 목표: Hive가 만드는 한국어 응답·문서에 자연스러운 후편집 논리를 기본 적용하고, 기존 글 윤문에는 `humanize-kor`를 명시적으로 제공

## 제품 계약

- 자동 한국어 core와 `humanize-kor` Skill은 같은 rule pack·profile·검사 engine을 사용함
- 일반 한국어 생성에는 작고 안전한 생성 규칙을 항상 적용하고, 완성 초안에는 결정적 검사와 필요한 구간의 국소 재작성을 적용함
- `humanize-kor`는 사용자가 붙여넣거나 지정한 기존 한국어 글을 빠르게 윤문하는 명시적 진입점이며, 일반 응답의 자동 적용을 대신하지 않음
- 원문·직접 인용·코드·명령·경로·URL·수치·날짜·단위·version·고유명사를 보호함
- 정확한 원본이 어색한 윤문본보다 우선하며, 검사 실패 때 부정확한 후보를 채택하지 않음
- Hive는 provider API·credential·model process를 소유하지 않으며, 실제 진단·재작성은 활성 host가 수행함

## 적용 profile

| Profile | 대상 | 보존 경계 |
| --- | --- | --- |
| `response` | 일반 한국어 응답 | 쉬운 말·구체적 의도·필요한 예시, 짧은 응답의 과윤문 금지 |
| `release-note` | Discord·release 요약 | main list와 example sublist, version·명령·Skill ID 보존 |
| `documentation` | README·guide·Wiki | 제목·링크·계약·경고·사실 보존 |
| `technical` | CLI·오류·schema 설명 | code·field·경로·정확한 화면 문구 byte 보존 |
| `verbatim` | 인용·법률·사용자 원문 요청 | 검사만 수행하고 재작성 금지 |

## Host 적용 수준

- Codex: 항상 적용되는 생성 정책과 final self-review. 공식 final-response replacement hook이 없으면 그 한계를 명시함
- Claude: `Stop`에서 `last_assistant_message`를 검사하고, 실패 시 bounded rewrite를 요청함. Hook이 응답을 직접 바꾼다고 표현하지 않음
- Antigravity·Gemini CLI: `AfterAgent` final 검사와 bounded retry를 사용함. Streaming `AfterModel` chunk 교체는 문서 전체 빈도 검사에 사용하지 않음
- Hive-owned 문서·공지·CLI 문자열: host와 무관한 결정적 gate를 commit·게시 전에 적용함

## Watermark·출처 계약

- 자연스러운 한국어 작성과 검증 가능한 출처 은폐는 다른 목적임
- 통계적 watermark나 탐지기 회피율을 측정·최적화·광고하지 않음
- 숨은 문자 삽입·후보 단어 교란·반복 재작성으로 detector를 속이는 기능을 만들지 않음
- 출처 표시·인용·저자·기관·링크·AI 사용 고지처럼 원문에 존재하는 provenance를 삭제하거나 거짓 인간 작성 주장을 추가하지 않음
- 사용자가 detector 우회나 의무 고지 회피를 요청하면 그 목적은 수행하지 않고, 출처와 의미를 보존하는 일반 문체 개선만 제안함
- `sanitize`의 zero-width·bidi·NFC 처리는 text hygiene이며 통계적 watermark 제거로 표현하지 않음
- 정상적인 윤문이 부수적으로 탐지 점수를 바꿀 수 있음을 인정하되, 그 변화는 성공 기준이나 품질 지표가 아님

## Upstream update 계약

- Floating `main`, upstream `install.sh|update.sh`, symlink, cron, 자동 `git pull`을 제품에서 실행하지 않음
- Upstream version·commit·tree digest·license digest·선별 파일 digest·Hive 변환 version을 manifest에 고정함
- `rules-data`: taxonomy·quick rules·diagnosis rules·baseline·profile 자료. 격리 build와 corpus gate 통과 뒤 독립 language pack 후보로 승격 가능함
- `engine-code`: metrics·sanitize·gate·chunking 코드. Rust·Python·보안·세 운영체제 검증과 번호 시험판이 필요함
- `host-surface`: Skill·agent·hook·manifest 변경. 세 host capability·projection·upgrade 수용이 필요함
- Upstream check는 update 가능성만 보고하고, staging·검증·preview·명시 승인 뒤 원자 활성화함
- 활성 pack과 이전 pack을 함께 보존하고 실패·schema mismatch·품질 저하 때 즉시 rollback함
- Upstream 삭제·force-push·license 변경은 기존 검증 pack의 사용 권한과 digest 기록을 제거하지 않음

## Checklist

- [x] [KOR10-001] 자동 한국어 core·명시적 `humanize-kor`·지속 가능한 upstream update를 `0.10.0` 제품 범위로 유지보수자 승인
- [ ] [KOR10-002] `im-not-ai 2.3.2@0ac1e84`의 MIT license·source inventory·symlink·host version drift·retired reference·runtime 경계를 provenance manifest로 고정
- [ ] [KOR10-003] `response|release-note|documentation|technical|verbatim` profile과 보호 span·예시 sublist·쉬운 설명 계약 정의
- [ ] [KOR10-004] 번역투·반복·대구·피동·명사화·상투구·리듬 검사와 change rate·touch rate·서법·수치·인용·링크 보존을 Hive 결정적 core로 구현
- [ ] [KOR10-005] 한국어 응답·문서 생성의 자동 draft→inspect→국소 rewrite→verify 경로와 정확한 원본 fallback 구현
- [ ] [KOR10-006] Codex instruction/self-review, Claude Stop validate-retry, Antigravity AfterAgent validate-retry의 capability·consent·bounded loop adapter 구현
- [ ] [KOR10-007] Hive-owned CLI 문자열·guide·Wiki·Discord subscriber payload의 profile별 정적 gate와 diff receipt 구현
- [ ] [KOR10-008] 기존 한국어 text·file을 명시적으로 윤문하는 `humanize-kor` Skill과 light·standard·heavy·redo·원본 보존·preview 계약 구현
- [ ] [KOR10-009] Watermark 우회·detector 최적화·출처 삭제·거짓 저자 표시 금지와 text hygiene 허용 경계를 schema·Skill·projection·회귀 시험에 적용
- [ ] [KOR10-010] Upstream version·commit·digest·license·변환 version을 기록하고 `rules-data|engine-code|host-surface`를 분류하는 update check·preview manifest 구현
- [ ] [KOR10-011] 검증된 language pack의 staging·원자 활성화·history·rollback과 raw upstream install·floating update 차단 구현
- [ ] [KOR10-012] 한국어 gold corpus·blind 평가·의미·수치·명령·list·link·인용 무회귀·세 host·세 운영체제·direct upgrade·rollback 수용

## 출시 경계

- `0.10.0-test.1`은 이 기능을 포함하지 않으므로 안정판 수용 근거로 재사용하지 않음
- 구현 뒤 `0.10.0-test.2` 이상을 새 product bytes로 게시·설치·세 운영체제 수용함
- 안정판 `0.10.0`은 새 번호 시험판 수용과 유지보수자의 버전명 포함 명시 승인 전 계속 금지함
