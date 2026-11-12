# 단계 2 — 기준선과 공식 확장점 결정

> Plan version: 0.14.2
> Scope: source

브랜치: `test/overhaul-2-baseline`([브랜치 규칙](README.md)). 진입: 0.14.2 정식·정리 확인. 제품 바이트 변경 없음. 실험 코드는 `tests/work/`의 일회성 스크립트, 결과는 `docs/research/overhaul/<ID>.md`

## 집계 기준

- 9/10 처리 완료: 기준선·조사·결정 확인 6개, 창 실험 대체 종료 3개. 실험 성공 9개라는 뜻과 구분
- OVX-005–007 기존 근거 파일 변경 없음. 실제 대체 왕복은 단계 3 첫 항목으로 이동, 이번 요청에서 미실행
- OVX-010은 현재 사용자 지시로 실행 제외. 제공자 창 제어 서면 확인은 비차단 별도 조사, 단계 3 진입의 조건 아님

## 항목

- [x] [OVX-001] 프로젝트 판별 코퍼스와 현재 결과
  - state: complete; evidence: repo:docs/research/overhaul/OVX-001.md#sha256:a915958a3ba29cede7f163a2bf3b7ddb1bb4a3225e856427d67c8fe89943e55b
  - 근거: [측정 결과](../../research/overhaul/OVX-001.md)
  - 변경: `tests/fixtures/project-detection/`에 표시 파일만 둔 최소 프로젝트 30종(TypeScript 웹 앱, Next.js, Vite, Python, Rust, Go, Java, .NET, 모노레포, 문서 전용, 빈 폴더 등)과 기대 결과표. 현재 `project_kind` 결과 기록
  - 확인: 30종 전부 기록, `general` 판정 비율 수치
- [x] [OVX-002] 지식 회수 정답 질의집과 현재 회수율
  - state: complete; evidence: repo:docs/research/overhaul/OVX-002.md#sha256:9d142f8151f828020262bfad89a34f41e989435ec7c3e60ef26e16be1c4681a7
  - 근거: [측정 결과](../../research/overhaul/OVX-002.md)
  - 변경: 한·영 질의 40개 이상(HTML 디자인 선호 사례, "색"·"글꼴" 같은 짧은 한국어 질의 포함)과 합성 지식 저장소, 과적합 방지용 보류 묶음 분리
  - 확인: 질의별 기대 사실과 현재 적중 표, 전체 회수율 수치
- [x] [OVX-003] 실제 기억 저장소의 세 사실 존재 확인
  - state: complete; evidence: repo:docs/research/overhaul/OVX-003.md#sha256:49cf248d7549f650d98d5acaadf296fcf61da7f1164e9c5757547d2029c18149
  - 근거: [세 사실의 존재·범위](../../research/overhaul/OVX-003.md)
  - 변경: `~/.hive/knowledge/Claims`에서 미니멀 스타일·색·피할 디자인 사실 검색, 읽기 전용
  - 확인: 3개 각각 존재/부재 기록. 부재면 원인은 기록 규칙, 0.15.2의 OV152-002 우선
- [x] [OVX-004] 크기 기준선
  - state: complete; evidence: repo:docs/research/overhaul/OVX-004.md#sha256:32b2ddcd2538fb0beae217e5ff27d94fe450abf26f27153d94447b796a19b821
  - 근거: [크기 측정](../../research/overhaul/OVX-004.md)
  - 변경: 새 프로젝트당 Hive 파일 수, 내장 Skill 수·단어 수, 임시 사용자 기준 `hive storage` 분류별 용량
  - 확인: 세 수치와 측정 명령 기록
- [x] [OVX-005] 창 실험 1 — 새 설계로 대체 종료
  - state: complete; evidence: repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:de58ee344cdb4f4b933a1ba490da5eea13bc076fbdb38c532d1f66bae16e7c66
  - 근거 보존: [환경·차단 기록](../../research/overhaul/OVX-005.md)
  - 종료 이유: ADR-0026의 공식 확장점 한정으로 창 열기·자동 입력·화면 회수 실험 폐기. 실제 제공자 실행 성공과 구분
  - 대체: 단계 3 [OV150-001](0.15.0-orchestrator.md)의 프로젝트 한정 훅·파일 봉투·Codex 왕복, 후속 작업자 프로필·복구 검증
