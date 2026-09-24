# 현재 상태

- [지침 복구안](../plans/directive-context-recovery-0.11.0.md): 조사 완료·구현 전
- [APC 계획](../plans/active/all-public-project-refresh-0.11.0.md): 안정판 9개 갱신·복구와 test.7 수용 완료. main·develop만 유지, 커밋 보존

- [PRF 계획](../plans/active/project-refresh-exposure-0.11.0.md): test.6 공개 수용·실제 스킬 자연어 갱신 확인. 예시 적용·보존·develop PR 통합 완료

- 작업 브랜치: `develop`; 통합: `develop`; 안정판: `main`
- 공개 수용 완료: `0.11.0-test.7`; 공개 안정판: `0.10.3`
- 제품 버전: `0.11.0`
- 승인 범위: 미출시 `0.10.4`를 계승하는 전체 구현, 기존 기능 유지와 Codex 우선 검증
- 추가 결정: 세션·양수 프로세스 ID 없는 자동 재개 거부; 원본 없는 이식 지식은 과거 조회 유지와 현재성 미검증 표시
- [실제 호스트 수용](../research/native-host-qualification-0.11.0.md): 지원하는 파일 검사와 실패·미지원 경계 확인, 두 호스트 비교·자료 복원 완료. 전체 도구·오류 상황의 상시 보호 보장 제외
- 정본: [계획](../plans/PLAN.md), [결정](../decisions/ADR-0023-foundation-refactor.md)

## 생성된 현재 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.11.0`
- 현재 등록 항목: 48/48 완료

- `agent-owned`: 없음
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

## 구현과 근거

- 계획 파서·버전별 ID·완료 근거·출시 검사 결합, 기존 과거 완료와 현재 집계 분리
- `0.11.0` 메타데이터·`v0.10.3`의 정확한 동결 기준본 추가, 이전 기준본 바이트 보존
- 지식 저장·조회 책임 분리와 반환 정본 검사. 원본 부재는 `historical-unverified`, 확인된 변조·내용 변경은 거부; 조회의 자동 복구 없음
- 자동 재개·반복 작업 준비의 공통 사용량·세션 검사. 허가 직전 정책·제어·중단 지문 재확인, 같은 관측 재사용
- 사용자·프로젝트 파일 처리 분리, 공통 준비·비덮어쓰기 적용·복구 연결. 갱신 백업·복구 준비도 같은 함수 사용
- 훅의 세 호스트 형식·프로젝트별 등록·철회·진단·복구 구현. 실행별 개선 후보·반례·명시적 검토와 세션 결합 종료 안내; 자동 정책 변경 없음
- [규칙 목록](../architecture/policy-rule-inventory.md), [호스트 상태표](../research/host-contract-matrix-0.11.0.md), [전후 비교](../research/refactor-baseline-0.11.0.md)
- `7445f519`: 현재 작업의 초기화 감지만 제외·복원. 잔여량 하한 보호와 이미 발생한 초기화 중단 유지, 별도 세션으로 제외 권한 이전 금지
- [2026-09-20 실제 검증](../research/host-acceptance-resume-2026-09-20.md): 새 Codex 작업의 합성 결정 조회·후속 파일 작성과 두 번째 요청의 재조회 확인. 파일 훅의 실제 결과는 아래 별도 기록, [수동 취소 후 같은 작업 재개](../research/codex-native-cancellation-2026-09-20.md) 확인

## 검증 상태

- [Windows Rust 전체](../../tests/results/runs/20260919T192546-3d9fa21840b4.md): 941개 통과·4개 제외, 형식·Clippy 통과. 제외: 선택형 검색 환경 1개·대규모 기밀 자료 1개·SSD 별도 성능 시험 2개
- [Windows Python 전체](../../tests/results/runs/20260919T192646-002ae3023009.md): 930개 중 887개 통과·43개 제외. 문서 117·보안 98·계약 474·통합 82·배포 116개 통과
- 제외: POSIX 전용 권한·FIFO·프로세스, Windows 링크 권한 등 실행 조건. 해당 환경과 실제 호스트 차단의 증명 제외
- 지식 22개 질문의 근거·순서 유지, 반복 저장·조회·비밀값 거부의 불필요한 쓰기 0건. 교대 p95 +8.95%, 질문별 11개는 10% 초과
- 큰 자료의 release 검색 엔진은 기존 50,000개 수용 기준 통과. 실제 앱·최신 CLI 전체 성능과 별도 범위
- 고정 지침 일반 경로 21.68%·지식 경로 20.03% 바이트 감소. 모델 토큰·품질·질문 횟수의 증명 제외
- 실행 8개 비교: 7개 결과 유지, 세션 없는 자동 호출 1개는 승인된 입력 거부. 파일 보존·센서 호출 수 별도 확인
- Source Wiki 204개 문서의 최신 출처·색인 검사: 오류 0개·경고 0개

