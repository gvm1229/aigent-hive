# 0.11.1 정식 게시 결과

- 기준일: 2026-10-05 한국 시간. 현재 Windows·Codex에서 GitHub·npm의 실제 공개 상태 조회
- [정식 릴리스](https://github.com/gvm1229/aigent-hive/releases/tag/v0.11.1): 일반 안정판·주석 태그·후보 소스 연결 확인
- [main CI](https://github.com/gvm1229/aigent-hive/actions/runs/37232482010)·[최종 후보](https://github.com/gvm1229/aigent-hive/actions/runs/37233123956) 성공, Windows x64·macOS arm64/x64·Linux musl arm64/x64 빌드와 이전 안정판 갱신 검증
- [실제 업로드](https://github.com/gvm1229/aigent-hive/actions/runs/37234181513)의 npm 표시 전파 시간 초과 후 여섯 정확한 버전·SHA512·latest 확인, [같은 후보의 공식 복구](https://github.com/gvm1229/aigent-hive/actions/runs/37234688620) 성공. 새 파일 재업로드·새 후보 생성 없음
- 여섯 패키지 모두 latest 0.11.1·시험판 표시 0.11.1-test.8, 로컬 후보 압축 파일과 공개 dist.integrity 일치
- 공개 무결성 확인서: `hive.release-verified`, 버전·소스 일치·배포 순서 20. 게시 작업의 체크섬·GitHub 증명·네이티브 파일 동일성 검사 성공
- 승인된 기존 배너·881자 한국어 공지의 Discord 전송 단계 성공. 스크립트의 두 전송 요청 성공 범위, 구독자 열람·향후 알림 수신의 증명 제외
- [실제 사용자 프로젝트 호출](../../docs/research/project-skill-user-acceptance-0.11.1.md)과 [test.8 수용](../../docs/research/skill-merge-public-test8-0.11.1.md)의 범위·미실행·최초 macOS 검색 실패 원인 미확정 한계 보존
- 정식 버전을 최초 행동 시험으로 사용한 설치 없음, 현재 사용자 설치는 이미 검증한 test.8 유지

## 최초 실패와 예방

- 후보 37228708411: CURRENT의 제품 버전 줄 형식 오류, 업로드 전 실패. 독립된 줄 복원·실제 버전 검증부 통과
- 후보 37230009485 성공 뒤 게시 37230948639: 영어 VERIFY-02 25단어 초과로 업로드 전 실패. 20단어 수정·현재 실제 문서의 CI 선행 검사 추가
- [전체 게시 전 점검](stable-preflight-20261005.md): 초기 실패·진단 한계·코드 변경 없는 복구·12개 증명 검증 보존
- 보호 PR [#68](https://github.com/gvm1229/aigent-hive/pull/68)·[#69](https://github.com/gvm1229/aigent-hive/pull/69)·[#70](https://github.com/gvm1229/aigent-hive/pull/70)의 필수 검사 통과 후 일반 병합, 보호 우회 없음
- 스킬 내부 loop 연결은 없는 소스 작업: 소스 계획·결정적 검사·독립 GitHub CI로 검증, 별도 실행 흐름 활성화 영수증 주장 없음

## 공개 연결 값

```json
{
  "status": "published-and-verified",
  "product_version": "0.11.1",
  "release_date": "2026-10-05",
  "published_at_utc": "2026-10-04T21:08:13Z",
  "source_commit": "a01b9e3116e901729f9c403b5a1d83d579b81199",
  "product_tree_sha256": "sha256:c4f357f2abdbc6b2231d9e50a025d1e8a35b22dd17060c0c01f3f0454873c074",
  "required_main_ci": 37232482010,
  "candidate_run_id": 37233123956,
  "upload_run_id": 37234181513,
  "recovery_publication_run_id": 37234688620,
  "accepted_test": "0.11.1-test.8",
  "acceptance_run_id": 37220067876,
  "tag": "v0.11.1",
  "tag_object": "a4b6013d7ee7b476dd1b31f2344ee618934d0b70",
  "is_prerelease": false,
  "release_sequence": 20,
  "public_integrity_receipt_sha256": "339887b41669c2b6717f5bcaaced72a51ac07fd62e3a15e984f8e79657a17900",
  "discord_step": "success",
  "subscriber_sha256": "df5e6899112ad7ec15ab41908ef34218fceafed6383d2b3da061f66a74f917c1",
  "packages": [
    {
      "package": "aigent-hive",
      "status": "ready",
      "version": "0.11.1",
      "test": "0.11.1-test.8",
      "latest": "0.11.1",
      "candidate_integrity_match": true,
      "dist_integrity": "sha512-8Z9LfFSwcchNNu0iE4BTvcAp6WkDBS3oaf6FByxXxd8Wb9MLHz0Mpl9f/h/qTXSFgEp3zm4YjZbR+hpS4DfQ9w=="
    },
    {
      "package": "@aigent-hive/win32-x64",
      "status": "ready",
      "version": "0.11.1",
      "test": "0.11.1-test.8",
      "latest": "0.11.1",
      "candidate_integrity_match": true,
      "dist_integrity": "sha512-HVoeuhhaDHqK3o4ItUw8UgWKqgo0UoZv+2cWdbYYPwnSOjEOeeCAT89oLLPI/AhQACHsWdlMjB3WprPsDWY0oQ=="
    },
    {
      "package": "@aigent-hive/linux-x64",
      "status": "ready",
      "version": "0.11.1",
      "test": "0.11.1-test.8",
      "latest": "0.11.1",
      "candidate_integrity_match": true,
      "dist_integrity": "sha512-+mK9AKPnOkRZEqlDklSOt7bHWtI54xSSqQ8guFV8oXQ8/8+czCERD9qRdHysTHoXigyZ344ebAHvE9U8eoRkjA=="
    },
    {
      "package": "@aigent-hive/linux-arm64",
      "status": "ready",
      "version": "0.11.1",
      "test": "0.11.1-test.8",
      "latest": "0.11.1",
      "candidate_integrity_match": true,
      "dist_integrity": "sha512-fjOzBZO5AmAw7tAn3vsBfOZrFOyYVzuONzHccL58vaaApL0km88ufOtAUQX/2sVuHjx/ByqOS2C4fLHBQdQMhQ=="
    },
    {
      "package": "@aigent-hive/darwin-x64",
      "status": "ready",
      "version": "0.11.1",
      "test": "0.11.1-test.8",
      "latest": "0.11.1",
      "candidate_integrity_match": true,
      "dist_integrity": "sha512-qY8qrseLjymuyrgkBWwHD9pChnCBt2NUGx7Tou9GKIlm8gWnzvi5k6JkYzksXSwp8AgqOrG8BpJR2Ks21IEvSA=="
    },
    {
      "package": "@aigent-hive/darwin-arm64",
      "status": "ready",
      "version": "0.11.1",
      "test": "0.11.1-test.8",
      "latest": "0.11.1",
      "candidate_integrity_match": true,
      "dist_integrity": "sha512-G5JkL19QZeQGYoQwTzS8TNX096F9Rs3mhBhKXmJtBPnHxvjBnuUcONvXW0uJvS+k6yJZnabYe1TgPSjG6FRJpg=="
    }
  ]
}
```