- [x] [OVX-006] 창 실험 2 — 새 설계로 대체 종료
  - state: complete; evidence: repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:de58ee344cdb4f4b933a1ba490da5eea13bc076fbdb38c532d1f66bae16e7c66
  - 근거 보존: [알림 왕복 미실행](../../research/overhaul/OVX-006.md)
  - 종료 이유: 대기 창 키 입력 경로 제거. 실제 왕복 성공 없음. 대체는 단계 3 OV150-001, 정식 중재는 단계 6 OV160-002·003
- [x] [OVX-007] 창 실험 3 — 새 설계로 대체 종료
  - state: complete; evidence: repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:de58ee344cdb4f4b933a1ba490da5eea13bc076fbdb38c532d1f66bae16e7c66
  - 근거 보존: [재개·사용량·수준 미실행](../../research/overhaul/OVX-007.md)
  - 종료 이유: 화면 문구 수집·외부 창 재개 경로 제거. 대체는 단계 3 OV150-001·005·006의 공식 프로필·확인증·중복 방지, 실제 재개·취소는 단계 6·7 수용. 기존 미실행을 성공으로 변경하지 않는 종료
- [x] [OVX-008] 제공자 약관 원문 재확인
  - state: complete; evidence: repo:docs/research/overhaul/OVX-008.md#sha256:67722d841b3fa50ad242ea8c715ff98893cf200a2994ca6a50d8acf132ce5bb2
  - 근거: [공식 약관과 판정](../../research/overhaul/OVX-008.md)
  - 변경: Anthropic·OpenAI·Google의 현재 약관·정책에서 대화형 CLI의 자동 입력·병렬 사용 조건 확인. 조항 요약·출처·마지막 확인일 기록
  - 확인: 세 제공자 표 완성, 확인일 90일 이내
- [x] [OVX-009] 공식 확장점 결정과 승인 반영
  - state: complete; depends: OVX-008; evidence: repo:docs/decisions/ADR-0026-official-cli-extension-points.md#sha256:de58ee344cdb4f4b933a1ba490da5eea13bc076fbdb38c532d1f66bae16e7c66
  - 근거: [ADR-0026](../../decisions/ADR-0026-official-cli-extension-points.md), [반영 계획·검증](../../research/overhaul/OVX-009.md)
  - 결정: 2026-11-12 사용자 승인으로 Fable 권고 (b) 채택. 공식 확장점만 사용, 외부 키 주입·화면 캡처·Claude 비대화형 금지. Codex·Gemini 구독 비대화형 작업자, 호스트 실행 소유권 유지
  - 반영: 02의 Runtime and orchestration·Skill boundary, 단계 3·6·7과 README. ADR-0019 수정·대체 없음
  - 확인: 승인 원문·날짜 보존, 지침 읽기 예산·문서·계획 검사. 실제 제공자 CLI·모델 실행 없음
- [x] [OVX-010] develop 병합과 단계 브랜치 종료
  - state: complete; depends: OVX-001,OVX-002,OVX-004,OVX-008,OVX-009; evidence: repo:docs/research/overhaul/stage-2-review.md#sha256:b2c0e536cad2a7ef61114d1524f96594aac1c24acc4deab4f3a5957a3f08ef02
  - 결과: Fable 단계 검토(COMMENT) 지적 반영 뒤 `develop`에 fast-forward 병합, 단계 브랜치 로컬·원격 삭제
  - 변경: 단계 브랜치 `test/overhaul-2-baseline`를 최신 `develop`에 병합(fast-forward 우선, 아니면 merge 커밋), `develop` push. 병합 뒤 단계 브랜치 삭제
  - 확인: `git branch --merged develop`에 단계 브랜치 포함, `develop`과 `origin/develop` 일치
