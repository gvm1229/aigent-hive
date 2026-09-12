# 현재 상태

- 작업 branch: `develop`; stable source branch: `main`
- 제품 버전: `0.10.3`
- 현재 공개 stable: `0.10.3`
- 완료 작업: 소스·harness 지침 품질 개선 `INS102-*` 6/6, 전역 사용자 갱신 `GUU102-*` 10/10
- 출시 상태: Stable `0.10.3` 공개·독립 검증 완료
- 활성 계획: [PLAN.md](../plans/PLAN.md)
- 개선 계획: [instruction-quality-0.10.2.md](../plans/active/instruction-quality-0.10.2.md)
- 전역 갱신 계획: [global-user-update-0.10.2.md](../plans/active/global-user-update-0.10.2.md)
- 출시 계획: [release-0.10.2.md](../plans/active/release-0.10.2.md)
- 결정: [ADR-0022](../decisions/ADR-0022-global-user-update.md)

## 권한과 범위

- 2026-09-12: [사용량 복구 실행 계획](../plans/active/usage-recovery-0.10.2.md)의 `0.10.3` 안정판 공개 완료; 표식 재측정·같은 위치 교체의 공개 시험·실제 설치 수용
- 사용자 보정: 별도 `v2` 저장소 제안 폐기, 같은 위치의 옛 파일 교체·제거. 모든 Hive 프로젝트·소스 공통 적용; `AI_Learning` 전용 해결 금지
- `UGR103-*`: 10/10. 실제 사용자 설치 `0.10.3-test.1`, `AI_Learning`의 형식 1·이전 프로세스 표식 재측정 제거, 소스 작업 허용, 다른 등록 프로젝트 기준 보존 확인
- 안정판 구독자 안내: `docs/releases/0.10.3.subscriber.ko.md`, 승인 digest `sha256:bbb0cc20655adb4e08fcbb9bcff6e1c837bec2a3905a69a2b8991b41d60381a6` 등록·게시 완료

- 이번 안정판 승인 범위의 게시·실제 사용자 루트 설치 갱신 완료
- 과거 project/user base·외부 파일·사용자 선택 보존
- 전역 사용량 보호: 유지보수자가 지정한 남은 사용량 `2%`

## `0.10.3` 안정판 증거

- `main` 통합 commit: `8afe730759325bc003475f81545da7afc88cfb18`; 안정판 후보 `34640191785` 성공
- 최초 게시 `34678212448`은 macOS 보조 npm 패키지의 태그 전파 지연으로 중단. 같은 후보의 복구 게시 `34678404622` 성공
- npm `aigent-hive@0.10.3`와 `latest=0.10.3`, GitHub 정식 Release·annotated tag `v0.10.3`을 독립 확인
- 이 Windows Codex 설치: `0.10.3-test.1`에서 `0.10.3` 갱신과 `hive --version` 확인; 이전 `AI_Learning` 경로 부재로 안정판 재실행 미수행

## 개선 작업

- reset-booster `RB104-001–003` 구현: 같은 측정 범위의 잔량 증가를 `hive.usage-reset`으로 차단하고,
  기준 관측을 같은 런타임 표식에 저장. 15초 감시·진행 중 추론 중단·현재 작업 opt-out은 검증된 host API가
  없어 `RB104-004`에 남음.

- 모호한 일반 요청의 작업 권한 유지와 명시적 프롬프트 작성 경로 분리
- 현재 Skill 참조·기본 stable 채널·공유 지식 색인·종료 hook 규칙 정합화
- 사용자 설정 진입 문서와 작업별 참조 분리, Rust·정적 호스트 투영에 동반 자료 포함
- 인증 실패의 읽기 전용 진단 우선, 전체 제거·재설치의 별도 권한 유지
- 독립 문서 기반 행동 평가와 관련 시험 완료. 실제 장시간 호스트 성공률 증명은 제외
- 완료 수치는 개선 계획의 검증된 체크리스트 기준

## 전역 갱신과 출시

- `GUU102-*` 10건과 로컬 출시 검증 `REL102-001–002` 완료
- `REL102-003–006`은 `0.10.2-test.3` 후보·게시·세 운영체제 공개 수용과 source·제품 digest 결합 완료
- `REL102-007` stable 문서·구독자 안내 digest 승인 완료
- `REL102-008–009` 완료: PR `#54`의 `main` 통합, 후보 `34190329196`, 최종 게시 `34200601474`
- npm 여섯 package의 `0.10.2`·`latest`·무결성 값과 GitHub 정식 Release·annotated tag·공개 자산 독립 확인 완료
- 기존 출시 종료 시 남은 항목: 0건. 현재 계획 작성 범위의 남은 작업: 0건; 제품 복구는 후속 `UGR102-*` 10건으로 별도 관리
- 이전 출시 증거와 조사 기록: [이전 상태](../archive/state/0.10.2-before-instruction-closeout.md)
