# 시험 결과

- 실행별 기록: [runs/](runs/)
- 목적·소스·환경별 탐색: [INDEX.md](INDEX.md)
- 과거 시험의 선별 결과: [legacy/](legacy/)
- 별도 빌드의 조사 목록: [legacy-build-caches.md](legacy-build-caches.md)
- 정리 구현 계획: [implementation-plan.md](implementation-plan.md)
- 정보 조회 순서: 시험 목적 → 소스·변경 지문 → 운영체제·도구 → 결과·한계
- 검색 예시: `rg -n 'purpose|source_commit|platform|status' tests/results/runs`
- 과거 통과와 현재 코드 검증 구분, 관련 변경·신선한 증거 요구 시 재실행
- 작은 검토 완료 결과만 보존, 모델·데이터베이스·환경 복제본 제외
- 비밀·개인 경로·원문 세션 기록 제외
- 실행 중 보고서는 진행 정보이며 통과 근거에서 제외
- 삭제 선행 조건: 결과 Markdown 검증·Git 기록, 명시적 종료·미사용 확인
