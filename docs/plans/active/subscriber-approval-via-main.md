# 구독자 안내 승인 방식 변경

> Plan version: `0.14.1`
> Scope: `source`

- 요청·결정일: 2026-11-12
- 범위: 현재 제품 0.14.1과 별개인 저장소 출시 자동화. 제품 파일 변경·새 공개 시험 없음
- 권한: `chore/subscriber-approval-via-main` 구현·검증·커밋·push. 다른 작업 트리·보호 브랜치 통합·PR 생성·GitHub 설정·출시 실행 제외
- 근거: 지침 02의 배포 경계, 03의 CI 규칙, `scripts/check-test-release-gate.py`의 제품 경로 목록. 이번 알림 스크립트와 `.github/`는 비제품 파일

## 실행 항목

- [x] [SMAIN-001] 알림 구현·검사: `scripts/publish-stable-discord-update.py`의 외부 지문 대신 `--approval-env`로 지정한 환경 변수의 정확한 `main-pr` 값 요구. 원문·한 줄 SHA-256 파일·파일명 대조 유지. 등록 스크립트와 해당 검사 삭제. 두 출시 workflow에 환경 변수 연결, 기존 후보 SHA·main·공개 안정판 확인 유지. 복구의 다른 커밋 문구 재전송 제거, 출시 소스 커밋과 버전별 중복 차단 유지. 관련 Python 모듈·release 검사 묶음·설치된 actionlint 실행, 종료값 0 확인
  - state: complete; evidence: repo:tests/results/subscriber-approval-via-main-2026-11-12.md#sha256:17b1f1ee26169202efd61fcb232bebfc43120c1bc61cb767aa56bc9875487c25
- [x] [SMAIN-002] 승인 문서·지식: 결정 기록·출시 안내·소스 전용 요약 스킬·기존 참조 갱신. 승인 원문과 지문 파일의 develop→main 검토 PR, `release-publication` 환경 변수 `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr`의 최초 1회 설정, 기존 비밀값 삭제 가능 명시. documentation 검사 묶음·한국어 문체·Source Wiki 검사, 종료값 0 확인
  - state: complete; evidence: repo:tests/results/subscriber-approval-via-main-2026-11-12.md#sha256:17b1f1ee26169202efd61fcb232bebfc43120c1bc61cb767aa56bc9875487c25
- [x] [SMAIN-003] 전달: 구현과 문서 관심사별 한국어 커밋, 정확한 원격 브랜치만 push, 세션 기록 제거. 실제 GitHub 설정·Discord 전송 미실행 한계 기록
  - state: complete; evidence: repo:tests/results/subscriber-approval-via-main-2026-11-12.md#sha256:17b1f1ee26169202efd61fcb232bebfc43120c1bc61cb767aa56bc9875487c25

## 순서·완료 기준

`SMAIN-001` → `SMAIN-002` → `SMAIN-003`. 환경 변수 누락·다른 값·지문 불일치 시 전송 전 거부, 정확한 값과 파일에서 기존 배너→안내 순서 유지. 로컬 검사는 Windows의 Codex 실행 범위이며 GitHub 환경 변수 설정·실제 Discord 수신 증명 제외. 실패 시 해당 검사 원인 수정 후 재검사, 새 권한 필요 단계만 중단. 기존 제품 출시 항목의 완료 여부 변경 제외.

## 후속 정리 계획

- 2026-11-12 후속 요청: 기존 미커밋 작업은 `2437cd55`에서 완료된 상태 확인
- 순서: 최신 `origin/develop` 병합 → 과거 기록 바이트 복원 → 현재 사실 지문 갱신 → 알림 단위 검사·documentation·release·전체 문체·설치된 actionlint 검사 → 한국어 커밋·지정 브랜치 push
- 변경 경계: `docs/archive/**`, ADR-0020, 0.13.2·0.14.0 출시 계획은 `origin/develop` 원본. 재정비 현재 위치·OV141-011도 원본 보존. 현재 사실·안내는 유지
- 검사 실패 복구: 과거 기록 수정 대신 필요한 검사 예외·대상 범위 조정, 변경된 검사 동작의 반례 검증
- 완료 기준: 요청 검사 종료값 0, 과거 기록·출시 소유 항목 원본 일치, 지정 원격 브랜치 push, 세션 기록 제거. 다른 작업 트리·main·설정·workflow·PR 접근 제외
