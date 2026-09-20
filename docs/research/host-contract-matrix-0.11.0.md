# 0.11.0 호스트 연결의 확인 범위

확인일: 2026-09-19. 실행 환경: Windows 11 x64, Codex 앱. `RFH-001`의 공식 규약·코드 대조이며 실제 새 대화 수용은 `RFH-002–003`의 별도 책임.

## 확인한 설치 정보

| 대상 | 버전·호출 방법 | 상태 | 증명 범위·한계 |
| --- | --- | --- | --- |
| Codex 앱 동봉 CLI | `--version`: `0.155.0-alpha.9`; `features list`: `hooks stable true` | `verified` | 로컬 명령 실제 실행. 훅 정의 신뢰·로드·차단 효과의 증명 제외 |
| Claude Code CLI | `claude --version`: `2.1.163` | `verified` | 로컬 명령 실제 실행. 최신 문서의 모든 기능이 이 버전에 있다는 추정 제외 |
| Antigravity CLI | `agy --version`: `1.1.18` | `verified` | 로컬 명령 실제 실행. 별도 IDE·2.0 앱의 기능 수용과 구분 |
| macOS·Linux 설치본 | 이번 작업에서 호출 없음 | `unverified` | 해당 운영체제 실행 환경에서 같은 검사가 필요한 상태 |

## 호스트 공식 규약과 현재 연결

| 작업 | 호스트가 제공하는 경로 | 결합·완료 확인 | Hive 실제 연결 상태 |
| --- | --- | --- | --- |
| Codex 실행·재개 | app-server의 `thread/resume`, `turn/start` | thread·turn ID, `turn/completed` 결과 | `unverified`: 현재 앱 작업에 연결된 전송·결과 수용 미실행 |
| Codex 취소 | `turn/interrupt` | 성공 응답 뒤 해당 turn의 `interrupted` 확인 필요 | `unverified`: 인터페이스 존재만으로 현재 추론 중단 가능 판정 금지 |
| Claude 재개 | CLI의 `--resume` | 지정 세션과 새 작업 결과의 연결 필요 | `unverified`: Hive가 모델 CLI를 대신 실행하는 연결 없음 |
| Claude 취소·완료 | 호스트 소유 제어와 실행 결과 | 정확한 작업 식별자·종료 상태 확인 필요 | `unverified`: 이 설치 버전에서 연결된 작업 수용 미실행 |
| Antigravity 재개 | TUI의 `/resume` | 사용자가 선택한 대화와 후속 결과 확인 필요 | `unverified`: 수동 화면 기능과 Hive의 기계적 연결은 별개 |
| Antigravity 취소 | TUI의 `Esc`, 호스트 작업 관리 도구 | 중단 요청과 실제 종료의 별도 확인 | `unverified`: 현재 Codex 작업에서 해당 호스트 제어 미실행 |

