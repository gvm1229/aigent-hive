# 0.11.2 후속 기능의 구현 가능성

- 조사일·출처 조회일: 2026-10-05
- 환경: Windows·Codex, `agy --version` 1.1.18, Antigravity IDE 실행 파일 2.5.5
- 실제 실행: 로컬 버전·도움말·실행 파일 정보 조회, 저장소 코드와 기존 결과 검토, 공식 문서 조회
- 미실행: 모델·하위 작업 호출, 로그인 자료 열람, 전역 설정 변경, 새 훅 설치, 실제 사용량 전달, macOS 재현
- 로그인 상태: 현재 사용자의 Antigravity 로그인 완료 설명. 계정 조회나 자격 증명 검사를 통한 독립 확인 제외
- 판정 구분: 공식 계약 존재, 현재 설치본 제공, 실제 실행 성공의 별도 증명
- 일반 문서의 게시·수정일 미표시. 조회일 기준 계약이며 설치 버전 전체에 대한 소급 보장 제외

## 사용자 결정의 대응

| 원래 항목 | 현재 결정 | 소유 |
| --- | --- | --- |
| 후속 1·추가 1: Claude 실제 호스트·구독 사용량 | 환경 부재로 대기 유지 | [Claude 후속](../plans/backlog/claude-host-acceptance.md) |
| 후속 2: 15초 감시·자동 중단 | 0.11.2 편입, 지원 연결 검증 선행 | HCT112-002·003 |
| 후속 3: Notion 정본 | 폐기 | [ADR-0025](../decisions/ADR-0025-0.11.2-scope.md) |
| 후속 4: 작업 자동 분담 | 0.11.2 편입, 호스트 증명 기준 유지 | HCT112-004 |
| 후속 5: 게시자 서명 | 외부 승인 대기 유지 | [서명 후속](../plans/backlog/platform-signing.md) |
| 후속 6: 벡터 | 완료·종료, 엔진 비교 문서 보존 | [비교 참고](../plans/backlog/alternative-indexes.md) |
| 후속 7: Obsidian | 기본 Markdown 사용 가능, 별도 플러그인 과제 종료 | [검토 결과](../plans/backlog/obsidian-integration.md) |
| 후속 8: Graphify 잔여 | 0.11.2 편입 | GPH112-001–003 |
| 추가 2: Antigravity 사용량 | 공식 출력 발견, 구현·실제 수용 편입 | HCT112-001 |
| 추가 3: 실제 호스트 범위 | Windows 우선 확장·한계 검증 편입 | HCT112-005 |
| 추가 4: macOS 오류 | Windows 진단 준비, macOS 실제 수용 마지막 | QLF112-003·005 |
| 추가 5: 토큰 집계·예산 | 추가 조사·수집기 보완 편입 | QLF112-001 |
| 추가 6: Windows 오류 5 | 원인 분리·격리 재현 편입 | QLF112-002 |
| 추가 7: 발표 자료 | 폐기, 로고 완료 | [폐기 기록](../state/artifacts/aigent-hive-marketing-deck.md) |

## Antigravity 사용량

