# 0.14.0 전역·프로젝트 대화형 설정

> Plan version: 0.14.0
> Scope: product

## 항목

- [x] [S140-001] 공통 질문 엔진·터미널/기계 입력 분리
  - state: complete; evidence: repo:docs/research/cli-project-validation-0.14.0.md#sha256:08b1fb1500f31f7e75d15cfa1a6378ddf3b96b24375cff3a0632712032fccf80
- [x] [S140-002] 한국어/영어 전역 초기 설정·재설정
  - state: complete; evidence: repo:docs/research/cli-user-settings-0.14.0.md#sha256:029f15d8f36dbc94ad6e63444e3ba8ad22ae8ef1566a0f2a6080d77bb110f636
- [ ] [S140-003] AI 없는 프로젝트 설정·기능 확인·보존
  - state: agent-owned; depends: S140-001
- [ ] [S140-004] 얇은 AI 연결과 제한된 결과 반환
  - state: agent-owned; depends: S140-002,S140-003

## S140-001 — 공통 엔진

- M06 `54c4d6dc`: [언어·입력 계약과 근거](../details/setup-language-input-0.14.0.md)

- 소유: `user_setup.rs`, `main.rs::run_setup`, 기존 설정 스키마·질문 목록. 질문 순서·조건·기본값·검증·미리보기·적용을 새 setup_engine 모듈로 추출, 기존 renderer·소유권 검증 재사용
- Question 자료: id, revision, type(bool/enum/list/text/integer), en/ko 설명, options, current/default, source, 조건, 검증 규칙. 코드와 Skill의 독립 질문 목록 제거, 정본 catalog에서 CLI/describe/문서 생성
- 터미널 입력은 번호 선택·예/아니요·텍스트 입력 제공. 잘못된 값은 해당 질문만 반복. 입력 종료/Ctrl-C는답변 취소, 이전 답안 보존. 컬러·화면 지우기 없는 단순 줄 기반 UI를 기본으로 선택
- 같은 엔진에 터미널/답안 파일/AI 구조화 입력 적용. 기존 전체describe 유지, 새section 필터와show/resume 지원. 기계 결과stdout·사람 안내stderr 분리
- 초기 질문 자료는 다음 단계에 필요한 값만 반환. 정상 요약에 전체 스키마·파일 목록·지문·매 질문 전체 답안 포함 금지. 상세 진단은 명시한 describe/진단 경로로 제한
- 적용 직전 선택·변경·보존·다운로드를 한 번 요약. 기존 설정 요청 범위 안의 기계적 검증은 반복 승인 없이 처리, 외부 연결·추가 설치·후크는 기존 개별 동의 유지
- 검증: 입력 모의시험의 전체 분기와 실제 PTY 질문/Enter/취소/EOF/잘못된 값/재개. 잘못된 기계 입력에는 질문 대기 없이 오류

## S140-002 — 전역 설정

- M04 `06d35f5a`: [벡터 선택·별도 동의·재개 근거](../details/vector-choice-0.14.0.md)
- M05 `3623df57`: [앱 선택 해제·보존·재개 근거](../details/host-deselection-0.14.0.md)

- 진입: S140-001 완료. 기존 user-setup의 언어·모드·위키·사용자 맥락·성격·호스트·Judge·Skill·사용량·알림·vector 선택을 같은 엔진으로 연결
- 언어 미선택 시에만 `한국어 / English` 첫 질문. 저장된 언어 존재 시 재사용, 사용자가 언어 변경을 요청한 경우만 재질문. 질문·오류·진행·복구·완료 요약 모두 같은 언어
- 빠른 설정은 권장 묶음을 보여 준 뒤 한 번 수락. 맞춤은 관련 질문만 한 항목씩 표시. 이미 답한 선택형 질문 반복 금지
- 신규 모델·Fast 모드 추천은 안내만 제공. 사용자 호스트 설정 자동 편집·모델 호출·추론 강도 자동 변경 없음
- 파일 탐색/진단은 사용자 범위에 한정. 전역 설정에서 현재 저장소 조사나 프로젝트 설정 자동 시작 금지
- [색인 범위 분리](../../research/cli-index-lifecycle-0.14.0.md)와 [위키 재활성화 동의·종료 복구](../../research/cli-index-reactivation-0.14.0.md) 구현. 일반 파일·AI 입력의 승인 전달, Windows 게시 여섯 지점 재개 확인. 개별 파일 교체 동작 내부 중단·다른 운영체제 실제 CLI 수용은 별도
- 완료: 두 언어의 새 설치·일부 변경·전체 재검토·진행 재개를 CLI만으로 완료, 같은 입력의 기존 답안 파일 경로와 최종 결과 일치

