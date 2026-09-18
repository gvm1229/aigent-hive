# 현재 상태

- 작업 branch: `refactor/hive-foundations`; 통합 branch: `develop`; 안정판 branch: `main`
- 개발 목표: `0.11.0-test.1`; 공개 안정판·현재 제품 파일: `0.10.3`
- 사용자 승인: 기존 `0.10.4` 변경을 계승하는 전체 리팩터링 구현, 기존 기능·명령 유지, Codex 우선 검증
- 정본: [활성 계획](../plans/PLAN.md), [결정](../decisions/ADR-0023-foundation-refactor.md)

## 생성된 현재 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.11.0`
- 현재 등록 항목: 12/35 완료

- `agent-owned`: `HK-001`, `HK-002`, `HK-003`, `HK-004`, `HK-005`, `RFB-001`, `RFB-002`, `RFR-001`, `RFK-001`, `RFK-002`, `RFK-003`, `RFH-001`, `RFH-002`, `RFH-003`, `RFP-001`, `RFP-002`, `RFP-003`, `RFP-004`, `RFT-001`, `RFT-002`, `RFT-003`, `RFT-004`
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: `RB104-004`
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

## 현재 증거와 다음 작업

- 기준 코드 `39e53695`의 Windows debug 빌드 성공, 기존 vendor 경고는 원본 보존
- 바이너리와 7개 CLI 경로의 반복 결과 보존, 준비 5회·측정 30회. 시작 지연 p95 약 8.4–9.2ms; 검색·호스트 실행 성능의 증명과 구분
- 계획 ID 21개를 [출시 형식으로 정합화](../plans/refactor-id-mapping.md), 항목 의미·완료 상태 유지
- 공통 계획 파서·생성기·출시 후보 결합·실제 계획의 시험 36개 성공, 상태 이관과 문서 검사 연결 완료
- 순서 조정: 기준 보존 → `RFS-001–002` 계획 계약 → `RFB-001` 버전 메타데이터. 제품 구현 전에 기존 거부 기준 복구
- `RB104-001–003`은 새 Windows Rust 전체 회귀 913개 성공으로 근거 교체. 별도 수용 4개는 제외 사유 유지. `RB104-004`의 실제 호스트 지원 확인 전 완료 처리 금지
- 소스 검증 실행 그래프 연결이 없어 검증형 스킬 활성 주장 제외, 소스 계획과 결정적 시험 기준 진행

## 남은 검증과 권한

- 안정판 통합·태그·게시·실제 사용자 설치는 해당 버전의 별도 명시 승인 필요
- 실제 호스트 신뢰·로드·취소·재개·효과 확인은 모사 자료로 대체 금지
- 기존 Source Wiki 원본 지문 경고·사용량 보호 안내 문체·Skill 투영 불일치는 `RFB-002` 정리 범위
- 다른 운영체제 및 새 제품의 공개 수용은 미실행, 계획의 해당 항목에서 확인
- 이미 완료된 소스 브랜치 검사와 연구는 [기존 근거](../research/host-policy-hooks-2026-09-18.md) 보존

## 이전 상태

- [구현·상태 생성 전 전체 기록](../archive/state/0.11.0-before-plan-generation.md): 이전 출시·권한·설계·검증 한계 보존
- [이전 0.10.2 상태](../archive/state/0.10.2-before-instruction-closeout.md)
