# 컴퓨터 간 지식 이전

> Checklist owner: `KTX10-*`
> 대상: `0.10.0`의 `.hivekb` 이전·FTS 준비·선택형 벡터 재생성

## Checklist

- [x] [KTX10-001] `knowledge-transfer` 도입과 `knowledge-import → knowledge-scan` 이력 전환 — `421da3e7`; 현재 사실·공개 안내 정합화와 Rust `hive-projection` 38개 통과
- [x] [KTX10-002] 내보내기 미리보기·운영체제별 경로·범위·제외·파일 지문 계약 — `hive knowledge transfer export --preview|--apply`; 파일 생성 전 예상 지문·용량·제외 수 확인, 일반 경로·기존 export 호환 유지
- [x] [KTX10-003] 가져오기 미리보기 지문·예상 SHA-256·충돌 항목 제외·재시도 계약 — `ceaa8990`: 적용 잠금 안의 원문·파일 지문 대조, 유효 충돌·제외 적용·반복 가져오기 회귀
- [x] [KTX10-004] 논리 프로젝트 연결·분리 모음·원문·권한·억제 기록 보존 — 저장소 회귀 13개와 실제 CLI 분리·승인 연결·조회 통과. 물리 운영체제 간 검증은 `KTX10-006`
- [x] [KTX10-005] FTS 준비 완료와 벡터 재생성 예·아니요·취소 분리 — `2e1f2ab8`, `3916bf27`: 이전 기록·지연 선택 보존과 실제 모델 재생성·완성 색인 재사용 통과
- [x] [KTX10-006] Windows↔macOS·Linux 실제 이전, 실패·성능·직접 갱신 검증 — [세 방향 CI 수용](../../../tests/results/knowledge-transfer-cross-os-2026-08-31.md), 기존 버전 갱신 회귀와 100·1,000·5,000파일 실측
- [x] [KTX10-007] 사용자 안내·구독자 업데이트 검토 초안·다음 공개 시험 수용 — [test.7 세 운영체제 공개 설치](../../../tests/results/knowledge-transfer-public-test7-2026-08-31.md) 통과. 초안 미전송, 안정판 제외
- [x] [KTX10-008] 여러 `.hivekb` 일괄 preview·입력·대상 지문·용량 계약 — `merge preview`, 입력 순서·중복 지정 결정성 회귀
- [x] [KTX10-009] 동일 원문 자동 중복 제거와 범위별 출처 보존 — 같은 바이트 한 번 처리, 통합·미선택 Wiki 원본 merge provenance 보존
- [x] [KTX10-010] 의미 후보·수정본·억제 충돌의 검토 요청·재개 계약 — `separate|equivalent|choose` 검토 지문과 SHA-256 수정본 선택, 비선택 원본 보존
- [x] [KTX10-011] 검토 지문을 묶은 단일 원자 적용·FTS·벡터 선택 연결 — 기존 import rollback·FTS 한 번 생성·transfer vector 선택 재사용
- [x] [KTX10-012] `knowledge-transfer` 안내·투영·결과 schema 정합화 — Codex·Claude·Antigravity 투영 38개, review schema·안내 검사 통과
- [x] [KTX10-013] 실패·재실행·100·1,000·5,000파일·교차 운영체제 검증 — [Windows 측정](../../../tests/results/knowledge-transfer-merge-2026-08-31.md), test.8의 Windows↔macOS·Linux 실제 수용 통과
- [x] [KTX10-014] 사용자 안내·구독자 검토 초안·다음 공개 시험 수용 — [test.8 수용](../../../tests/results/knowledge-transfer-merge-public-test8-2026-08-31.md) 통과, 안정판 제외

## 결정

- 전체 이식 가능한 지식이 기본 범위. 기밀·비밀 값·이식 불가 항목은 제외·보고
- FTS 준비가 지식 이전 완료. 벡터 재생성은 새 컴퓨터의 `yes` 설정에서만 별도 질문
- 벡터 재생성의 아니요: 이번 이전 작업만 지연, 전역 설정 보존
- 충돌은 현재 자료 유지·충돌 항목 제외 또는 취소. 덮어쓰기·자동 병합 금지
- 기존 `.hivekb` 형식과 사용자 선택 파일 전달 방식 유지
- 여러 묶음 통합: 확실한 같은 원문만 자동 통합. 의미 후보는 활성 host 검토 자료로 준비하고, 확정 불가 모순만 사용자에게 묶어 질문
- 전체 입력·대상·검토 지문이 같은 경우에만 한 번 적용. 서로 다른 접근 범위의 모음 자동 통합 금지

## 실행 계약

- 승인 Skill: 저장소의 `harness/skills/verified-workflow/SKILL.md`, 소스의 계획·증거·종료 정책 적용
- 실제 작업 실행 graph 활성화 주장 없음. 분리 시험의 성공을 전체 작업 완료로 대체 금지
- 실제 모델 수용: `tests/work/knowledge-transfer-native/receipt.json`; 기존 합성 시험 자료만 사용
- `KTX10-006`: Windows·macOS 생산 묶음과 반대 운영체제 수신을 CI 산출물로 연결. 원격 결과 전 완료 표시 금지
