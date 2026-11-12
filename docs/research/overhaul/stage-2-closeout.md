# 단계 2 현재 결과와 재개 경계

- 기준일 2026-11-12, 실행 호스트 Codex·Windows 11, 제품 Hive 0.14.2
- 완료 5/10: [OVX-001](OVX-001.md) 30종의 general 기본값 100%, [OVX-002](OVX-002.md) 회수율 22/40·보류 3/10, [OVX-003](OVX-003.md) 관련 사실 3/3·범위 한정, [OVX-004](OVX-004.md) 파일 81개·Skill 28개·14,527단어·198,235바이트, [OVX-008](OVX-008.md) 세 제공자 약관 표
- 부분 확인: [OVX-005](OVX-005.md)의 Windows psmux 3.3.2·CLI 명령 경로·WSL 배포판. 실제 창 실험은 차단
- 차단: [OVX-006](OVX-006.md) 실제 알림 왕복, [OVX-007](OVX-007.md) 실제 재개·소모량·Gemini 별칭. 원인: 세 제공자 외부 자동 입력·화면 회수 허용 미확인, OVX-005 실제 수용 부재
- 환경 정정: WSL Ubuntu-24.04 로그인 셸에서 tmux와 Windows 연결 Codex·Gemini 경로 확인, Claude 경로 결과 없음. [재확인 한계](OVX-005.md) 참조. macOS 실행 환경 없음, 설치·로그인 대행 없음
- 남은 외부 근거 담당: 각 제공자의 공식 허용 문서 또는 서면 확인. 해소 뒤 OVX-008 갱신 → 준비된 환경에서 OVX-005 → OVX-006·007만 재개
- 사용자 결정: OVX-009, 02의 Runtime and orchestration·Skill boundary와 ADR-0019의 모델 프로세스 소유 경계 변경 여부. 오래된 계획의 78–84행 번호 대신 현재 섹션으로 식별
- 제안할 결정 범위: 제공자별 약관 허용 확인을 전제로 Hive의 공식 CLI 대화형 psmux·tmux 실행·입력·관측·종료·재개 허용 여부. 제공자 API 직접 호출·키·인증 파일 접근 금지 유지, 헤드리스는 보조 경로. Claude 헤드리스 출력은 현재 사용자 금지 유지
- OVX-009 ADR 초안·지침 변경·승인 대행 없음. OVX-010 develop 병합·단계 브랜치 삭제 없음

- 단계 검토: [Fable 5.2 검토 지적·권고와 보완](stage-2-review.md)

## 검증과 한계

- 항목별 실제 실행: `python scripts/test-lanes.py --lane documentation`, `python scripts/check-human-documentation-style.py --all`, `python scripts/check-plan-state.py`, `hive source-wiki index/lint`, `python scripts/branch-policy.py check test/overhaul-2-baseline`
- 문서 시험: 마지막 항목별 실행에서 163개 중 161개 통과·2개 제외, 종료 0. 두 제외는 현재 Windows의 심볼릭 링크 생성 권한 부재. 접합 경로 시험은 별도 실행, 모든 링크 형식·운영체제의 완전 수용 증명과 구분
- 초기 OVX-002 문서 탐색 실패 2회와 합성 입력의 문체 지적은 원래 실패 기록 보존 후 수정. 실제 측정 입력 10줄만 경로·줄·정확 지문 예외, 규칙 완화 없음
- 상태 검사: 기존 등록 제품 계획 8개 유효, 별도 scratch 등록으로 단계 2의 10개·의존성·완료 5개 근거 SHA-256 검사. source 단계 파일 이동·제품 버전 변경 없음
- 최종 범위: 문서·계획·Wiki·브랜치 검사, 제품 경로 차이 없음 확인. 실제 제공자 동작·Linux·macOS 제품 수용·전체 Rust 회귀·CI 전체 대기 제외
- 제품 검증 제외 이유: 제품 바이트 변경 없는 source 측정·문서 작업. provider 실험 제외 이유와 구분
- 사용자 비추적 AI 학습 검토 문서 두 개의 편집·커밋·삭제 없음. Claims·전역 호스트 설정·인증 파일 수정 없음
- 임시 입력·측정 스크립트는 `tests/work/overhaul-2-baseline/`와 `tests/work/pane-lab/`에 재현용 보존, Git 커밋 제외. 실제 Claims 검색 임시 결과는 관련 범주·지문 세 쌍만 보존
- 현재 프로젝트 범위의 안전한 기준선 사실만 Source Wiki에 기록. 사용자 Claims 쓰기 금지에 따라 실제 사용자 기억 저장 명령 미실행, 개인 선호·다른 프로젝트 본문 복사 없음
- 단계 브랜치 push·정확한 HEAD 일치 확인과 세션 기록 제거 뒤 보고. develop draft-merge·정식 출시·설치 없음