## S140-003 — 프로젝트 설정

- `fdcf5176`: [최소 답안의 Windows 파일/Agent 적용·검증·재개](../../../tests/results/runs/20261026T054430-ffbbc9678d96.md), 전체 범위와 한계는 해당 근거


- M03 실행: [파일 입력의 공통 적용·복구](../details/project-file-input-0.14.0.md). 기존 명령·오류 분류 유지와 공통 확정 처리 재사용

- 진입: S140-001 완료. `hive setup --target <dir>`에서 기존 `--answers/--capabilities` 자동 구성, 명시한 target만 읽기. source marker·타사 지침·비밀 경계 유지
- 저장소 이름·경로·알려진 manifest 유무만 결정적으로 조사. 프로젝트 종류 general/custom과 사용자 선호는 추정 대신 질문. 전역 설정 부재 시 같은 과정에서 전역 필수 언어·선택을 먼저 완료
- 빠른 설정은 전역 언어·위키·성격 상속과 기존 daily-work 묶음. 영구 역할 없음·외부 지식 승격 없음·선택 후크 없음이 기본. 지식 포함 경로는 명시 선택 전 빈 목록, 비밀 제외 규칙 유지
- 맞춤 설정은 기존 지원 필드 전부 입력 가능. 역할은 기존 서명된 역할 정의 선택 또는 구조화된 직접 입력으로 추가, 파일 범위·책임·검증 항목 표시. CLI의 자연어 역할 생성 없음, AI 제안은 별도 선택형 입력
- 호스트 공개 실행 파일과 부작용 없는 version으로 존재 확인, 서명된 릴리스의 정적 지원표와 신뢰할 수 있는 호스트 기능 자료만 사용. 실행 파일 존재만으로 subagent/후크/자동 실행 지원 추론 금지
- 기능 상태는 supported/best-effort/unsupported/unverified 구분. 기본 지침·설정 설치와 실제 호스트 실행 자격 검증 분리. 증거 없는 선택형 실행 기능은 설치 선택에서 제외·이유 표시, 사용자가 필수로 요구한 기능은 그 부분 적용 전 차단
- 재설정: 이전 답안·역할 본문·할당·인계 보존. 이전판 투영은 기존 project upgrade의 scan→preview→apply 경로, 안전하게 병합하지 못한 충돌은 원문 보존·해당 경로 보류. AI 없이는 불가능한 의미 병합을 성공으로 표시 금지
- 완료: 전역 상속/맞춤·빈 역할·사용자 역할·다중 manifest·미설치 호스트·unverified 기능·기존 사용자 수정·새 프로젝트를 실제 CLI로 수용

## S140-004 — AI 연결

- M17 `44ced505`: [작은 답안·제한 요약·CLI 연결과 검증 한계](../details/0.14.0-delivery-milestones.md#m17m18-현재-검증). Windows 임시 루트·가짜 앱 적용/재개 통과, M18 연결 오류 모의 `41f59243` 통과, 실제 모델·다른 OS 수용 별도

- user-setup/project-setup Skill은 시작 조건·한 번의 상태 조회·CLI 실행/안내·결과 확인만 소유. 질문 번역·매 답변의 전체 YAML 작성·반복 schema 조회·일반 설정의 별도 Judge/하위 에이전트 제거
- 호스트의 사용자용 대화형 터미널이 있으면 해당 경로 사용. 비대화형 도구 호출만 가능하면 단일 CLI 실행 명령과 이유 안내, 숨은 입력창을 기다리거나 AI 질문 반복으로 자동 전환 금지
- AI가 이미 사용자 선택을 받은 경우 같은 엔진의 답안 입력으로 전달. 추측한 선택을 explicit로 기록 금지. 정상 경로의 AI 역할은 시작과 결과 설명으로 제한
- 기본 모델 안내: 이용 가능한 가벼운 모델·일반 속도, Codex에서는 검증한 Luna low/medium 안내. xhigh·Fast 기본 권고 제외, 다른 호스트에 Codex 모델명 노출 금지
- 현재 정본 Skill과 현재 투영만 생성기로 갱신. 역사 bases와 사용자/타사 바이트 보존
- 검증: CLI/답안/AI 입력의 의미 동등성, language·source 표시, 중복 질문0건, 정상 출력량 제한. 실제 모델 비용과 정확성은 Q140-001에서 별도 수용
