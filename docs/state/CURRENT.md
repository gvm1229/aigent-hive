# 현재 상태

- 기준 branch: `develop@8151f51`, `origin/develop` 일치
- product version: `0.9.5`
- 다음 target: `0.10.0-test`
- 활성 계획: [`PLAN.md`](../plans/PLAN.md)
- 현재 milestone: Graphify 실패 뒤 `0.10.0` 대체 범위 결정

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

## `0.10.0` 완료 준비

- 문서 archive·버전 비종속 backlog·현재 정본 축소
- phase 중심 시험·fixture의 목적 중심 재편
- Graphify `0.9.47` 격리 조사·하드 게이트 실패·backlog 이전
- Codex·Claude·Antigravity 작업 자동 분담 가능성 조사
- Notion 실제 연결과 작업 자동 분담 활성화: backlog

## `0.10.0` 남은 범위

- Graphify 실패를 대체할 제품 범위와 수락 기준의 사용자 확인
- pre-`0.10.0` canonical 지식·프로젝트 보존 upgrade·rollback
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
- 제품 통합 중단, 후보 backlog 이전, `0.10.0` 대체 범위 결정 필요
- Codex·Claude·Antigravity 공식 하위 agent 기능 확인
- exact runtime attestation 부재와 설치 host 증거 결손으로 자동 분담 활성화 보류

## 현재 장애 요인

- Source Wiki index rebuild·lint 완료, error·warning `0건`
- Graphify macOS·Linux·Markdown 의미 추출·50,000 chunk 비용 미검증
- 현재 Windows에서 macOS·Linux 실제 설치 수용 실행 불가
- Claude 설치본 미인증·필수 lifecycle 수정 이전 version, Antigravity CLI 미설치

## 다음 작업

1. Graphify 실패 뒤 `0.10.0` 대체 범위 결정
2. 승인된 대체 범위의 번호 공개 시험판과 세 운영체제 수용

## 과거 기록

- `0.9.5` 마감 전 전체 상태: [`0.9.5-closeout.md`](../archive/state/0.9.5-closeout.md)
- 완료 계획: [`Archive`](../archive/README.md)
- 버전 비종속 후보: [`Backlog`](../plans/backlog/README.md)