근거: [Codex app-server](https://learn.chatgpt.com/docs/app-server),
[Claude CLI](https://code.claude.com/docs/en/cli-reference),
[Antigravity CLI](https://www.antigravity.google/docs/cli/reference/).
문서는 현재 공개 규약이며 로컬 설치 버전의 실행 결과를 대신하지 않는 자료.

## 정책 훅의 지원 경계

| 연결 | 구현·등록 경로 | 상태·제한 |
| --- | --- | --- |
| Codex 파일 변경 | `.codex/hooks.json`, `PreToolUse`의 `apply_patch` | CLI 변환 `verified`; 실제 호스트 `unverified`. 정확한 정의의 신뢰 절차 별도 |
| Claude 파일 변경 | `.claude/settings.local.json`, `Write|Edit|MultiEdit` | CLI 변환 `verified`; 실제 호스트 `unverified` |
| Antigravity 파일 변경 | `.agents/hooks.json`, 세 파일 편집 도구 | CLI 변환 `verified`; 보호 대상은 `deny`, 다른 파일은 `ask`. 도구 권한을 새로 허용하는 `allow` 응답 제외 |
| 고정 규칙 전달 | Codex·Claude `SessionStart`, Antigravity `PreInvocation`의 `invocationNum=0` | CLI 변환 `verified`; 읽기·준수율·실제 전달 `unverified` |
| 문맥 압축 후 재전달 | Codex의 `SessionStart` compact 경로 | 공식 경로 존재. 현재 호스트의 실제 재전달 `unverified`; Antigravity의 별도 압축 연결 미구현 |
| 종료 알림 | Codex·Claude `Stop`의 `systemMessage`, 명시적 `--review-run` | 실행·호스트·세션 결합의 CLI 연결 `verified`, 실제 앱 전달 `unverified`. 종료의 완료 판정 없음 |
| Antigravity 종료 알림 | 현재 `Stop`은 `decision=allow` | 비재개 알림은 현재 연결에서 `unsupported`. 알림을 위해 `continue`를 반환하는 방식 제외 |
| 셸·기존 터미널 입력·MCP·다른 편집기 | 이번 파일 변환기의 등록 범위 밖 | `unsupported`는 이 Hive 변환기의 범위. 호스트 자체의 훅 지원 여부와 구분 |

근거: [Codex 훅](https://learn.chatgpt.com/docs/hooks),
[Claude 훅](https://code.claude.com/docs/en/hooks),
[Antigravity 훅](https://www.antigravity.google/docs/hooks).
Codex의 기존 `write_stdin`에는 새로운 사전 검사가 없으므로 셸 시작 검사만으로 후속 입력 보호 주장 금지.
Antigravity 종료 `reason`의 모델 전달은 `continue` 조건이므로 중립 종료 알림의 근거로 사용 불가.

## 중복·복구·증거 처리

- [run.rs](../../crates/hive-cli/src/run.rs): 계획·상태·소유 호스트의 연속성 검사, 자동 준비의 역할·작업 개정·내용 지문 결합. 준비 성공은 호스트 실행 성공과 구분
- [orchestration.rs](../../crates/hive-core/src/orchestration.rs): 이벤트 순서·선행 지문·제어 세대 검사. 동일 결과 재전달은 중복 처리, 다른 내용·늦은 결과·충돌은 거부 또는 격리
- 전송 여부가 불분명한 `dispatch-uncertain`은 임의 재전송하지 않고 실제 호스트 결과를 확인하는 상태. 취소 요청은 취소 완료의 대체 근거에서 제외
- [configure.rs](../../crates/hive-cli/src/policy/configure.rs): 명시적 미리 보기와 지문 승인, 외부 설정 보존, 등록·제거·복구·단계별 미확인 진단. 설정 존재와 보호 활성의 구분
- [실행·결과 회귀](../../tests/results/runs/20260919T052051-1aa19aaa6ca3.md): Windows 48개 중 45개 통과, Unix 프로세스·POSIX 권한 등 3개 제외. 결정적 계약과 합성 자료의 증명, 실제 호스트 완료·취소의 증명 제외
- [훅 변환·등록 회귀](../../tests/results/runs/20260919T055402-839901f02c26.md): Windows 10개 통과. 생성 명령과 입력·출력 검증이며 앱 신뢰·자식 실행·시간 초과 수용은 미검증
- [실행별 안내 회귀](../../tests/results/runs/20260919T060857-b7ac1e0676c3.md): Windows 12개 통과. 생성 명령에서 다른 세션의 알림 없음·반복 호출의 쓰기 없음·검토 후 알림 종료·등록 철회 확인. 실제 앱 실행 증명의 대체 자료에서 제외

`RB104-004`의 15초 감시·진행 중 추론 중단은 여전히 `unverified`.
검증된 인터페이스로 현재 작업을 정확히 제어한 결과가 없으므로 별도 감시 프로세스나 모델 실행을 만들지 않는 경계.

## Windows Codex 실제 파일 편집의 첫 수용

- 유지보수자 승인 아래 `tests/work/codex-policy-acceptance-1/consumer`만 사용. 별도 작업 두 개, 각각 `apply_patch` 2회. 전역 설치·새 작업 브랜치·모델 CLI 실행 없음
- [기준선](../../tests/results/runs/20260919T085959-fa3935a9eaa1/baseline.json): 일반 파일과 보호 파일 편집 모두 성공. 지문으로 실제 변경 확인, 다음 시험 전 원래 바이트 복원
- [프로젝트 등록](../../tests/results/runs/20260919T090048-42ece749eaf9.md) 뒤 [실제 차단 시험](../../tests/results/runs/20260919T090551-7e11819b212d.md): 보호 파일도 편집 성공, 차단 수용 실패
- 동시점 CLI 버전 조회 `0.155.0-alpha.9.2`, 프로젝트 신뢰와 훅 기능 활성 확인. 개별 훅 정의 신뢰·로드·검사 실행 단계는 미확인; 특정 단계의 실패로 원인 확정 불가
- [등록 명령 직접 실행](../../tests/results/runs/20260919T090748-1327714bcbd1.md): Windows 명령의 표준입력·JSON 응답 정상. 일반 파일 중립·보호 파일 거부, 오류 출력 0바이트. 호스트 호출·신뢰 증거의 대체 자료에서 제외
- 공식 `/hooks`의 정확한 정의별 신뢰 검토가 다음 확인 단계. 유지보수자가 직접 조작을 나중으로 미뤄 실제 앱 재시험 보류. 신뢰 우회·전역 설정 변경 없음
- 설정과 복사한 실행 파일은 격리 폴더에 보존. 등록·진단 성공만으로 보호 활성·`HK-003` 완료 주장 금지
