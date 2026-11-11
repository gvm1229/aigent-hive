# 구독자 안내의 main PR 승인

- 결정일: 2026-11-12
- 근거: 사용자 요청의 출시마다 비밀값 변경 제거, 검토한 main PR을 구독자 안내 승인으로 사용
- 범위: 저장소 출시 자동화·소스 전용 요약 스킬. 설치 제품 파일·버전 변경 없음

## 승인과 전송

1. `docs/releases/<X.Y.Z>.subscriber.ko.md` 원문과 `.sha256` 지문 파일의 일반 develop→보호된 main 검토 PR 반영
2. `release-publication` 환경의 일반 변수 `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr` 최초 1회 설정. 비밀값 제외, 이후 출시별 설정 변경 불필요
3. 원문과 지문을 출시 소스 커밋에서 읽고 검증. 한 줄·정확한 파일명·원문 바이트 일치 확인 유지
4. GitHub Release 성공 뒤 기존 비밀값 `AIGENT_HIVE_RELEASE_DISCORD_WEBHOOK_URL`로 배너→한국어 안내 순서 전송

변수 누락·다른 값은 전송 전 거부. 안정판 게시 전 검사에서도 같은 거부 유지. 게시의 main 후보 SHA 결합과 복구의 공개 안정판·성공 게시 실행 결합 유지.

## 교체·제외

- 출시별 외부 승인 지문과 등록 스크립트 폐기. 과거 출시별 지문 비밀값 삭제 가능, 이번 작업의 GitHub 설정 변경 없음
- 다른 main/develop 커밋 문구의 재전송 폐기. 누락 복구는 출시 당시 문구만 허용, 버전별 중복 차단 유지
- 변경 문구는 새 검토 PR과 새 출시 소스 필요. PR 승인만으로 게시·Discord 발송 실행 권한 발생 없음
- 과거 결정의 외부 지문 등록 절차를 이 결정으로 대체. Git 이력에서 과거 방식 복구 가능

## 검증 범위

Windows의 Codex에서 알림 동작 25개·출시 151개·문서 163개·보안 103개 검사 및 문체 검사 종료값 0 확인. actionlint는 미설치로 미실행, 두 workflow의 셸 실행 부분 21개는 Bash 구문 검사 통과. 실제 GitHub 변수 설정·workflow 실행·Discord 발송 제외. 로컬 HTTP 수신 검사로 전송 순서와 거부 조건 확인, 실제 Discord 수신자 열람 증명 제외.

상세 근거·건너뛴 범위: [검증 기록](../../tests/results/subscriber-approval-via-main-2026-11-12.md). 기존 0.14.1 안내문 바이트는 보존, 문체 검사 예외는 해당 초안 17행의 정확한 지문 한 건만 허용.