- 공식 [상태 표시 JSON](https://antigravity.google/docs/cli/statusline/): `quota`의 모음별 `remaining_fraction`·`reset_time`, 대화 식별자·버전 제공. 상태 변경 때 표준 입력 전달
- `stack_with_default`로 기본 표시와 병행 가능. 기존 사용자 명령과의 병합·설치 권한은 별도 문제
- 공식 [headless 계약](https://antigravity.google/docs/cli/headless): `usage`는 토큰 수, `result.usage`는 대화 누적. 단계별 수치와 누적 수치의 중복 합산 금지. `/usage`는 직접 처리하는 텍스트 응답이며 스트리밍 입력에서 미지원
- 로컬 `agy --help`: `--output-format json|stream-json`, `--model`, `--effort` 제공. 도움말 성공은 로그인·quota 전달 성공의 증명 제외
- 현재 코드: `usage_control.rs::parse_capture`는 Claude만 허용, `read_claude_capture_snapshot` 재사용 구조 존재. Antigravity 분기는 `SensorError::Unsupported` 고정
- 권고: 같은 수신·정제·최신성 검사 구조에 Antigravity 어댑터 추가. 원문·이메일·경로·대화 식별자 저장 금지, 계정·대화 지문과 정제된 모음별 값만 보존
- 결손: 설치 1.1.18의 실제 출력, 모음 식별자·단위·0~1 범위·초기화 시각, 오래된 값과 누락 값 판정. 고정 7일 창 추정 금지
- IDE 2.5.5 로그인은 CLI 상태 표시 계약의 증명 제외. 기존 “공식 출력 없음” 판정은 현재 CLI 문서 기준 대체, IDE까지 일괄 지원 주장 금지
- 15초 주기 보장은 상태 변경 콜백과 별개. 갱신 없는 과거 관측의 반복 사용 금지

## 자동 중단·작업 분담

- 공식 [Codex App Server](https://learn.chatgpt.com/docs/app-server): `turn/interrupt`와 중단 완료 사건, 사용량·모델 변경 사건 제공
- 현재 `usage/codex_native.rs::exchange`: 계정·사용량 조회용 별도 서버 생성. 현재 데스크톱 실행의 제어 연결로 사용 불가
- 권고: 현재 호스트가 제공한 연결에 정확한 대상·실행·제어 세대를 결합한 선언형 요청. Hive의 새 모델 서버·강제 프로세스 종료·사용자 입력 흉내로 대체 금지
- 현재 대화에 노출된 자식 중단 도구는 자식 작업용. 부모 전체 자동 중단이나 주기 실행 연결의 증명 제외
- 공식 [Codex 하위 작업](https://learn.chatgpt.com/docs/agent-configuration/subagents): 역할별 정의·모델·추론 설정 제공. 요청 설정과 실제 실행 영수증의 구분 필요
- 공식 [Antigravity 하위 작업](https://antigravity.google/docs/subagents): 비동기 작업과 `inherit|flash|pro` 모델 등급. CLI의 모델·추론 인수가 자식의 정확한 모델·추론 설정과 같다는 추정 금지
- [Antigravity 원격 제어](https://antigravity.google/docs/remote-control)는 사용자 기기 간 조작 기능. Hive가 현재 실행에 연결할 공개 로컬 중단 API의 대체 증명 제외
- 권고: `custom_agent_cli.rs`·`native_workflow.rs`의 기존 정의·검증 경로 확장, 실제 역할·모델·추론·정의 지문·결과 지문 확인. 수신 불명확은 재전송 없이 별도 상태 유지
- 구현 가능 범위: 선언·상태·실패 판정·호스트별 기능 검사. 실제 15초 감시·현재 작업 중단·엄격한 자동 분담 활성화는 호스트 연결 증명 선행

## 호스트 검사 범위

- 공식 [Antigravity 훅](https://antigravity.google/docs/hooks): `PreToolUse`의 도구명·인수와 허용·거부 응답, 검사 시간 제한 제공
- 현재 `policy/native.rs`의 파일 도구 변환과 [기존 수용](native-host-qualification-0.11.0.md) 재사용. Windows Antigravity 자식의 정상 편집·보호 편집·검사기 부재·시간 초과부터 검증
- 셸 문자열 분석만으로 모든 쓰기 효과의 차단 보장 불가. 기존 터미널·MCP·다른 편집기는 각각 실제 변경 경계와 지원 여부 확인
- 구현 후보: 도구별 명시적 지원 표·진단·상속 검사. 미지원 경로를 보호 성공으로 합산하는 처리 제외
- 다른 운영체제 실제 앱 검증은 Windows 완료 뒤 순서. macOS 부분은 QLF112-005에 결합

## Graphify 잔여 범위

- Hive 고정 버전 `graphifyy==0.9.47`: [증분 불일치·전역 격리·문서 의미 추출의 기존 실패](graphify-0.10-feasibility.md)
- 조회한 upstream `v8` 브랜치의 [패키지 선언](https://raw.githubusercontent.com/Graphify-Labs/graphify/v8/pyproject.toml): 0.9.76·Apache-2.0. 가변 브랜치 내용이며 새 배포물의 채택·검증 근거 제외
- [upstream 안내](https://github.com/Graphify-Labs/graphify): 코드의 로컬 구문 분석과 문서의 호스트 스킬 경로 구분. 문서 추출의 모델 의존성 유지
- 권고: 검토할 정확한 버전·커밋·의존 파일을 먼저 고정, 기존 실패 표본 재현. 통과 전 0.9.47 교체 금지
- Hive가 범위별 파생 그래프를 소유하고 전역·비공개 모음을 물리 분리. upstream 전역 모음 호출로 권한 검사 대체 금지
- 의미 추출은 활성 호스트가 검토 가능한 근거를 생산하고 Hive가 검증·가져오기만 수행. 모델 API 호출·키·새 추론 서버 제외
- 수용: 증분/전체 정규화 동등성, 30개 관계 질문, 고유 5만 청크, 출처·권한·삭제·복구·FTS 무회귀. Windows 우선, macOS 마지막

## Obsidian의 추가 가치

- 공식 [기존 폴더 열기](https://help.obsidian.md/Files+and+folders/Manage+vaults): Markdown 폴더를 바로 보관함으로 선택 가능
- 공식 [관계 그래프](https://help.obsidian.md/Plugins/Graph+view): 문서 내부 링크를 기반으로 기본 시각 탐색 제공
- 결론: 단순 보기·편집·링크 탐색에 Hive 전용 플러그인 불필요. 기존 후속 아이디어 종료
- Hive의 출처 지문·공개 범위·의미 관계는 일반 Obsidian 그래프와 별개. 그 차이만 사용 안내에 표시, `.obsidian/` 변경·자동 동기화 추가 제외

## 토큰 예산과 Windows 샌드박스

- [기존 시험](directive-context-host-acceptance-2026-09-27.md): 완료 뒤 누적 입력 1,137,263개, 계획 1,000,000개 초과. 집계 오류를 확정하는 증거는 부재
- 수집기 후보: 대화별 시작 기준값과 마지막 누적값 유지, 단조 증가분만 합산. 부모·자식 포함 관계를 실제 사건으로 검증, 캐시 입력의 이중 합산 제외
- 사건 지연·모델 1회 실행의 초과 가능성 때문에 사후 계측만으로 엄격한 상한 보장 불가. 후속 실행 전 남은 예산 예약과 현재 실행의 중단 확인을 별도 검증
- [Windows 기존 오류](project-refresh-exposure-0.11.0.md): `helper_sandbox_lock_failed`·오류 5. CLI 종료 0과 파일 읽기 실패의 불일치
- 공식 [Windows 샌드박스](https://learn.chatgpt.com/docs/windows/windows-sandbox): 별도 사용자·권한·정책에 의존. 오류 5만으로 잠금 경합이나 특정 ACL을 단정하는 판정 제외
- 조사 순서: 동일 실행 파일·동일 격리 폴더의 직접 읽기와 샌드박스 `command/exec` 비교, 동시/단일 실행 차이·정제된 단계별 오류 확인. 모델 없이 도구 실행 가능
- 금지: 전체 접근 모드를 성공 근거로 대체, 전역 권한·방화벽·로그온 정책의 자동 변경. 필요 시 정확한 수동 조치만 제시

## macOS가 필요한 이유와 최종 순서

- [0.11.1-test.8](skill-merge-public-test8-0.11.1.md): macOS 검색 도우미 실패, 새 환경의 동일 파일 재시험 통과. 원인 미확정 유지
- 현재 `vector.rs`의 도우미 실패 변환과 `vector_runtime.py`·`vector_parent.py`의 종료 처리 조사 대상
- Windows 가능: 종료 코드·신호·제한시간·JSON 상태·정제된 오류 종류 분리, 성공 출력 후 비정상 종료 반례, 이전 정상 색인 보존 시험
- macOS 필요: 실제 arm64·Python·동적 의존 환경의 간헐 실패 재현과 수정 후 검증. Windows 모의 시험으로 해당 조건 입증 불가
- 최종 순서: Windows 구현·단위/통합·실제 호스트 수용 → Linux의 해당 공개 검사 → macOS arm64 재현·수용
- 재현 실패 시 원인 해결 주장 금지. 최초 실패·환경 차이·반복 수·성공 범위 보존, QLF112-005의 미해결 조건으로 명시
