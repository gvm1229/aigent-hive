# 0.11.2 호스트 지원과 연결 검토

- Windows 개발 빌드의 검토 자료. 공개 시험·현재 사용자 설치·전역 설정 적용 미실행
- 구현된 명령과 실제 호스트의 동작을 별도 판정. `supported`는 지정 범위의 증명, `partial`은 일부만 증명, `unsupported`는 현재 Hive 경로 없음, `unverified`는 실제 증명 부족
- 현재 기록: [구현·시험 근거](../research/implementation-evidence-0.11.2.md). 검증 밖 기능은 비활성·대기 유지

## 기능별 현재 범위

| 기능 | Windows Codex | Antigravity CLI 1.1.18 | Antigravity IDE 2.5.5 |
| --- | --- | --- | --- |
| 구독 사용량 | 기존 설치의 작업 시작 조회 통과. 새 수신 기능과 별도 | `partial`: 실제 `/usage` 두 모음 확인·수신 검사 구현, 콜백 전달 미검증 | `unverified`: CLI 계약의 적용 증거 없음 |
| 현재 실행 중단 | `unverified`: 현재 앱 실행의 인증된 연결 없음 | `unsupported`: 현재 Hive 제어 경로 없음 | `unsupported`: 현재 Hive 제어 경로 없음 |
| 15초 감시 | `unsupported`: 관측·중단을 결합한 현재 실행 경로 없음 | `unsupported`: 상태 변경 콜백은 주기 감시 증명과 별도 | `unsupported` |
| 엄격한 자동 분담 | `unverified`: 실제 역할·모델·결과의 독립 증명 필요 | `unverified`: 모델 등급을 정확한 모델 ID로 치환 불가 | `unverified` |
| 저장 뒤 의미 관계 | `partial`: 현재 대화의 제한된 입력 검토·적용 확인, 새 대화의 자동 발견 미검증 | `unverified`: 실제 모델 검토·적용 미실행 | `unverified` |

기능 명세 형식 2의 선언만으로 실행 권한 부여 없음. 형식 1의 새 기능은 `unverified` 처리. 사용량 조회용 별도 Codex 서버를 현재 앱의 중단 연결로 사용하는 경로 제외.

## 파일·셸·MCP·자식 검사

| 경로 | Hive의 판정 범위 | 실제 호스트 증거 |
| --- | --- | --- |
| Codex `apply_patch` | 지정 파일 변경의 사전 검사 | [0.11.0 Windows 자료](../research/native-host-qualification-0.11.0.md)의 부모·자식 한 사례. 새 버전의 모든 실행 증명으로 확대 금지 |
| Antigravity 파일 도구 | `write_to_file`·`replace_file_content`·`multi_replace_file_content`의 지정 파일 검사 | 같은 과거 자료의 IDE 2.5.5 부모 사례. 자식 상속·CLI 1.1.18 전달은 미검증 |
| 셸·기존 터미널 입력 | `unsupported`: 파일 변환기의 선택 범위 밖 | 실제 셸 쓰기 가능의 과거 관측 유지. 모든 쓰기 보호 주장 제외 |
| MCP·다른 편집기 | `unsupported`: 해당 효과의 Hive 변환 경로 없음 | 실제 차단 성공으로 집계 제외 |
| 검사기 부재 | 실행된 바깥 명령이 명시적 JSON 거부를 반환하는 경로 | 과거 실제 부모 호스트에서 거부 확인, 같은 버전·정의·도구 범위에 한정 |
| 시간 초과·손상 응답·비정상 종료 | 호스트의 오류 처리와 Hive의 명시적 거부 구분 | 과거 편집 진행 관측. 오류만으로 쓰기 차단 보장 불가 |

0.11.2의 합성 계약·보안 검사는 변환기와 파일 보존 규칙의 증명. 새로운 대화의 훅 로드·신뢰·자식 상속은 실제 호스트에서 별도 확인 필요. 실제 시험 전에 대상 폴더·도구·변경 전후 바이트·복원 대상을 고정.

## Antigravity 상태 표시 연결 검토

대상은 Antigravity를 기본 호스트로 설정한 소비자 프로젝트. 현재 수신 명령의 Source Workspace 전역 연결 지원 주장 제외. 다음은 실제 TUI 전달을 검증하기 위한 초안이며, 현재 사용자 설정에 자동 적용하지 않는 자료.

1. 검증할 Hive 실행 파일의 버전·지문과 소비자 프로젝트 확인. CLI 1.1.18의 두 모음 계약에 한정
2. 사용자가 선택한 연결 스크립트 경로와 정확한 내용을 미리 제시. 기존 `statusLine` 유무·내용·활성 상태와 복원 방법을 함께 확인
3. 기존 사용자 명령이 있는 경우 그대로 유지. 입력을 두 명령에 나누는 전달 방식이 검증되기 전까지 교체·자동 결합 제외
4. 기존 사용자 명령이 없는 경우의 검토 예시: 입력을 파일에 저장하지 않고 Hive에 전달, 결과 JSON은 화면 출력에서 제외, 종료 코드는 유지

```bat
@echo off
"<검증한 hive.exe의 절대 경로>" usage capture --host antigravity --target-from-stdin --stdin-json --output json >nul
exit /b %errorlevel%
```

[공식 상태 표시 계약](https://antigravity.google/docs/cli/statusline/)의 설정 대상은 `~/.gemini/antigravity-cli/settings.json`. 기본 표시와 함께 쓰는 검토 예시:

```json
{
  "statusLine": {
    "type": "command",
    "command": "<검토한 연결 스크립트 경로>",
    "stack_with_default": true
  }
}
```

위 예시는 적용 승인의 대체 자료가 아닌 변경 제안. 다른 설정 필드·원래 사용자 명령의 변경 제외. 실제 전달에서는 두 모음의 비율·초기화·수신 시각·계정/대화 결합 확인 필요. 현재 공개 명령 응답 확인만으로 상태 표시 전달 성공 처리 금지.

복원은 연결 직전의 `statusLine` 내용·활성 상태로 되돌리는 절차. 원래 해당 항목이 없었다면 이번에 추가한 항목만 제거. 그 밖의 설정 바이트와 사용자 명령 보존. 연결 시험·복원 후 기존 표시의 실제 동작 확인 필요.

## Obsidian과의 차이

Obsidian의 기본 사용은 기존 Markdown 폴더 열기로 가능. 일반 그래프는 문서에 적힌 링크의 표시. Hive의 선택형 의미 관계는 현재 호스트가 근거를 검토한 별도 파생 자료이며, 원문에 링크를 자동 삽입하는 기능과 구분.

Obsidian 전용 플러그인·설정 변경·동기화 추가 없음. 추정 관계를 정본 사실로 자동 승격하는 경로도 없음.
