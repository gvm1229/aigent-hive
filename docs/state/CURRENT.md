# 현재 상태

- 기준 branch: `feature/0.10.0@bb6887c`, `origin/feature/0.10.0`과 push 전 local 차이
- product version: `0.9.5`
- 다음 target: `0.10.0-test`
- 활성 계획: [`PLAN.md`](../plans/PLAN.md)
- 현재 milestone: Host-neutral 연속 실행 closure·hook gate

## 최근 검증 근거

- npm `latest=0.9.5`, GitHub Release `v0.9.5`
- stable publication run `32115507331` 성공
- full-history checkout 보정 뒤 native runtime run `32118217691`의 다섯 target 성공
- Windows x64 공개 npm `0.9.5` binary SHA-256 `67dcdb4a83a2be1256c846c0a18b94166fb7ad11d7edcdd1ed9b1a750066237b`
- Windows 격리 Codex `0.148.0` user setup·install·validate와 stable update current 성공
- `origin/develop`과 `origin/main` product tree 차이 `0건`
- GitHub 열린 issue·PR `0건`
- 목적별 Python lane: documentation 43, security 103, contract 362, integration 82, release 55 통과
- Rust workspace: `394+3+1+104+34+61+54+113` unit·integration과 Wiki qualification 일반 시험 통과
- Windows 미지원 symbolic link·FIFO·POSIX·macOS 동작: 예상 건너뜀, 해당 플랫폼 수용 근거 아님

## `0.9.5` 마감

- `REL95-001–006` 완료
- macOS arm64 공개 안정판의 격리 direct install·user setup·validate·update check 성공
- Windows x64 공개 npm 안정판의 격리 설치·`hive --version`·user projection validate 성공
- `0.9.5` 출시 마감 완료
- `0.9.6` 미출시, `0.9.x` release line 종료

## `0.10.0` 완료 준비

- 문서 archive·버전 비종속 backlog·현재 정본 축소
- phase 중심 시험·fixture의 목적 중심 재편
- Graphify `0.9.47` 격리 조사·하드 게이트 실패·backlog 이전
- Codex·Claude·Antigravity 작업 자동 분담 가능성 조사
- host-owned 프로젝트 Skill 경로 세션 예약 계약: `0.10.0` 범위 편입
- registered nested-project knowledge scan 수정: `0.10.0` 범위 편입
- Hive-native Markdown 관계 graph·optional Graphify full-rebuild code-only adapter 범위 승인
- Backlog 6개·Archive 미완료 22개 검토 완료, 추가 자동 승격 `0건`
- 유지보수자 최종 선택: 추가 후보 없음, [`ADR-0020`](../decisions/ADR-0020-0.10.0-product-scope.md) 범위 확정
- Vector 검색: Qdrant Edge·SQLite engine hard gate와 통과 시 optional hybrid adapter 범위 추가
- Host-neutral 연속 실행: Goal·read-only closure gate·선택형 Stop hook의 조건부 조사 범위 추가
- `oh-my-codex@3ad79a8`·Codex `0.148.0`·Claude·Antigravity 공식 hook 조사 완료: Goal/task 실행 주체 + bounded Stop nudge 권고
- `ralph-loop` → `verified-workflow` rename·자연어 continuation 자동 routing 범위 승인
- 명시적 `adversarial-judge` Skill: 기존 package/quorum을 재사용하는 host-native clean-context Judge 단계로 범위 승인
- `package-review` → `judge-evidence`, `iterative-execution` → `verified-workflow` 병합과 모든 predecessor retired artifact cleanup 범위 승인
- `ralph-loop|iterative-execution` canonical source·catalog·schema·three-host projection을 `verified-workflow`로 병합 완료 (`cd3379a`)
- 자연어 verified workflow routing·사용자 override·inactive host fail-closed 구현 완료 (`c032030`)
- `hive run closure` read-only gate·pending criterion·closure digest·host-owned continuation envelope 구현 완료 (`97490e6`, `c37e8cb`)
- Stable Skill registry: `0.8.0`, `0.9.0–0.9.5` digest·side-effect·capability coverage 구현 완료 (`354ea0a`)
- `adversarial-judge` Skill·clean-context dispatch envelope·`judge-evidence` rename·Copier three-host parity 구현 완료 (`83e9722`, `b60f5e1`, `ac178d3`, `af51885`, `bb6887c`)
- 공개 stable 합집합 `0.8.0`, `0.9.0–0.9.5` 확인; historical built-in registry의 `0.9.1–0.9.5` 결손과 future stable publication append gate 범위 승인
- Stable tag Skill transition 비교: `0.8→0.9.0` rename 필수, `0.9.0–0.9.4` digest epoch 변화, `0.9.4→0.9.5` exact no-change epoch 공유 가능
- nested Git repository 아래 registered project scan 허용과 foreign sibling 격리 구현·회귀 검증 완료 (`7aab389`)
- host-owned Skill 세션 예약 구현·three-host 경로 회귀 검증 완료 (`96f2b06`)
- Notion 실제 연결과 작업 자동 분담 활성화: backlog

