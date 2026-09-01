# `target/debug` 빌드 캐시 정리 전 기록

- 조사 시각: 2026-08-29, Windows 로컬 작업 공간
- 경로: `target/debug`
- 조사 결과: 88,438,528,892바이트, 82.365GiB, 파일 112,594개
- 내용: Rust 개발 실행 파일·라이브러리·디버그 정보·증분 컴파일·의존성 빌드 산출물
- 현재 Cargo·Rust·Hive·Python 시험 프로세스: 0개
- Git 추적 파일: 0개
- 공개 `0.10.0-test.6` 수용: 원격 `d331dc87` 제품·공개 패키지 근거로 완료, 이 로컬 캐시의 보존 근거에서 제외
- 재생성: 동일 Rust 도구·`Cargo.lock`·소스로 필요한 대상만 다시 빌드 가능
- 복구 한계: 삭제한 로컬 디버그 파일의 직접 복구 보장 없음, 시험 결과는 [tests/results](README.md)와 공개 수용 근거에서 별도 보존

## 삭제 승인 범위

- `target/debug` 정확 경로만 대상
- `target/release`, `tests/work/scope-audit-20260828`, `vector*`, `vector-test5*`, `vector-test6*` 제외
- 삭제 직전 프로세스·Git 추적·경로·지문 재확인 필수
