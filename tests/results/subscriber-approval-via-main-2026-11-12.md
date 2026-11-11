# 구독자 main PR 승인 검증

- 실행일: 2026-11-12, 호스트 Codex, Windows 11, Python 3.12.0
- 구현: `1570b621`, 허용된 `chore/subscriber-approval-via-main`만 push
- 제품 파일 변경 없음, 버전·공개 시험 추가 없음

## 통과

| 명령·범위 | 결과·증명 | 한계 |
| --- | --- | --- |
| `python -m unittest tests.conformance.release.test_stable_release_discord_notification -q` | 25개, 종료값 0, 건너뜀 없음. 허용 변수·누락·다른 값·지문·파일명·한 줄 제약과 배너→안내 순서 | 로컬 HTTP 수신기, 실제 Discord 제외 |
| `python scripts/test-lanes.py --lane release` | 151개, 종료값 0, 10개 건너뜀. [기록](runs/20261111T161538-34693b9e22bd.md) | 아래 운영체제·실행 파일 제약 |
| `python scripts/test-lanes.py --lane documentation` | 163개, 종료값 0, 2개 건너뜀. [기록](runs/20261111T161549-9213e0a18391.md) | 아래 심볼릭 링크 권한 제약 |
| `python scripts/test-lanes.py --lane security` | 103개, 종료값 0, 5개 건너뜀. [기록](runs/20261111T161559-9d9fbe822145.md) | 아래 운영체제·권한 제약 |
| `python scripts/check-human-documentation-style.py --all --output json` | 종료값 0, 지적 0건. 기존 안내 초안 17행의 지문 결합 예외 | 안내문 재작성·문구 승인 증명 제외 |
| 두 workflow의 모든 `run`을 Git Bash의 `bash -n`에 입력 | 21개 구문 검사 종료값 0 | GitHub Actions 의미 검사·실제 실행 제외 |
| `python scripts/check-markdown-links.py --output json`, `python scripts/check-plan-state.py` | 종료값 0 | 저장소 문서 연결·계획 정합성 범위 |

## 건너뜀·미실행

- actionlint: 설치된 실행 파일 부재로 미실행. Bash 구문 검사와 Python workflow 계약 검사를 수행, actionlint 자체 검증 증명 제외
- 출시 검사 10건: macOS 설치 검사 7건은 Windows에서 macOS 설치 시험 자료 사용 불가, POSIX 진입점 1건은 Windows 미지원, 공개 0.13.2-test.2 갱신 1건은 명시 선택된 고정 Windows 실행 파일 부재, 연결된 실행 파일 거부 1건은 Windows 심볼릭 링크 권한 부재. 해당 운영체제·공개 실행 파일·심볼릭 링크 동작 증명 제외
- 문서 검사 2건: Windows 심볼릭 링크 생성 권한 부재. 문체 검사기·시험 결과 정리 도구의 심볼릭 링크 탈출 반례 미실행, junction 검사는 별도 실행
- 보안 검사 5건의 개별 대상·이유는 아래 목록. 이 호스트에서 미실행한 해당 동작의 증명 제외
- GitHub 환경 변수·비밀값 변경, 보호 브랜치 통합, PR 생성, workflow 실행·조회, 실제 Discord 전송 모두 요청 범위 밖으로 미실행. 최초 1회 변수 설정은 유지보수자 소유의 별도 작업

## 이전 검사와 복구

- 최초 문서 검사: 변경된 참조의 출처 지문 불일치로 실패. 해당 영어·한국어 사실 쌍의 지문 갱신 후 최종 검사 통과
- 초기 출시·문서 실행 중 문서 변경 발생. 초기 기록은 당시 진단용으로 보존, 완료 근거는 소스 변경 없는 위 최종 기록만 사용
- Git Bash 탐색 경로 보정 후 알림 검사 25개 전부 실행. 제품 스크립트·workflow의 추가 변경 없음

- `test_generated_directory_symlink_escape_rolls_back_every_namespace`: Windows의 심볼릭 링크 생성 권한 부재, WinError 1314

- `test_generated_file_symlink_escape_rolls_back_every_namespace`: Windows의 심볼릭 링크 생성 권한 부재, WinError 1314

- `test_shared_agents_symlink_escape_rolls_back_every_namespace`: Windows의 심볼릭 링크 생성 권한 부재, WinError 1314

- `test_executable_change_after_version_fails_closed`: Unix 실행 파일 식별용 시험 자료, Windows 미지원

- `test_symlink_swap_cannot_change_the_qualified_executable`: Unix 심볼릭 링크 식별용 시험 자료, Windows 미지원

## 브랜치 전달

- 구현 `1570b621`·문서 `d9259553`의 지정 원격 브랜치 push 성공
- 보호 브랜치 통합·PR·GitHub 설정·workflow 조작 없음. 다른 출시 세션 작업 트리 접근 없음
- 완료 후 남은 이번 범위의 에이전트 소유 작업 없음. 환경 변수 설정·실제 발송은 범위 밖
