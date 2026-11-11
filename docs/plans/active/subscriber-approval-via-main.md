# 구독자 안내 승인 방식 변경

> Plan version: `0.14.1`
> Scope: `source`

- 요청·결정일: 2026-11-12
- 범위: 현재 제품 0.14.1과 별개인 저장소 출시 자동화. 제품 파일 변경·새 공개 시험 없음
- 권한: `chore/subscriber-approval-via-main` 구현·검증·커밋·push. 다른 작업 트리·보호 브랜치 통합·PR 생성·GitHub 설정·출시 실행 제외
- 근거: 지침 02의 배포 경계, 03의 CI 규칙, `scripts/check-test-release-gate.py`의 제품 경로 목록. 이번 알림 스크립트와 `.github/`는 비제품 파일

## 실행 항목

- [x] [SMAIN-001] 알림 구현·검사: `scripts/publish-stable-discord-update.py`의 외부 지문 대신 `--approval-env`로 지정한 환경 변수의 정확한 `main-pr` 값 요구. 원문·한 줄 SHA-256 파일·파일명 대조 유지. 등록 스크립트와 해당 검사 삭제. 두 출시 workflow에 환경 변수 연결, 기존 후보 SHA·main·공개 안정판 확인 유지. 복구의 다른 커밋 문구 재전송 제거, 출시 소스 커밋과 버전별 중복 차단 유지. 관련 Python 모듈·release 검사 묶음·설치된 actionlint 실행, 종료값 0 확인
  - state: complete; evidence: repo:tests/results/subscriber-approval-via-main-2026-11-12.md#sha256:0c0fed9f92bfdede64e97b627b88f260025a329af789fc02832a9f9142c358dd
- [x] [SMAIN-002] 승인 문서·지식: 결정 기록·출시 안내·소스 전용 요약 스킬·기존 참조 갱신. 승인 원문과 지문 파일의 develop→main 검토 PR, `release-publication` 환경 변수 `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr`의 최초 1회 설정, 기존 비밀값 삭제 가능 명시. documentation 검사 묶음·한국어 문체·Source Wiki 검사, 종료값 0 확인
  - state: complete; evidence: repo:tests/results/subscriber-approval-via-main-2026-11-12.md#sha256:0c0fed9f92bfdede64e97b627b88f260025a329af789fc02832a9f9142c358dd
- [ ] [SMAIN-003] 전달: 구현과 문서 관심사별 한국어 커밋, 정확한 원격 브랜치만 push, 세션 기록 제거. 실제 GitHub 설정·Discord 전송 미실행 한계 기록
  - state: agent-owned

## 순서·완료 기준

`SMAIN-001` → `SMAIN-002` → `SMAIN-003`. 환경 변수 누락·다른 값·지문 불일치 시 전송 전 거부, 정확한 값과 파일에서 기존 배너→안내 순서 유지. 로컬 검사는 Windows의 Codex 실행 범위이며 GitHub 환경 변수 설정·실제 Discord 수신 증명 제외. 실패 시 해당 검사 원인 수정 후 재검사, 새 권한 필요 단계만 중단. 기존 제품 출시 항목의 완료 여부 변경 제외.
