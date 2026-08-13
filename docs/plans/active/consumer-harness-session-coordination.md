# `0.9.3` 소비자 하네스 세션 조정·최소 변경 directive 갱신

> Checklist owner: `CHS93-*`
> 대상: `0.9.3`
> 상태: 구현 대기
> 근거: 소비자 하네스와 전역 설치에 동시 세션 충돌 검사가 없다는 2026-08-13 검토

## 목표

- 소비자 프로젝트의 추적 파일 편집 전 세션 범위·충돌·종료 상태 확인
- directive 준수 계약과 Hive CLI의 원자적 충돌 판정을 분리
- 기존 Hive 프로젝트의 오래된 directive를 사용자 작성 문구 보존 상태에서 최소 변경 갱신
- 세션 조정 미지원 host도 동일한 안전 계약과 명시적 unsupported 결과 유지

## 제품 경계

- 소비자 세션 상태: Git 제외 `.hive/runtime/active-sessions/` 아래 Hive-owned Markdown·TOML
- Hive CLI: begin·check·update·close의 exact target·경로 범위·원자적 점유 판단만 담당
- host hook: 지원·명시 동의·정확한 event가 모두 확인될 때만 편집 전 검사 연결. 그 밖의 host는 CLI와 directive 계약 유지
- Hive 외 editor·사용자·외부 도구의 직접 편집 절대 차단 주장 없음
- raw host session·대화·credential 저장 없음. 공개 식별자는 Hive 생성 local session ID 또는 digest만 허용
- update 대상: Hive marker block과 Hive-owned directive rule. 사용자 작성 본문·foreign directive·비충돌 Hive rule 보존

## Checklist

- [ ] [CHS93-001] `.hive/directives/03-session-coordination.md`와 AGENTS Hive marker의 소비자 공통 계약·Git 제외 runtime 경로·상태 전이 정의
- [ ] [CHS93-002] `hive session begin|check|update|close` exact target control plane, path canonicalization·parent/child overlap·atomic contention·stale session recovery 구현
- [ ] [CHS93-003] Codex·Claude·Antigravity projection과 host capability/consent 조건의 pre-edit 검사 연결. 미지원·비동의 host의 truthful fallback 검증
- [ ] [CHS93-004] `project-setup` preview·apply의 outdated Hive directive surgical upgrade 구현. 새 규칙과 직접 모순되는 Hive-owned clause만 갱신하고 사용자 작성·foreign·비충돌 bytes 보존
- [ ] [CHS93-005] 신규·기존 프로젝트 fixture의 session collision·close/recover·user-authored directive 보존·rollback·세 host projection·full static/installer qualification

## 수락 기준

- 동일·상위·하위 path를 점유한 활성 Hive 세션의 concurrent begin: 하나만 성공, 나머지는 mutation `0건`의 충돌 결과
- exact target session check·close: selected pointer·다른 project runtime과 무관
- 새 project setup: 세션 directive·runtime ignore·AGENTS entrypoint 일관 생성
- 기존 project update: 변경 전 preview에 rule-level 대상·근거·보존 bytes 표시, apply 뒤 직접 모순 rule 외 diff `0건`
- 사용자 작성 directive와 foreign host configuration overwrite `0건`
- provider API·credential·직접 model/subagent process spawn·신규 OMX/OMC 경로 `0건`

## 출시 연결

- `REL93-001`과 `REL93-005–010`의 current-tree·설치·세 host·문서 qualification에 포함
- `CHS93-001–005` 완료 전 `0.9.3-test.N` 및 stable publication 금지
