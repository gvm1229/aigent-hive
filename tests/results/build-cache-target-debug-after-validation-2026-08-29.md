# `target/debug` 검증 뒤 재생성 캐시 기록

- 조사 시각: 2026-08-29, Windows 로컬 작업 공간
- 경로: `target/debug`
- 조사 결과: 2,508,105,701바이트, 2.336GiB, 파일 3,246개
- 생성 원인: 시험 산출물 수명주기 연결의 정적 계약·문서 lane 검증
- 현재 Cargo·Rust·Hive·Python 시험 프로세스: 0개
- Git 추적 파일: 0개
- 결과 근거: `tests/results/runs/`의 문서 lane 실행 기록과 현재 커밋의 정적 계약 검증
- 재사용 계획: 현재·72시간 안의 구체적 요청 없음
- 삭제 범위: `target/debug` 정확 경로만. `target/release`와 `tests/work` 보존 원본 제외
