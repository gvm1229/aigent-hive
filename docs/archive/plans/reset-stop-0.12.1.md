# 초기화 감지 뒤 세션 중단 0.13.0

> Plan version: 0.13.0
> Scope: product

- 공개 목표0.13.0. 기존 RST121 ID·0.12.1-dev 실측 원문·이 파일 경로는 검증 연속성을 위해 보존

- [x] [RST121-001] 공유 관측·15초 조회·60초 관측 공백 중단
  - state: complete; evidence: repo:docs/research/session-stop-0.12.1.md#sha256:50b5ba7baa1b9c2ed7bf96669115860655bb2fe0b7b5d7df7ac5fa628c650437
- [x] [RST121-002] Windows 앱 유지·세션별 한 차례 중단·원인 안내·직접 재개
  - state: complete; evidence: repo:docs/research/usage-stop-notice-0.12.1.md#sha256:58148052d858dec443c5d8231c0ad91c7748dfd8e54e2a5b5939e6fc5790cffd
- [x] [RST121-003] 행동 회귀·Windows 수용 근거·설치·상시 관측
  - state: complete; evidence: repo:docs/research/session-stop-installation-0.12.1.md#sha256:be40010560ba6958e4871ccfee9bfd5f8b997f1b53ee2dac9c77d71345c875c2

## 최종 사용자 계약

- 실제 초기화 감지 즉시 진행 중 작업에 한 차례 중단 요청. 정상 조회가 연속 60초 없을 때도 같은 범위의 보호 중단
- 15초 조회 시작·5초 제한·2초 재시도, 예정 시각만으로 초기화 판정 금지. 실패와 초기화의 이유 구분
- Codex 앱 유지. 재개 잠금·해제·사건 확인 명령 제외, Codex의 재개·새 입력 허용. 자동 재개 없음
- 공식 로고의 알림 전용 창: 중단 이유·확인 상태·감지 시각·직접 재개 안내, 닫기 버튼만 제공
- 연속 조회 실패·같은 초기화·제어 연결 재시작·앱 재시작의 반복 중단 억제. 사건·세대 번호는 원인 기록과 중복 억제 목적
- 하한·0% 보호 유지. 다른 기능은 [후속 차수](../backlog/steering-0.12.1.md), 정식 게시 별도 단계

## 검증 범위와 한계

- 기존 실제 네 실행의 native 중단 283~306ms·앱 유지, 정확한 과거 실행의 현재 읽기 4/4·1180ms
- 최종 한 차례 제어는 Windows 모의·Rust 회귀, 공식 로고와 닫기만 남긴 창은 실제 Windows 표시 검증. 사용자 요청에 따른 추가 실제 중단 시험 없음
- [최종 결합 수용](../../../tests/results/session-stop-one-shot-qualification-0.12.1.json)·[현재 설치](../../research/session-stop-installation-0.12.1.md). 0.12.1-dev 설치·정본 보존·로그인 자동 시작·새 정상 관측 확인
- 실제 제공자 할당량 초기화·서버 처리/과금 종료·OS 자식 프로세스의 정확한 5초 종료는 미검증. 다른 OS·원격·클라우드 미지원
- 이전 전체 자동 확인 실패는 그대로 보존. 비활성 과거 파일 14개 삭제의 정책 거부는 별도 수동 정리, 이번 활성 제품 완료와 구분
- [이전 계획](../../archive/plans/reset-stop-0.12.1-before-notice-only.md)·[구현 방식과 과거 실패](../../research/session-stop-0.12.1.md). 이전 앱 종료 수용과 폐기한 재개 잠금은 현재 동작의 증명에서 제외
