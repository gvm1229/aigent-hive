# 재정비 한정 명확한 권고안의 상시 승인

## 계획과 결정

- 범위: 재정비 계획 문서만, 재정비 전체 완료까지. AGENTS·01 등 일반 지침과 사실 문서의 승인 안전장치 변경 없음(유지관리자 정정, 2026-11-12). 제품 버전 0.14.2 유지
- 근거: 2026-11-12 유지관리자 지시. 명확한 권고안은 추가 질문 없이 실행, 선택과 이유는 소유 계획·근거에 기록
- 순서: 재정비 현재 위치·브랜치 규칙 → OVX 상태·약관 선행 조건
- 선택 이유: 같은 권고안을 다시 승인하는 절차 제거. 비슷하게 좋은 구조 선택지와 상시 경계 변경만 질문
- 보존: 05 안전·자격 증명 파일·제공자 API·AI trailer·Codex Ultrafast·`claude -p` 금지, 정식 이후 새 제품 바이트의 새 버전·test.1
- 제외: 일반 지침 변경, 실험 실행, 제품 변경
- 승인: 검사·수용 통과와 계획 순서 준수 시 버전별 정식 태그·보호 main 통합·게시·설치의 상시 승인. 단계 브랜치 삭제·다음 단계 시작도 추가 확인 제외
- OVX-003 읽기 전용 유지. OVX-005는 OVX-008의 제공자별 현재 약관 허용 확인 뒤에만 실행, 불허·불명확 제공자 부분은 blocked
- 별도 질문: OVX-009의 02 프로세스 실행 금지 변경
- 확인: documentation 검사, 문체 전체 검사, 지침 예산, Source Wiki lint, 계획 상태, 브랜치 정책
- 완료 조건: 관련 검사 통과, 요청 경로만 커밋, 최신 origin/develop 기반 ff push, 작업 브랜치 삭제·세션 기록 제거·detached 유지
- 복구: 편집 원인 실패 수정 후 해당 검사 재실행, 외부 실패는 해당 작업만 중단

## 검증

- 실행 환경: Codex 초안 뒤 orchestrator가 일반 지침 변경을 되돌리고 재정비 범위로 축소·검사
- 실험·출시·설치 실행 없음. 문서 정합성 검사는 실제 제품 동작·제공자 약관 허용의 증명 범위 밖

## 2026-11-12 위임 확대

사용자 원문:

> For the duration of our overhaul plan, I grant you explicit authorization to autonomously proceed with everything including the stable version release.

> I don't want to wake up in the morning and see that there's a question from you that's been standing for multiple hours.

- 범위: 재정비 완료까지. 정식 배포 포함 모든 `awaiting-user-authority` 항목 사전 승인
- 구조 선택: 권고안으로 결정, 선택·이유를 근거 파일에 기록 뒤 진행
- 유지 경계: 금지 명령, 자격 증명, 제공자 API, 전역 호스트 설정 변경 금지

## OV150-002 적용

- 2026-11-13 현재 요청: 위 2026-11-12 원문을 0.15.0 시작 승인으로 적용. 실제 왕복 미완료 상태에서 OV150-002–010 구현 선행 선택
- 이유: 봉투 작성·정적 검증은 호스트 모델 실행과 독립. 기본 비활성 유지로 실행 검증의 경계 보존, OV150-011의 실제 왕복 수용은 계속 미완료
- 이번 종료 경계: OV150-011 전, 태그·게시·develop 병합 없음
