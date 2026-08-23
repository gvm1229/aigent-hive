# 현재 상태

- 기준 branch: `feature/0.10.0@50c7333`
- 원격 기준: `origin/feature/0.10.0` 동기화
- product version: `0.9.5`
- 다음 target: `0.10.0-test`
- 활성 계획: [`PLAN.md`](../plans/PLAN.md)
- 현재 milestone: 관계 graph·Graphify adapter 마감

## 최근 완료

- Agent 지침 단일 정본·경량화 `DIR10-001–007` 완료
- Source `AGENTS.md`: 4,442 byte, 8KiB 예산 통과
- 활성 source directive: 기준 대비 29.5% 축소
- 소비자 `AGENTS.md` router: 기준 대비 80.0% 축소
- Source·소비자 규칙 ownership 대장과 정적 중복·경로·투영 gate 추가
- `hive-render`: preference 유무와 무관한 canonical template 경로 통합
- `user_setup`·`user_install`: 공통 완료·중단·안정판 renderer 통합
- 현재 소비자 projection만 갱신, historical project·user base 변경 `0건`
- Host-owned Skill 예약과 registered nested-project knowledge scan 수정 완료
- Hive-native Markdown 관계 graph의 public command·격리·derived generation 구현
- `verified-workflow` rename·자연어 routing과 `adversarial-judge` 기본 구현 완료
- Continuation closure·bounded Stop hook·중단 3조건 경계 구현 완료
- Codex·Claude·Antigravity continuation adapter·승인형 Stop hook·취소·bounded nudge 수용 완료
- `0.7.0–0.9.5`와 공개 시험 predecessor의 retired Skill direct-jump cleanup·rollback 수용 완료
- Stable Skill compatibility ledger와 npm·GitHub 공개 stable parity 게시 gate 추가
- `hive judge receipt`의 세 host launch·result·identity·verdict binding과 quorum 분리 수용 완료
- Native graph 증분 동등성·FTS planner·metadata·lifecycle·비용 receipt·JSON/HTML export 완료
- Graphify code-only receipt·정규화·atomic activation·native fallback·exact consent gate 완료
- 여섯 graph scope 물리 격리와 graph 전후 canonical Markdown·FTS 무회귀 검증 완료
- Graphify `0.9.47` Windows x64·macOS arm64·Linux musl x64 30-wheel lock과 platform digest binding 완료
- Vector quality +15.0 points·세 engine 50k lookup 통과, 50k embedding build 10분 초과로 `defer`
- Vector engine·embedding runtime product dependency 추가 `0건`

## 현재 검증 근거

- Directive gate: failure `0건`
- Source Wiki: 156 page, error·warning `0건`
- Python lane: documentation 45, security 103, contract 372, integration 84, release 55 통과
- Rust `hive-render` 63, `hive-cli` user setup 46·user install 89, historical upgrade 3 통과
- Rust workspace 전 범위 통과; `hive-update` Windows 파일 잠금 1건은 동일 시험 격리 재실행 통과
- Human documentation style 19 통과·Windows 전용 1건 건너뜀
- Markdown link 5 통과
- GitHub Actions: feature branch push trigger 없음, CI run 생성 `0건`
- Release lane의 macOS·POSIX 전용 8건과 integration의 Windows 권한 필요 symbolic link 12건 건너뜀
- 위 건너뜀은 현재 Windows host의 제한이며 해당 운영체제 수용 근거 아님

## `0.10.0` 남은 범위

- 세 운영체제 30개 관계 질문·성능 공개 수용
- 번호 공개 시험판과 Windows x64·macOS arm64·Linux musl 수용
- `REL10-005–007`: 안정판 후보·게시·설치

## 현재 장애 요인

- Agent 소유 구현을 막는 수동 blocker 없음
- Graphify macOS·Linux·Markdown 의미 추출·50,000 chunk 비용 미검증
- 현재 Windows에서 macOS·Linux 실제 설치 수용 불가
- Claude 설치본 미인증·필수 lifecycle 수정 이전 version
- Antigravity CLI `1.1.18`: `/hooks` 검사 성공, 실제 Stop continuation 시험 미완료
- 안정판 `0.10.0`: 유지보수자의 버전명 포함 명시 승인 전 tag·protected `main` 통합·게시·설치 금지

## 다음 작업

1. `KRG10-016` 30개 관계 질문·세 운영체제 수용 준비
2. `REL10-001–004` 번호 공개 시험판 준비·수용

## 과거 기록

- `0.9.5` 마감: [`0.9.5-closeout.md`](../archive/state/0.9.5-closeout.md)
- 완료·대체 계획: [`Archive`](../archive/README.md)
- 버전 비종속 후보: [`Backlog`](../plans/backlog/README.md)
