# ADR-0024: Codex 스킬 제공 위치와 자동 정리

- 상태: 승인된 0.11.1 구현 범위
- 근거: 같은 기본 스킬의 project·local file·aigent-hive 중복 표시
- 참고: [oh-my-codex 설치 방식](https://github.com/Yeachan-Heo/oh-my-codex/blob/main/plugins/oh-my-codex/skills/omx-setup/SKILL.md), [과거 중복 문제](https://github.com/Yeachan-Heo/oh-my-codex/issues/1069)

## 결정

- Codex의 기본 스킬 제공: 검증된 활성 Hive 플러그인 우선
- 전역 갱신의 사용자 복사본 자동 정리, 프로젝트 갱신의 프로젝트 복사본 자동 정리
- 별도 정리 명령·추가 승인 질문 제외, 전역 갱신의 프로젝트 일괄 변경 제외
- 설치 명세와 전체 원본 일치 파일만 제거; 수정본과 외부 파일 보존
- 프로젝트 선택 목록 유지, 실제 제공 출처와 로컬 보존 이유는 `skill-providers.json`에 기록
- 플러그인 부재·미검증 시 프로젝트 로컬 제공; 기존 플러그인 의존 경로는 프로젝트 갱신으로 복원
- 설명 언어와 선택 화면 메타데이터를 제외한 지침 본문·부속 파일 동일성 검사
- 디렉터리 이름·캐시 존재만으로 소유권 또는 활성 제공 판정 금지
- 사용자 정리도 기존 설치 transaction의 백업·경합 검사·실패 복구에 포함
- 다른 호스트의 전용 제공 방식 유지

## 수용 범위

- [구현 계획](../plans/active/skill-delivery-0.11.1.md)
- 파일·명령 시험과 실제 Codex 발견 시험의 별도 근거
- 현재 사용자 설치·안정판 게시·보호 브랜치 통합은 버전별 별도 승인
