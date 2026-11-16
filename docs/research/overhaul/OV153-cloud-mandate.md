# 0.15.3 cloud 위임

- 기준일: 2026-11-17
- 유지관리자 원문: "Delegate what you can do the cloud session. Enforce it to work until every single work that's intended for cloud session work is complete for the 0.15.3 scope. So that the local session can work right after the cloud session is completed. I do not want to babysit the cloud session. I just want to wait, then come back and hand things off to the local session, that simple."
- cloud 세션은 중간 보고·확인 요청 없이 [실행 장소 규칙](../../plans/overhaul/venue.md)의 local 단계만 남을 때까지 진행. 정식 0.15.3 게시 금지

## cloud 순서

1. OV153-011 구현: `fix/ov153-011-agent-setup` 브랜치의 aef8ee61은 중단된 로컬 부분 구현, 빌드·시험 미확인. [요청 원문](OV153-011-request.md)대로 완성. `--capabilities` 없는 에이전트 적용, 설정 요약의 에이전트 경로, project-setup·user-setup Skill과 모든 복사본, 시험, 근거 `OV153-011.md`
2. 독립 검토(Fable 5.2, 부재 시 code-reviewer) 뒤 수정·재확인, develop 병합·브랜치 삭제
3. 0.15.3-test.2: 시험 의도·CI·후보·게시·세 OS 수용
4. 출시 노트·구독자 안내 갱신, 정식 공개 문서 준비(출시일은 출시 때 기입). main PR·정식 워크플로 실행 제외
5. `OV153-local-handoff.md`에 local 명령·기대 결과 기록, CURRENT에 "local 세션 시작 위치" 표시, push

## local 순서

- test.2 PC 설치·갱신, 실제 Claude·Codex 세션에서 Skill로 빈 폴더(한 번에 하나씩 질문)와 기존 프로젝트(탐색→제안→적용) 설정, 다섯 앱 검증
- 통과 뒤 main PR·정식 후보·게시·확인·OV153-010 종료·PC 정식 설치
