# Discord `v0.9` 연결 계획

> Checklist owner: `DIS9-*`
> Target: `0.9.0-test.8`
> Decision: [`ADR-0018`](../../decisions/ADR-0018-notion-wiki-backend.md)

## 목표

- usage guard 중단의 선택형 Discord outbound 알림
- 전역 설정 안의 webhook 환경 변수 안내·검증·시험 알림
- 프로젝트·run·요청 요약·진행 상태를 구분하는 안전한 알림
- README와 release bundle의 접근 가능한 단일 HTML 안내
- credential·원문 prompt·절대 경로 자동 전송 `0건`

## 경계

- 전역 incoming webhook 1개를 프로젝트가 상속
- project별 webhook override와 inbound continuation: 초기 범위 제외
- Codex inbound continuation: 공식 호스트 기능 전까지 `unsupported`
- Claude inbound continuation: 공식 Discord Channel plugin으로 위임
- 원문 prompt: 기본 제외, 명시적 opt-in·직전 preview·redaction 통과 때만 제한적 포함

## Checklist

- [x] [DIS9-001] Discord·Notion integration gap 감사와 위협 모델 기록
  - Evidence: `DNI-001`, `ADR-0018`, `docs/research/discord-notion-host-integrations.md`
- [x] [DIS9-002] `user-setup` catalog·schema에 Discord opt-in·환경 변수 이름·prompt privacy mode의 typed migration 추가
- [x] [DIS9-003] usage guard opt-in 뒤 webhook 생성 안내·환경 변수 이름 검증·단일 시험 알림과 비밀 없는 중단 재개 기록 구현
- [x] [DIS9-004] 재설정의 Discord on/off·환경 변수 교체·`전체 검토|선택 항목 검토|중단한 단계부터 계속` 동작 구현
  - `선택 항목 검토` 시작 시 모든 전역 설정과 각 하위 설정의 짧은 목적을 항목별 한 줄 목록으로 먼저 표시. `사용량 보호` 하위에는 사용 여부·중단 기준·CodexBar 대체 수단·Discord 알림 사용 여부·webhook 환경 변수 이름·요청 공개 범위 포함
- [x] [DIS9-005] usage halt snapshot에 안전한 project identity·run ID·요청 요약·checkpoint reference 결합과 cross-project binding 차단 구현
- [x] [DIS9-006] canonical plan/run 기반 진행 상태 reducer와 분모 없는 작업의 truthful unknown 표현 구현
- [x] [DIS9-007] Discord payload schema·human message에 project·request·progress·resume context, redaction·size limit 구현
  - `interface_language`와 일치하는 English 또는 한국어만 사용. 같은 알림 안의 언어 혼합 금지
  - 사용자가 대화로 선택할 수 있는 안전한 필드·순서 설정: 남은 사용량, 프로젝트, 요청 요약, 진행 상태, host, 재개 안내, 측정 시각·검증 참조
  - 시험 알림: 실제 중단 알림과 같은 renderer·필드·언어 사용. 첫 줄에만 자유롭게 형식 변경을 요청할 수 있다는 현지화된 시험 고지 추가
  - 원문 prompt는 `raw-prompt`를 별도 승인한 경우에만 포함. 기본 `summary`는 요약 또는 비공개 표시 유지
- [x] [DIS9-008] 대화형 setup·README·release bundle HTML 안내와 `hive guide integrations --open` 또는 exact local locator 구현
- [x] [DIS9-009] fake webhook·두 프로젝트 halt·plan-backed/unplanned progress·redaction 회귀와 지원 host E2E 구현
- [ ] [DIS9-010] 독립 numbered test candidate·clean install 수용. npm `latest` 불변과 stable feature parity 확인
- [ ] [DIS9-011] 사용자가 승인한 구역형 Discord Markdown 알림 적용: 시험·실제 공통 renderer에 빈 줄, 이모지·굵은 구역 제목, 사용량·작업 정보·작업 계속 요청 안내 추가. 밑줄 표기 없음

## 완료 기준

- 사용자가 global setup 대화로 Discord 연결·시험 알림 완료
- 사용자가 예를 들어 `한글로 남은 사용량과 프로젝트를 포함해 Discord 알림 형식을 바꿔 주세요`라고 요청하면 안전한 설정 항목으로 반영하고, 같은 형식의 시험 알림 확인
- 서로 다른 두 프로젝트의 중단 알림 식별 가능
- 원문 prompt·credential·절대 경로의 동의 없는 외부 전송 `0건`
- 시험판과 정식판 Discord 기능 차이 `0건`

## 외부 중지 경계

- 실제 Discord server의 incoming webhook 생성·환경 변수 설정
- GitHub·npm numbered test publication authority
