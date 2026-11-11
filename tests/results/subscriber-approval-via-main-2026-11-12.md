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

## 후속 정리 검증

- 요청 범위: 지정 작업 트리와 `chore/subscriber-approval-via-main`만 사용. 이전 미커밋 작업은 시작 시 `2437cd55`에 이미 저장, 이전 세션 기록 부재 확인
- 최신 `origin/develop` 기준: `599685af`. rebase 대신 병합, 병합·복원 커밋 `13365b93`
- 원본 복원: `docs/archive/**`, ADR-0020, 0.13.2·0.14.0 출시 계획. `git diff origin/develop -- <대상>` 출력 0행, 저장 바이트 일치
- 출시 담당 원본 보존: `docs/plans/active/overhaul-0.14.1.md` 전체, 재정비 README의 현재 위치·OV141-011. README의 브랜치 규칙 아래 기존 승인 방식 한 줄만 교체
- 유지: 현재 `docs/facts/**`·`docs/guides/**` 승인 안내. 원본 복원으로 바뀐 출처 지문 갱신. 새 재정비 사실 쌍도 현재 main PR·일반 변수 방식으로 정합성 유지
- 검사 변경: 출시 담당자의 최신 안내문으로 불필요해진 옛 초안 17행 문체 예외 삭제. 과거 비밀값·폐기 스크립트 참조는 기존 검사 통과, 검사기·시험 대상 범위·예외 추가 없음

| 실제 실행 명령 | Windows 11·Codex 결과 | 증명 범위·한계 |
| --- | --- | --- |
| `python -m unittest tests.conformance.release.test_stable_release_discord_notification -q` | 25개, 종료값 0, 건너뜀 없음 | 로컬 수신기와 workflow 계약, 실제 GitHub·Discord 제외 |
| `python scripts/test-lanes.py --lane release` | 151개, 종료값 0, 10개 건너뜀. [실행 기록](runs/20261111T181508-9a56d21dc3b7.md) | 위 기존 운영체제·고정 실행 파일·심볼릭 링크 제약 동일. 이후 변경은 문서·문체 예외뿐, 출시 스크립트·workflow 동일 |
| `python scripts/test-lanes.py --lane documentation` | 163개, 종료값 0, 2개 건너뜀. [실행 기록](runs/20261111T181748-7aa94b7214c6.md) | Windows 심볼릭 링크 권한 부재 반례 2건 제외, junction 검사는 실행 |
| `python scripts/check-human-documentation-style.py --all` | 종료값 0, 지적·실패 0건 | 전체 저장소 문체 검사, 안내문 내용 승인 증명 제외 |
| 링크·계획 상태 검사 | 각각 종료값 0 | 문서 연결·14개 계획 항목 정합성 범위 |
| Source Wiki 색인 재생성·lint | 각각 종료값 0, 264개 문서, 경고·오류 0건 | 정본 사실 쌍과 파생 색인의 일치, 외부 서비스 동작 제외 |
| `python scripts/branch-policy.py commits origin/develop..HEAD` | 종료값 0 | 이번 브랜치 커밋 메시지의 금지된 신원·공동 작성·세션 링크 부재 |

- 문서 검사 최초 시작: 출시 검사와 시험 기록 잠금 경합으로 종료값 2. 출시 검사 종료 후 순차 실행으로 원인 제거, 최종 통과
- 문체 검사 최초 실행: 출시 담당자가 바꾼 안내문의 옛 예외로 실패. 해당 예외 제거 후 최종 통과, 안내문 추가 수정 없음
- actionlint: 현재 PATH에 설치된 실행 파일 부재, 미실행. Python workflow 계약 검사는 실행, actionlint 의미 검사 증명 제외
- 원격 출시 상태: 사용자 제공 0.14.1 출시 완료 맥락 수용. 이 작업의 원격 workflow·설정 조회 금지로 독립 재확인 제외, 출시 소유 기록은 `origin/develop` 그대로 보존
- 시험 산출물: 기존 공용 빌드와 소유 미확인 시험 자료의 삭제 제외. 이번 실행 근거는 저장소 기록으로 보존
- 최종 전달 대상: 이 브랜치만 일반 push. PR 생성·보호 브랜치 통합·비밀값·변수·workflow 조작·실제 발송 제외
