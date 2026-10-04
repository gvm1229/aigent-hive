# 현재 상태

- 제품 버전: `0.11.1`, 공개 안정판: `0.11.0`, 작업 브랜치: `develop`
- 공개 시험 수용·현재 사용자 설치: `0.11.1-test.8`
- 정본: [활성 계획](../plans/PLAN.md), [정식 출시 실행](../plans/0.11.1-stable-release.md)
- 이전 수치·승인·미게시 후보·로컬 검사 이력: [반영 전 상태](../archive/state/0.11.1-before-test8-application.md)

## 생성된 현재 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.11.1`
- 현재 등록 항목: 26/26 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

## 현재 근거

- [추가 정리](../../tests/results/further-cleanup-20261005.md): 최초 112.217GiB → 첫 정리 18.060GiB → 최종 정리 4.752GiB. 현재 설치·연구 코드·작은 결과 보존, 생성한 새 검증 산출물도 종료 후 정리
- [매일 정리](../guides/test-cleanup.md): 한국 시간 09시 자동화 활성, 검토·결과 커밋을 마친 정확 경로만 삭제. 관리 영역 20GiB 초과·미검토·만료·오류 보고, 미래 예약 실행의 성공 주장 없음
- [test.8 공개 수용·실제 반영](../research/skill-merge-public-test8-0.11.1.md): 소스 `06d56360`, 필수 CI `37217641485`, 후보 `37218215348`, 복구 게시 `37219894233`, 공개 수용 `37220067876`
- Windows·Linux 최초 성공, macOS 새 환경 재검증 성공. 최초 macOS 검색 도우미 실패·원인 미확정 한계 보존, 새 성공을 원인 수정 근거로 해석하는 주장 제외
- 공개 Windows 바이너리의 실제 Claude 2.1.163 세 경로 설치·갱신·보존 통과, 모델 호출 0건
- [부속 폴더 오류 수정](../research/skill-resource-cleanup-0.11.1.md): test.7 실제 적용은 자동 원복, 실제 삭제 파일로 빈 상위 폴더를 검증하도록 수정. Windows 전체 Rust 976개 통과·4개 수동 조건 제외, Python 925개 통과·44개 운영체제/권한 조건 제외, 제외 항목의 실행 성공 주장 없음
- 현재 사용자 test.8 인증 갱신·설치 검증 완료, 지식 502개·설정 2개·Hive 표시 밖 지침 보존
- WProject: 기본 스킬 제외·사용자 파일 이동 없이 새 Hive 개선과 사용자 검사 결합. 승인된 두 파일 내용 그대로 반영, 총 34개 경로·원본과 같은 추가 중복 14개 정리, 사용자 AGENTS·기존 별도 문서 3개 보존
- `hive.project-upgrade-current`, 두 번째 미리보기 변경 0건. 새 실제 Codex 관리 서버의 프로젝트 ship 한 개·활성 상태, 모델 호출 0건. Unity 실행·소비자 Git 커밋 없음
- `SDP-005` 전역 실제 호출은 사용자 새 대화에서 확인 완료, 프로젝트 호출 `DPS-003`과 구분

- 관련 구현 근거: [지침 이식·프로젝트 스킬](../research/directive-localization-and-project-skills-0.11.1.md), 승인된 [구독자 공지](../releases/0.11.1.subscriber.ko.md)

## 승인과 다음 행동

- 2026-10-05 현재 사용자 설치·검토된 WProject 결합·0.11.1 진행 승인에 따른 구현·공개 시험·반영 완료
- 0.11.1 안정판·보호 main·태그·npm·GitHub 게시·881자 Discord 문구와 전송의 기존 승인 유지, 추가 승인 질문 불필요
- [사용자 제공 두 새 대화 수용](../research/project-skill-user-acceptance-0.11.1.md): 프로젝트 ship의 명시·자연어 선택과 Unity 검사·변경 분리 규칙 설명 확인. 실제 검사·Unity 실행·커밋의 통과 증명 제외
- 두 실제 대화 근거와 파일·관리 서버 검사 구분, 호스트 강제 종료·모델 API 우회 없음
- DPS-003 완료, 기능 수용 26/26. 2026-10-05 대상 0.11.1 안정판까지 진행의 사용자 재확인. 승격 검사 보완 → 보호 main 통합·안정판 후보·게시 → 승인된 Discord 전송; 아직 게시 전
