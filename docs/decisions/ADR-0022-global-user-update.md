# ADR-0022: `0.10.2` 전역 사용자 설치 수렴

- 상태: accepted
- 날짜: 2026-09-07
- 기준 stable: `0.10.1`
- 첫 공개 시험: `0.10.2-test.1`

## 문제

실행 파일 갱신은 인증된 사용자 투영을 새 release로 바꾸지 못할 수 있으며, 최신 version에서는
즉시 종료한다. 구형 설정 또는 새 질문은 전역 setup이 필요하지만 사용자가 별도 `install`·`setup`
명령을 알아야 한다.

## 결정

- `0.10.2` 출시 범위에 소스·harness 지침 감사 후 승인된 품질 개선 포함
- 조사 가능한 문제는 허용 범위에서 해결, 새 권한·실질적 사용자 선택만 질문
- 검사 범위는 파일 확장자보다 실행·승인·배포 행동 변화 기준

- bare `hive update`는 실행 파일 version과 무관하게 Hive 소유 전역 사용자 상태를 점검·수렴한다.
- npm은 실행 파일만 설치한다. 최초 `hive update`가 대화형으로 호스트를 선택하고 최소 투영을 준비한다.
- versioned question catalog의 미응답 질문은 `setup-required` 상태와 일반 작업 차단을 만든다.
- 마지막 답 뒤에는 저장된 transaction을 digest-bound 방식으로 재개해 전역 설정·호스트 투영을 적용·검증한다.
- 질문 없는 update는 인증된 기존 답과 선언된 Skill migration만 사용해 자동 완료한다.
- 프로젝트 registry·project harness는 읽거나 변경하지 않는다.

## 결과

전역 설치의 정상 경로는 `npm install -g aigent-hive` 다음 `hive update` 하나가 되며, 새 설정 선택은
다음 호스트 세션에서 명시적으로 답할 때까지 안전하게 대기한다.