## 실제 호스트와 권한

- [수용 실행 기록](../research/host-acceptance-final-queue-2026-09-20.md): 두 호스트의 정상·검사기 부재·오류·설정 손상·지침 단독 비교 확인. Codex 미신뢰·자식 범위 확인. 정의·실행 파일·합성 자료 복원 완료
- `verified-workflow`는 소스 계획 경로 적용. 연결된 초기화·검증 영수증이 없어 활성 실행 주장 없음

- Windows Codex에서 기존 정의 신뢰 후 일반 3개 허용·보호 3개 거부 확인. 지침 단독 조건에서는 같은 보호 편집 3개 성공. 지정된 합성 요청만의 비교
- 검사기 부재 시 보호 편집 허용 발견. 형식 2 보완 뒤에도 실제 앱에서 실패했고, 바깥 PowerShell의 변수 해석 문제 재현. 형식 3의 cmd·PowerShell 회귀 14개와 Windows Codex 정상·검사기 부재의 실제 파일 차단 확인
- `tests/work/codex-policy-acceptance-1/`에 승인된 형식 3 적용 완료. 이전 실행 파일·실패 기록 보존, 정의 복원 뒤 새 작업에서도 실제 파일 차단 확인, 다른 호스트·시간 초과의 현재 판정은 위 최종 수용 기록 참조
- Claude 실제 수용과 15초 상시 감시·진행 중 자동 중단은 사용자 승인 후속 범위. Codex·Antigravity의 기본 수동 취소·재개는 별도 근거로 확인
- [세 운영체제 CI](https://github.com/gvm1229/aigent-hive/actions/runs/35465250818) 통과: Linux Rust 976개·4개 제외, macOS 관련 모듈 730개·1개 제외, Windows 관련 모듈 704개·1개 제외
- Linux Python 930개 중 921개 통과·9개 제외. 세 방향의 운영체제 간 지식 이전도 통과. 제외 사유·코드 지문·증명 한계는 [CI 근거](../../tests/results/runs/20260919T195640-1830f54e0861.md)에 보존
- [PR #60](https://github.com/gvm1229/aigent-hive/pull/60) develop 통합 `0090c097`, 정확한 머리 커밋 CI 통과. 0.11.0-test.1 시험판 게시 완료. 안정판 통합·태그·게시·실설치는 `0.11.0` 별도 명시 승인 필요

- 이전 test.1 근거 보존. 협업 지침 포함 38개 기준과 test.2 공개 수용 완료
- [공개 수용과 원인 조사](../research/refactor-release-qualification-0.11.0.md): 첫 실행의 Windows·macOS 실패 뒤 두 차례 세 운영체제 수용 통과. Windows 관측 경합 수정·필수 CI·develop 통합 완료. macOS 최초 원인 미확정 기록 보존

## 이전 근거

- [구현 전 상태](../archive/state/0.11.0-before-plan-generation.md), [ID 대응표](../plans/refactor-id-mapping.md)
- 소스 브랜치 검사와 초기 연구: [기존 근거](../research/host-policy-hooks-2026-09-18.md)

- 2026-09-21 사용자 범위 결정: 15초 상시 감시·진행 중 자동 중단만 [후속 목표](../plans/backlog/periodic-usage-interruption.md) 이전. 현재 작업 제어·명시적 재개는 새 Windows 회귀 61개 통과·Unix/POSIX 조건 3개 제외로 확인

- [최종 완료 검사](../../tests/results/runs/20260920T210828-a7f59de22f8d.md): 선행 34개 근거·제품 지문·공개 수용·PR #61 통합 확인. 현재 사용자 수동 장애 없음; 안정판은 별도 승인 범위

## 협업 지침 추가 범위

- [PDC-001–003](../plans/active/project-directives-collaboration-0.11.0.md): 협업자 설치 선택권을 보장하는 생성기·행동 검증·기존 설치 갱신
- [검증](../research/project-directives-collaboration-0.11.0.md): Windows 실제 AI 8조건·세 운영체제 공개 패키지 수용 완료, 모든 도구 조합 보장 제외
- [예시 수정](../research/example-project-hive-repair-0.11.0.md): PRF 검증 후 6단계 적용·보존·PR 통합 완료. 안정판은 별도 승인
