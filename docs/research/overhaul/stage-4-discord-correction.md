# 0.15.0 Discord 안내 정정

## 확인과 선택

- 요청·범위: [유지관리자 원문](stage-4-human-output-request.md), OV151-016. 현재 구현·모의 검사·브랜치 push만 진행, 외부 전송·main 병합 제외
- 실행 환경: Codex·Windows 11 10.0.26300, GitHub CLI로 공개 저장소의 실행 기록 읽기. 웹훅 비밀의 로컬 읽기·출력 없음
- 게시 실행 `43329036663`: main의 `0d8338ab1186df0af9a3da445629652b105b2d91`, 성공. 로그 998줄에서 메시지 ID 필드·`wait=true` 0건, 해당 실행 첨부 자료 0건
- 같은 소스의 앞선 실패 실행 `43327480672`: 로그 1,079줄에서 메시지 ID 필드 0건. 관련 별도 알림 복구 실행의 최근 두 건은 2026-10-06, 0.15.0 전송 근거에서 제외
- 당시 전송 코드: 응답 본문 폐기, 메시지 ID 저장 없음. 조회 결과와 코드의 일치, Discord 화면의 메시지 자체 부재를 뜻하는 근거는 아님
- [Discord 공식 웹훅 문서](https://github.com/discord/discord-api-docs/blob/main/developers/resources/webhook.mdx): 수정·삭제에 메시지 ID 필요, `wait=true` 전송 응답에 생성 메시지 포함. 메시지 ID 없는 기존 안내의 지정 수정·삭제 경로 없음
- 선택: 기존 ID 부재 상태에서는 **0.15.0 안내 정정** 표시의 새 글 전송, 이전 글의 유지관리자 수동 삭제 기록. ID를 나중에 확보한 경우 같은 실행 경로에서 원래 버전 확인 후 수정 우선, 수정 불가 시 삭제·재전송

## 구현 경계

- 기존 `release-discord-notification.yml`에 `operation=correct` 추가, 기존 복구는 `recover` 기본값과 원래 출시 소스·중복 거부 유지
- `correct`: main 수동 실행만 허용, 정정 대상 0.15.0과 원래 성공한 게시·전송 연결 확인. 공개 **정식 0.15.1** 소스에서 새 본문·지문·도구 읽기, 시험판·미발행 소스 사용 거부
- 기존 `release-publication` 환경·`AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr` 유지. 원문·지문을 develop→main PR에서 검토하는 승인 경로 유지, 로컬 웹훅 비밀 취급 없음
- ID 보유: GET으로 버전·본문 확인 → 같으면 무변경 → PATCH. 한정된 거부에서 DELETE → POST, 불명확한 네트워크 실패·429·500의 자동 삭제·재전송 없음
- ID 부재·확인 불가: 명시 정정 글 POST, 수동 삭제 필요 기록. 새 전송의 `wait=true`와 새 메시지 ID·소스·지문·수신 상태 보존, 웹훅·메시지 본문의 수신 기록 제외
- 리디렉션 거부·응답 크기 제한·전체 멘션 차단. 정확한 검토 소스별 중복 수신 기록 거부, 전송 불확실 시 맹목적 재실행 금지

## 실행 순서

2026-11-14 유지관리자 선택 "Send it early": 0.15.1 정식 전 조기 정정. 원래 메시지 ID `1570581574905499885` 제공, 제자리 수정 우선

1. 문서·워크플로만의 `fix/discord-0.15.0-correction`을 develop에서 분기. 정정 워크플로·전송 도구·시험·0.15.0 안내와 지문·정정 대기 목록·업데이트 안내 Skill, 제품 바이트 없음
2. develop 병합 뒤 develop→main PR. 기존 `main-pr` 승인 경로
3. main에서 0.15.0 정정 수동 실행. `correct`는 원래 게시 실행과 main의 검토된 원문을 사용, 수신 기록 이름은 버전 단위
4. 정정 수신 확인 뒤 정정 대기 목록 비우기
5. 0.15.1 정식은 그 뒤 일반 경로. 목록이 남아 있으면 게시의 안내 동시 전송 거부, `recover`는 정정 수신 기록 확인

- 순서 근거: 2026-11-14 유지관리자 지시. "the 0.15.0 update note on discord must be either edited or sent again BEFORE the 0.15.1 update note hits. To keep the version update note ordered."

```powershell
gh workflow run release-discord-notification.yml --ref main -f operation=correct -f product_version=0.15.0 -f publication_run_id=43329036663 -f message_id=1570581574905499885
```

- ID로 원래 글을 읽고 버전 제목이 같을 때만 PATCH. 수정 불가 응답이면 삭제 뒤 정정 글, 삭제도 불가하면 정정 글과 유지관리자 수동 삭제 기록
- 웹훅 주소·토큰 입력 금지, 비밀은 GitHub 환경 안에서만 사용

## 실행 결과

- 2026-11-14 [PR #96](https://github.com/gvm1229/aigent-hive/pull/96) develop 병합, [PR #97](https://github.com/gvm1229/aigent-hive/pull/97) main 병합 `72e640b2`. 두 PR의 CI 통과, develop 푸시 실행은 같은 커밋의 PR 실행에 밀려 취소된 뒤 재실행
- [정정 실행 43554636724](https://github.com/gvm1229/aigent-hive/actions/runs/43554636724) 성공. 수신 기록: `operation: edited`, GET 200·PATCH 200, 원래 메시지 `1570581574905499885` 제자리 수정, 수동 삭제 불필요, 승인 지문 `sha256:1759aa37…96d3`
- 수신자의 실제 열람은 미확인
- 정정 대기 목록 비움. 0.15.1 정식은 안내 동시 전송 가능, recover도 0.15.0 정정 수신 기록 확인 통과

## 증명 범위와 한계

- Windows의 로컬 HTTP 모의 서버로 수정·같은 본문·삭제 후 교체·ID 부재·권한 거부·잘못된 대상·일시 실패·리디렉션·잘못된 응답·승인/지문 거부 검사
- 실제 Discord 수정·삭제·전송, GitHub의 보호 환경 실행, 수신자의 열람은 미실행·미검증. 현재 태그·게시·병합 제외와 미발행 0.15.1 소스의 main 반영 전 경계 유지
