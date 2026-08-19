# 현재 상태

- 기준 branch: `develop@8151f51`, `origin/develop` 일치
- product version: `0.9.5`
- 다음 target: `0.10.0-test`
- 활성 계획: [`PLAN.md`](../plans/PLAN.md)
- 현재 milestone: `GPH10-001–006` Graphify 격리 조사와 채택 판정

## 최근 검증 근거

- npm `latest=0.9.5`, GitHub Release `v0.9.5`
- stable publication run `32115507331` 성공
- full-history checkout 보정 뒤 native runtime run `32118217691`의 다섯 target 성공
- `origin/develop`과 `origin/main` product tree 차이 `0건`
- GitHub 열린 issue·PR `0건`
- 목적별 Python lane: documentation 43, security 103, contract 362, integration 82, release 55 통과
- Rust workspace: `394+3+1+104+34+61+54+113` unit·integration과 Wiki qualification 일반 시험 통과
- Windows 미지원 symbolic link·FIFO·POSIX·macOS 동작: 예상 건너뜀, 해당 플랫폼 수용 근거 아님

## `0.9.5` 마감

- `REL95-001–005` 완료
- macOS arm64 공개 안정판의 격리 direct install·user setup·validate·update check 성공
- 남은 `REL95-006`: 현재 Windows x64의 공개 안정판 실제 설치·`hive --version`·user projection validate
- Windows CI native runtime 성공은 실제 사용자 설치 수용의 대체 근거 아님

## `0.10.0` 범위

- 문서 archive·버전 비종속 backlog·현재 정본 축소
- phase 중심 시험·fixture의 목적 중심 재편
- Graphify `0.9.47` 격리 조사와 하드 게이트 통과 뒤 선택형 파생 graph 도입
- Codex·Claude 작업 자동 분담의 공식·실제 가능성 조사만 수행
- Notion 실제 연결과 작업 자동 분담 활성화: backlog

## Graphify 경계

- Markdown: 유일한 지식 정본
- SQLite: 직접 사실·본문 검색
- Graphify: 관계·경로·영향 범위 탐색
- source·project·global 적용
- private·confidential collection별 격리 opt-in
- provider API·API key·query log·background watcher·Git hook·자동 MCP 등록 금지
- 조사 하드 게이트 실패 시 제품 통합 중단과 `0.10.0` 범위 재검토

## 현재 장애 요인

- Source Wiki index rebuild·lint 완료, error·warning `0건`
- Graphify 출력 schema·dependency lock·세 운영체제 설치·50,000 chunk 비용 미검증
- 현재 Windows에서 macOS·Linux 실제 설치 수용 실행 불가

## 다음 작업

1. Graphify 격리 조사와 채택 판정
2. 통과 시 조건부 제품 구현·upgrade 수용
3. 작업 자동 분담 가능성 조사
4. 번호 공개 시험판과 세 운영체제 수용

## 과거 기록

- `0.9.5` 마감 전 전체 상태: [`0.9.5-closeout.md`](../archive/state/0.9.5-closeout.md)
- 완료 계획: [`Archive`](../archive/README.md)
- 버전 비종속 후보: [`Backlog`](../plans/backlog/README.md)
