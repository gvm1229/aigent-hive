# 현재 상태

- 기준 branch: `feature/0.10.0@50c7333`
- 원격 기준: `origin/feature/0.10.0` 동기화
- product version: `0.9.5`
- 다음 target: `0.10.0-test`
- 활성 계획: [`PLAN.md`](../plans/PLAN.md)
- 현재 milestone: Adversarial Judge 세 host 수용

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

- Adversarial Judge의 three-host launch·quorum 수용
- 관계 graph·Graphify code-only adapter의 남은 hard gate와 upgrade·rollback 검증
- FTS·vector·graph 품질 기준선과 vector engine adopt|defer 판정
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

1. `JDG10` adversarial Judge three-host 수용 마감
2. `KRG10`, `VEC10` 관계·vector gate와 조건부 구현 마감
3. `REL10-001–004` 번호 공개 시험판 준비·수용

## 과거 기록

- `0.9.5` 마감: [`0.9.5-closeout.md`](../archive/state/0.9.5-closeout.md)
- 완료·대체 계획: [`Archive`](../archive/README.md)
- 버전 비종속 후보: [`Backlog`](../plans/backlog/README.md)
