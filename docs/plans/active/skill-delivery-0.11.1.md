# Codex 스킬 제공 위치와 자동 정리

> Plan version: 0.11.1
> Scope: product
> 최초 공개 시험 대상: `0.11.1-test.1`

## 목적과 승인 경계

- Codex 플러그인이 제공하는 기본 스킬의 사용자·프로젝트 복사본 중복 제거
- `hive update`와 프로젝트 갱신에 정리 포함; 별도 정리 명령·확인 질문 제외
- 동일한 인증 원본만 제거; 사용자 수정본·외부 파일·프로젝트 전용 선택 보존과 사유 표시
- 구현·시험판 준비·검증 승인; 안정판·보호 브랜치 통합·현재 사용자 설치는 별도 버전별 승인
- Codex 우선; 다른 호스트의 전용 스킬 제공 유지

## 수용 기준과 실행 순서

- [x] [SDP-001] 0.11.1 계획·버전·0.11.0 기준본 등록
  - state: complete; evidence: repo:tests/results/runs/20260929T195119-6c32a5791c1c.md#sha256:6288270a2ea9e6823ad1f4112b1f448326972c71f6b45b7d938e3ffa58405d54
- [x] [SDP-002] 사용자 설치·갱신의 플러그인 제공과 자동 정리
  - state: complete; depends: SDP-001; evidence: repo:tests/results/runs/20260929T202401-55370a195d9d.md#sha256:c51cedc5d15733bc85a23036179d9870c77ce476c9c8ff57cb41cd4d9ff9e39b
- [x] [SDP-003] 프로젝트 설정·갱신의 제공 출처와 자동 정리
  - state: complete; depends: SDP-002; evidence: repo:tests/results/runs/20260929T204752-0fe5ef9afbee.md#sha256:32ffc55bdcbf78116322a8fd4952b081416a5796ec935e3a0f9ab13c07bb40ca
- [x] [SDP-004] 설치·갱신·실패 복구와 전체 회귀 검증
  - state: complete; depends: SDP-002,SDP-003; evidence: repo:docs/research/skill-delivery-0.11.1.md#sha256:ddd6a0c570ad81f6929700ee0c282ecb2a2df68b8bb5e1ccb5b6410c47775bc3
- [ ] [SDP-005] 시험판 준비와 실제 Codex 발견 검증
  - state: awaiting-user-authority; depends: SDP-004; owner: 유지관리자; reason: 0.11.1-test.1 공개 세 OS 수용 완료, 현재 사용자 설치·Codex 재시작·실제 발견과 호출의 별도 승인 필요

## 구현 인계

1. SDP-001: `develop` 유지. 0.11.0 원본과 기존 계획 기록 보존, 새 버전 0.11.1·시험 대상 test.1 등록. 제품 번호·기준본 검사 통과 필요.
2. SDP-002: `user_setup::user_projection_files`, `plan_user_projection`, 적용·검증과 `user_install` 호스트 준비 재사용. Codex 선택 시 공통 기본 스킬 생성 제외. 플러그인 등록·내용 검증 후 정리 확정; 실패 시 이전 상태 복구. 인증된 동일 원본만 제거, 수정본은 보존·보고. 사용자 설정·활성 목록·지침 유지.
3. SDP-003: CLI가 검증한 플러그인 제공 정보를 `hive-render`에 전달; 순수 렌더러의 사용자·캐시 접근 금지. 같은 지침과 부속 파일을 제공하는 기본 스킬만 생략. 프로젝트 전용·외부·내용 차이 스킬은 로컬 제공. 활성 목록에 출처 기록. `project upgrade --user-root`는 선택형이며 미지정 시 기존 로컬 방식. project-refresh의 확인된 사용자 루트 전달. 원본 동일 파일 자동 정리, 수정본은 기존 보존 기록 유지. 플러그인 부재·미검증 시 로컬 제공, 의존성 상실은 검증 실패와 갱신 복구 안내. 전역 갱신의 프로젝트 일괄 변경 제외.
4. SDP-004: 관련 Rust·Python 시험 → 전체 회귀. 신규·0.11.0 이전·재실행·다중 호스트·프로젝트 전용·사용자 수정·부속 파일 수정·외부 파일 보존 확인. 플러그인 부재·비활성·구버전·손상·실패·변경 경합과 복구 확인. 결과마다 실제 실행 host/OS·증명 범위와 제외 이유 기록.
5. SDP-005: 0.11.1-test.1 준비와 세 OS 설치·갱신 수용. Windows 실제 Codex 시험 환경에서 발견 목록·명시 호출·자연어 호출 확인. 현재 사용자 설치 변경은 별도 승인. 파일 검사와 실제 앱 증명 구분, 실제 재시작·수동 조작 필요 시 독립 작업 완료 후 정확한 인계.

## 정리 계약

- 디렉터리 이름 또는 SKILL.md 하나만으로 삭제 금지; 설치 명세·원본·전체 부속 파일 확인
- 삭제 전 기존 백업과 파일 변경 경합 검사; 실패 시 복구
- 보존으로 남은 중복의 경로·이유 보고, 숨김·사용자 파일 덮어쓰기 제외
- 설정·재설정·갱신·검증의 같은 제공 규칙, 두 번째 갱신의 불필요한 변경 0건

## 실행 인계

- [공개 수용 근거](../../research/skill-delivery-public-test-0.11.1.md): 정확한 소스·후보·게시·세 OS 실행 파일 지문과 통과 범위
- Agent 소유 작업 0건; 미완료 1건은 현재 사용자 설치의 승인과 실제 Codex 새 세션 수용
- 승인 후 정확한 시험판 적용·사용자 자동 정리·대상 프로젝트 갱신·Codex 재시작·스킬 목록과 두 호출 방식 확인
