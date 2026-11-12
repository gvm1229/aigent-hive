# 단계 2 시작과 이전 브랜치 정리

- 확인일: 2026-11-12, 실행: Codex·Windows 11
- 범위: 문서·측정·제공자 약관 확인·허용된 대화형 실험. 제품 바이트·정식 태그·설치 변경 제외
- 2026-11-12 유지관리자의 단계 2 실험 명시 승인과 재정비 한정 상시 승인 적용. OVX-009 구조 결정·OVX-010 병합 전 중단
- 진입 근거: develop `80acfeb8`, 0.14.2 정식 태그 존재, 이전 단계 끝 커밋 `992ca582`의 develop 조상 관계 종료 0. 정식·실제 PC 검증은 [OV142-008](OV142-008.md)
- 실행: `git switch develop`, `git pull --ff-only` 종료 0, Already up to date
- 실행: `python scripts/branch-policy.py check test/overhaul-2-baseline`, `python scripts/branch-policy.py create test/overhaul-2-baseline` 종료 0
- 삭제: `git branch -d fix/overhaul-1-0.14.2`, `git push origin --delete fix/overhaul-1-0.14.2` 종료 0. 삭제 대상 끝 커밋 `992ca582`, develop 이력에서 복구 가능
- 보존: 사용자 비추적 검토 문서 두 개. 제품·전역 호스트 설정·인증 파일 수정 및 읽기 없음
- 사전 확인: 설치된 보호의 현재 Codex PID·세션 결합에서 `hive.usage-allowed`. 일일 갱신 확인은 24시간 제한으로 생략, 설치 없음
- 지식 조회: 오래된 파생 색인 오류 뒤 `hive source-wiki index`로 정본 268쪽 재색인, 제한 검색 종료 0·적중 0. 정본 지식 내용 변경 없음
- 검증: documentation, 문체 `--all`, 계획 상태, Source Wiki lint, branch-policy. 현재 Windows의 문서·이력 범위만 증명, Linux·macOS 실행 및 제품 재수용 증명 제외