## `0.10.0` 남은 범위

- 상위 Git repository 안의 registered project root knowledge scan 복구와 sibling 격리 검증
- Hive-native Markdown 관계 graph·선택형 Graphify code-only adapter 구현·검증
- FTS·vector·graph 품질 기준선, local embedding boundary와 engine adopt|defer 판정
- pre-`0.10.0` canonical 지식·프로젝트 보존 upgrade·rollback
- Codex·Claude·Antigravity 연속 실행 capability와 bounded Stop hook 채택 판정
- `hive run closure` schema·CLI와 `continue-active-run` 선택형 capability 구현 여부 판정
- `verified-workflow` rename·routing·legacy alias와 `adversarial-judge` envelope·three-host launch 검증
- `0.7.0–0.9.5`·공개 test predecessor direct upgrade의 retired Skill·projection closure와 foreign-byte conflict 검증
- Stable release ledger·historical Skill registry·release surface inventory 집합 parity 검증
- 번호 공개 시험판의 세 운영체제 수용과 안정판 출시

## Graphify 경계

- Markdown: 유일한 지식 정본
- SQLite: 직접 사실·본문 검색
- Graphify: 관계·경로·영향 범위 탐색
- source·project·global 적용
- private·confidential collection별 격리 opt-in
- provider API·API key·query log·background watcher·Git hook·자동 MCP 등록 금지
- `0.9.47` 반복 전체 build·작은 query 성능 통과
- 증분·전체 graph 동등성 실패, upstream global visibility 격리 미지원
- 전면 지식 graph 통합 중단, full-scope 후보 backlog 유지
- 승인 범위: Hive-native Markdown 관계 graph와 optional full-rebuild code-only adapter
- Codex·Claude·Antigravity 공식 하위 agent 기능 확인
- exact runtime attestation 부재와 설치 host 증거 결손으로 자동 분담 활성화 보류

## 현재 장애 요인

- Source Wiki index rebuild·lint 완료, error·warning `0건`
- Graphify macOS·Linux·Markdown 의미 추출·50,000 chunk 비용 미검증
- 현재 Windows에서 macOS·Linux 실제 설치 수용 실행 불가
- Claude 설치본 미인증·필수 lifecycle 수정 이전 version
- Antigravity CLI `1.1.18`: `/hooks` JSON inspection 성공, hook `0건`; 실제 Stop continuation 시험 미실행
- `0.10.0` 안정판: 유지보수자의 명시적 승인 전 tag·publication 금지. 구현·로컬/CI 시험: 유지보수자 승인

## 다음 작업

1. Revision당 1회 nudge·진행 없는 3회 cap·cancel 우선 continuation envelope
2. Codex·Claude·Antigravity project-local Stop adapter fixture와 실제 host 검증
3. 모든 stable predecessor의 Skill lifecycle registry·rename cleanup·direct jump upgrade
4. 명시적 adversarial judge·`judge-evidence` migration·host-owned launch receipt
5. Native 관계 graph·vector gold corpus·FTS baseline

## 과거 기록

- `0.9.5` 마감 전 전체 상태: [`0.9.5-closeout.md`](../archive/state/0.9.5-closeout.md)
- 완료 계획: [`Archive`](../archive/README.md)
- 버전 비종속 후보: [`Backlog`](../plans/backlog/README.md)
