# 현재 상태

- 제품·공개 안정판: `0.15.0`, [정식 릴리스](https://github.com/gvm1229/aigent-hive/releases/tag/v0.15.0)·[npm](https://www.npmjs.com/package/aigent-hive/v/0.15.0)
- 단계 3: 19/19 완료. [계획](../plans/PLAN.md)·[정식 근거와 한계](../research/overhaul/OV150-013.md). develop에서 문서 종료, 단계 브랜치 원격·로컬 삭제 완료. 이번 요청 범위의 다음 단계 구현 미시작
- 공개 test.1: 소스 29875af0·후보 43305279711·복구 게시 43312182798·[세 OS 수용 43312886228](https://github.com/gvm1229/aigent-hive/actions/runs/43312886228) success. Windows x64·Linux musl x64·macOS arm64의 실제 공개 설치. [정확한 대상·검증값 결합·한계](../research/overhaul/OV150-011.md)
- 정식: [PR #94](https://github.com/gvm1229/aigent-hive/pull/94) 일반 병합·main 0d8338ab, main CI 43323303057·후보 43323365241·[복구 게시/구독자 전송 43329036663](https://github.com/gvm1229/aigent-hive/actions/runs/43329036663) success. 추가 develop→main 모델 검토 없음
- 제품 지문 sha256:4e410af13fd9b735d87d1b44fa3f3d07ee996f333e8e0f5e360b0ca7c51aacdc 불변. npm 6개 latest=0.15.0·test=0.15.0-test.1, 정식 GitHub 파일 29개·주석 태그 대상 main 확인. 첫 게시의 npm 전파 지연 종료 6 보존, 같은 후보로 복구
- 구독자 안내: main-pr 승인·원문/지문 검증·전송 성공. 수신자의 실제 열람 미검증
- PC: Codex·Windows 11 실제 0.14.2→0.15.0, `hive --version`의 AIgent Hive v0.15.0 (released 2026-11-12) 확인. core/harness/settings verified·감시 0.15.0 running/healthy·model_calls=0
- 감시 현재·직전 실행 파일 2개, 이전 0.14.1 사본 1개 자동 정리. prune 미리보기 후보 0·건너뜀 0·오류 없음, 수동 confirm 불필요
- 선택 질문 usage-enforcement·usage-warnings 2개 보류. 기존 보호 선택·사용자 내용·프로젝트 파일 보존, 답변 대행 없음
- [OV150-001](../research/overhaul/OV150-001.md) 완료: Windows 11·Claude Code 2.1.322 대화형 Stop→Codex→Claude 반환 실증. Codex 1회·재생 추가 호출 0, 재생 확인증 반환 공백은 OV160-002/003
- 제품 조율 비활성·활성화 경로 없음. Gemini CLI·다른 운영체제 실제 왕복·실제 호스트 취소/재점유 미검증. macOS x64·Linux arm64는 빌드·게시 확인, 공개 설치 수용 제외
- [OV150-014](../research/overhaul/OV150-014.md): 비배포 참조 처리기·제품 SummonEnvelope 단일 계약·가짜 Codex 적합성 시험 완료. 실제 모델 활성화·전역 호스트 구성 수정 없음
- [단계 검토](../research/overhaul/stage-3-review.md): Fable 5.2 COMMENT·경계 위반 0·높은 위험 0. 수정·P1 완료, P2 단계 6·P4 단계 4 배치. 로컬 Windows와 실제 CI의 정확한 검증 범위는 해당 근거
- [OV150-015](../research/overhaul/OV150-015.md) Rust 1.100 검사 호환·[016](../research/overhaul/OV150-016.md) Windows 버전 판별·[018](../research/overhaul/OV150-018.md) 공개 후 POSIX 원본 갱신·[019](../research/overhaul/OV150-019.md) macOS 정규 경로 기대값 완료
- [OV150-017](../research/overhaul/OV150-017.md) 원래 Windows 일회성 재개 Conflict 원인 미규명. 후속 Windows 수용 success와 구분, 원인 수정 주장 없음
- 산출물 점검 review 470·active 1 보존. 공유 빌드·실험실·다른 작업 원본 유지, 전체 캐시 정리 완료 주장 없음. 임시 배포 사본 4개·약 380MB는 자동 승인 검토의 재귀 삭제 거부로 보존
- 비추적 외부 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외

## 이전 결과

- 단계 1b 8/8·0.14.2 정식·PC 갱신: [OV142-008](../research/overhaul/OV142-008.md). OV141-011 후속 조건 해결, 원래 실패 이력 보존
- 단계 2 10/10 처리·develop 병합·분기 삭제: [종료와 한계](../research/overhaul/stage-2-closeout.md). 창 제어 폐기와 [공식 확장점 결정](../decisions/ADR-0026-official-cli-extension-points.md)
- 기존 Windows knowledge_lock·예약 작업 조회 시간초과·활성 state.lock 읽기 경합은 이전 후속 조사. 이 버전에서 원인 수정 주장 없음

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.15.0`
- 현재 등록 항목: 19/19 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->
