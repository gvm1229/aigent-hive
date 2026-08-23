# 현재 상태

- 기준 branch: `feature/0.10.0@18b6037`
- 원격 기준: `origin/feature/0.10.0` 동기화
- product version: `0.10.0`
- stable baseline: `0.9.5`
- 다음 target: `0.10.0-test.2`
- 활성 계획: [`PLAN.md`](../plans/PLAN.md)
- 현재 milestone: 한국어 언어 core·`humanize-kor`·upstream update 구현

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
- `hive source-wiki graph` 공개 명령과 source 전용 파생 경로·FTS→관계 edge 결합 완료
- Windows source graph 직접 사실 30/30·근거 관계 30/30·cold CLI p95 `429.8227ms`·canonical 변경 0건
- Vector quality +15.0 points·세 engine 50k lookup 통과, 50k embedding build 10분 초과로 `defer`
- Vector engine·embedding runtime product dependency 추가 `0건`
- `REL10-001–002` version·release metadata와 전체 local gate 마감
- CI `32633357924`: Linux 전체·다섯 Python lane·Windows·macOS protected gate 통과
- Candidate `32633724977`: 다섯 target package·attestation·graph 자격·0.9.1–0.9.5 project coverage 통과
- Publication `32634206001`: `0.10.0-test.1` 게시, npm `latest=0.9.5` 불변
- 현재 Windows: CLI `0.10.0-test.1` 설치와 Codex 사용자 projection `0.10.0` validate 통과
- Disposable verified workflow 수용: `f050bb65`; 자연어 routing·run 생성·의도적 실패/재시도·독립 Judge receipt·새 process/session 복구·사용자 취소 6단계 통과
- 유지보수자 결정: `im-not-ai` 논리를 Skill 선택에 의존하지 않는 자동 한국어 core로 적용하고, 기존 글 윤문에는 명시적 `humanize-kor`를 제공하며, 검증된 upstream update를 `0.10.0` 범위에 포함
- 범위 변경 판정: `0.10.0-test.1`은 이전 범위의 유효한 기록이지만 새 안정판 수용 근거가 아니며 `0.10.0-test.2` 이상 필요

## 현재 검증 근거

- Directive gate: failure `0건`
- Source Wiki: 158 page, error·warning `0건`
- Python lane: documentation 45, security 103, contract 380, integration 84, release 58 통과
- Rust `hive-render` 63, `hive-cli` user setup 46·user install 89, historical upgrade 3 통과
- Rust workspace `--all-targets --all-features` 전 범위와 Clippy `-D warnings` 통과
- Human documentation style 19 통과·Windows 전용 1건 건너뜀
- Markdown link 5 통과
- Verified workflow 단일 수용 영수증 `sha256:a2fefa0da9027582bcfbcdc44da5dae33d22491bcb6c323cee78bcc0b0e81169`, provider process·stable release 실행 `0건`
- Routing·세 host Judge receipt·취소 closure Python 집중 회귀 3건 통과
- Rust retry policy와 canonical loop recovery 집중 회귀는 Windows GNU target에서 각 1건 통과
- 현재 shell에는 MSVC `link.exe`가 없어 fresh MSVC 재빌드는 실행하지 못함. 기존 Windows CLI 공개 수용과 GNU 집중 회귀를 대체 근거로 사용했으며, 이번 실행이 새 MSVC build를 증명하지는 않음
- GitHub Actions: feature branch push trigger 없음, CI run 생성 `0건`
- Release lane의 macOS·POSIX 전용 8건과 integration의 Windows 권한 필요 symbolic link 12건 건너뜀
- 위 건너뜀은 현재 Windows host의 제한이며 해당 운영체제 수용 근거 아님

## `0.10.0` 남은 범위

- `KOR10-002–012`: 자동 한국어 core·profile·결정적 gate·세 host adapter·`humanize-kor`·upstream pack update·rollback·수용
- `REL10-001–004`: 새 product metadata·전체 gate·`0.10.0-test.2` 이상·세 운영체제 수용
- `REL10-005–007`: 안정판 후보·게시·설치·명시 승인

## 현재 장애 요인

- 안정판 제외 범위에 한국어 core 구현·시험·번호 공개 시험판 Agent 소유 작업이 존재
- 안정판 `0.10.0`: 유지보수자의 버전명 포함 명시 승인 전 tag·protected `main` 통합·게시·설치 금지

## 다음 작업

1. `KOR10-002–011` 한국어 core·`humanize-kor`·upstream update 구현
2. `KOR10-012`, `REL10-001–004` 전체 gate와 `0.10.0-test.2` 이상 공개 수용
3. 명시적 `0.10.0` 안정판 승인 전 `REL10-005–007` 시작 금지

## 과거 기록

- `0.9.5` 마감: [`0.9.5-closeout.md`](../archive/state/0.9.5-closeout.md)
- 완료·대체 계획: [`Archive`](../archive/README.md)
- 버전 비종속 후보: [`Backlog`](../plans/backlog/README.md)
