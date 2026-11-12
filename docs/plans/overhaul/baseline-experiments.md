# 단계 2 — 기준선과 창 실험

> Plan version: 0.14.2
> Scope: source

브랜치: `test/overhaul-2-baseline`([브랜치 규칙](README.md)). 진입: 0.14.2 정식·정리 확인. 제품 바이트 변경 없음. 실험 코드는 `tests/work/`의 일회성 스크립트, 결과는 `docs/research/overhaul/<ID>.md`

## 항목

- [x] [OVX-001] 프로젝트 판별 코퍼스와 현재 결과
  - state: complete; evidence: repo:docs/research/overhaul/OVX-001.md#sha256:8a61efd983735d9a8efe85daf7e0fbe48ac18ebe5fe4ce96df3f19306d0c9a51
  - 근거: [측정 결과](../../research/overhaul/OVX-001.md)
  - 변경: `tests/fixtures/project-detection/`에 표시 파일만 둔 최소 프로젝트 30종(TypeScript 웹 앱, Next.js, Vite, Python, Rust, Go, Java, .NET, 모노레포, 문서 전용, 빈 폴더 등)과 기대 결과표. 현재 `project_kind` 결과 기록
  - 확인: 30종 전부 기록, `general` 판정 비율 수치
- [x] [OVX-002] 지식 회수 정답 질의집과 현재 회수율
  - state: complete; evidence: repo:docs/research/overhaul/OVX-002.md#sha256:9581105e5f8a69db4b40591ca3c281559fde48d30e8c262d991f077f9290233d
  - 근거: [측정 결과](../../research/overhaul/OVX-002.md)
  - 변경: 한·영 질의 40개 이상(HTML 디자인 선호 사례, "색"·"글꼴" 같은 짧은 한국어 질의 포함)과 합성 지식 저장소, 과적합 방지용 보류 묶음 분리
  - 확인: 질의별 기대 사실과 현재 적중 표, 전체 회수율 수치
- [x] [OVX-003] 실제 기억 저장소의 세 사실 존재 확인
  - state: complete; evidence: repo:docs/research/overhaul/OVX-003.md#sha256:49cf248d7549f650d98d5acaadf296fcf61da7f1164e9c5757547d2029c18149
  - 근거: [세 사실의 존재·범위](../../research/overhaul/OVX-003.md)
  - 변경: `~/.hive/knowledge/Claims`에서 미니멀 스타일·색·피할 디자인 사실 검색, 읽기 전용
  - 확인: 3개 각각 존재/부재 기록. 부재면 원인은 기록 규칙, 0.15.2의 OV152-002 우선
- [x] [OVX-004] 크기 기준선
  - state: complete; evidence: repo:docs/research/overhaul/OVX-004.md#sha256:d919e893c1e21d2851dee03af64731a6b4263965a2edf945294f23fb9222849d
  - 근거: [크기 측정](../../research/overhaul/OVX-004.md)
  - 변경: 새 프로젝트당 Hive 파일 수, 내장 Skill 수·단어 수, 임시 사용자 기준 `hive storage` 분류별 용량
  - 확인: 세 수치와 측정 명령 기록
- [ ] [OVX-005] 창 실험 1 — 다중화 도구와 대화형 실행
  - state: blocked; depends: OVX-008; owner: provider; reason: OVX-008의 세 제공자 자동 입력 허용 미확인
  - 근거: [환경·차단·해결 경로](../../research/overhaul/OVX-005.md)
  - 실행 조건: OVX-008에서 제공자별 현재 약관의 대화형 자동 입력 허용 확인 뒤에만 실행. 불허·허용 미확인 제공자 부분은 `blocked`와 이유·해결 경로 기록
  - 변경: `tests/work/pane-lab/` 스크립트. Windows psmux와 WSL·macOS tmux에서 3창 열기, `claude --model --effort`·`codex -m -c model_reasoning_effort=`·`gemini -m` 대화형 실행, `send-keys -l`로 한글·긴 문장 입력, `capture-pane`으로 상태 읽기
  - 확인: 각 창의 지정 모델·수준 표시, 입력 손실 0건, 자격 증명 파일 접근 0건
- [ ] [OVX-006] 창 실험 2 — 입력 대기 감지와 알림 왕복
  - state: blocked; depends: OVX-005; owner: provider; reason: OVX-005 실제 창 미수용과 OVX-008 자동 입력 허용 미확인
  - 근거: [훅·왕복 미실행과 해결 경로](../../research/overhaul/OVX-006.md)
  - 변경: Claude Code `Stop`·`Notification` 훅, Codex `notify`, Gemini CLI 훅으로 대기 신호 파일 기록. 대기 창에만 알림 한 줄 입력
  - 확인: 작업 중 창 입력 0건, 대기 → 알림 → 메시지 상자 읽기 왕복 3회 성공
- [ ] [OVX-007] 창 실험 3 — 종료·재개·한도·추론 수준
  - state: blocked; depends: OVX-005; owner: provider; reason: OVX-005 실제 창 미수용과 OVX-008 자동 입력 허용 미확인
  - 근거: [재개·사용량·수준 미실행과 해결 경로](../../research/overhaul/OVX-007.md)
  - 변경: 창 강제 종료 뒤 `claude -r`·`codex resume`·`gemini -r` 재개, 인증 만료·한도 도달 화면 문구 수집, 같은 계정 동시 창 3개의 사용량 소모 관찰, Gemini `modelConfigs.customAliases`의 `thinkingLevel` 효과
  - 확인: 재개 결과·감지 문구·소모량 표, Gemini 별칭 형식 확정
- [x] [OVX-008] 제공자 약관 원문 재확인
  - state: complete; evidence: repo:docs/research/overhaul/OVX-008.md#sha256:be476982054148e3252fc12da824f35cea1bb27a8190bae689fe3494bdc64e10
  - 근거: [공식 약관과 판정](../../research/overhaul/OVX-008.md)
  - 변경: Anthropic·OpenAI·Google의 현재 약관·정책에서 대화형 CLI의 자동 입력·병렬 사용 조건 확인. 조항 요약·출처·마지막 확인일 기록
  - 확인: 세 제공자 표 완성, 확인일 90일 이내
- [ ] [OVX-009] 창 실행 허용 ADR 초안과 승인
  - state: awaiting-user-authority; depends: OVX-005,OVX-006,OVX-007,OVX-008; owner: user; reason: 02 지침의 프로세스 실행 금지 변경 승인
  - 변경: `docs/decisions/`에 ADR 초안(대화형 창 실행 허용, 토큰·API 키 금지 유지, 헤드리스는 보조), `.agents/directives/02-architecture.md:78-84` 변경안, `ADR-0019`와의 관계
  - 확인: 사용자 승인 기록, `python scripts/check-agent-directives.py`
- [ ] [OVX-010] develop 병합과 단계 브랜치 종료
  - state: agent-owned; depends: OVX-001,OVX-002,OVX-004,OVX-008,OVX-009
  - 변경: 단계 브랜치 `test/overhaul-2-baseline`를 최신 `develop`에 병합(fast-forward 우선, 아니면 merge 커밋), `develop` push. 병합 뒤 단계 브랜치 삭제
  - 확인: `git branch --merged develop`에 단계 브랜치 포함, `develop`과 `origin/develop` 일치
