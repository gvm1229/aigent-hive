# 실행 장소 규칙

2026-11-16 유지관리자 요청: 남은 재정비 항목을 클라우드 세션 가능 항목과 이 PC 전용 항목으로 구분. 분류 원안은 Fable 5.2 advisor 읽기 전용 제안.

## 장소

- `cloud`: 코드·문서·계획 작성, 가짜 실행기 시험, Rust·Python 적합성 시험, CI, PR, develop 병합
- `local`: 이 Windows PC 전용. 실제 로그인 호스트의 설치·갱신·플러그인·Skill 표시, 구독 모델 왕복, Codex 위임, 데스크톱 앱, 실제 `~/.hive`, npm·Discord 자격 증명
- `split`: cloud 부분 뒤 local 부분. local 부분은 괄호로 표기
- 워크플로 실행의 cloud 가능 여부: **가능**. 2026-11-16 단계 5 cloud 세션에서 GitHub MCP `actions_run_trigger`(`run_workflow`)로 `release.yml`을 `develop`에서 실행, 204 응답과 실행 [43816758281](https://github.com/gvm1229/aigent-hive/actions/runs/43816758281) 생성. 이 환경 정책상 `gh` CLI는 검색 전용이라 `gh workflow run` 자체는 미사용. 실패 재실행(`rerun_failed_jobs`)도 같은 연결로 성공
  - 따라서 공개 시험 후보·게시·세 OS 수용 워크플로는 cloud 항목. npm·GitHub Release 확인도 cloud. 이 PC 실제 호스트 확인·PC 설치만 local
- 표기: 항목의 `state` 줄 다음에 `  - 장소: cloud` / `local` / `split(local: <단계>)`. 표기 없는 항목은 local로 취급
- 가짜 실행기·CI 통과는 실제 호스트 근거가 아님. 0.15.2의 실제 호스트 결함 세 건은 이 PC에서만 발견([OV152-001](../../research/overhaul/OV152-001.md))
- cloud 세션에는 실행 가능한 `hive`가 없으므로 [07](../../../.agents/directives/07-installed-usage-guard.md)의 해당 경로에 따라 사전 확인 생략·보고

## 진행 기본값

- 2026-11-16 유지관리자: cloud 세션은 다음 정식 버전에 남은 cloud 작업을 local 작업만 남을 때까지 계속. 중간 보고 대기·확인 요청 없이 진행, local 단계에 닿으면 아래 넘기기로 종료. 목적은 cloud 세션 관리 없이 local 세션으로 바로 넘기기
- 2026-11-17 유지관리자: local 실행을 명시하지 않은 작업은 cloud 세션이 기본. 원문 "unless explicitly requested to run on the local machine, it is to be automatically assumed that I want things to run on the cloud session". 기다렸다가 local 세션에 넘기는 방식 유지
- 2026-11-17 유지관리자: 세션 장기화 금지. 단계 완료·버전 출시·넘기기 문서 작성 같은 이정표에서 세션 종료, 새 세션이 저장소의 넘기기 문서로 이어 받기. 원문 "If a milestone is reached, then a new session is to be spawned. To lessen the token usage due to context overload."

## 넘기기

- cloud 세션이 local 단계에 닿으면 그 단계 중지, 항목은 `- [ ]` 유지
- `state: awaiting-external-evidence; owner: local-pc; reason: 실제 호스트 확인 대기`로 변경, 기존 `depends` 유지. `reason` 값에 `; ` 금지
- `docs/research/overhaul/<ID>-local.md`에 커밋·실행할 명령·기대 결과·실패 시 볼 곳 기록, CURRENT의 다음 작업에 링크, push
- 이후 그 항목에 의존하지 않는 cloud 항목만 계속
- local 세션: `git pull --ff-only` → CURRENT의 대기 목록 → 기록된 명령 실행 → 근거를 `<ID>.md`에 기록 → 통과 시 `complete`, 실패 시 `agent-owned`로 되돌리고 실패 근거 첨부. 수정 자체는 cloud 가능

## 결정과 미결

- Gemini CLI는 이 계정에서 Google 지원 종료. Gemini 실제 수용 항목(OV160-003·004)은 미검증 유지. Antigravity를 작업자로 대체 확인하는 방식은 약관 위험으로 폐기([OV170-028](../../research/overhaul/OV170-028.md))
- 제안(미결정): 단계마다 공개 시험과 정식 사이에 local 실제 호스트 확인 항목 1개(0.15.2 OV152-007 형식). 실제 호스트 CLI 출력 기록을 가짜 실행기가 재생하는 고정물. 읽기 전용 `scripts/qualify-local-hosts.py`. 이전 정식→새 버전 갱신의 CI 시험
