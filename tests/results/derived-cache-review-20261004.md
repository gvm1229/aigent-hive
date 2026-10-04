# 과거 합성 시험의 파생 캐시 검토

- 현재 요청: 시험 파일 정리·반복 누적 방지
- 실제 검토 환경: Codex, Windows 11, Python 3.12; 다른 운영체제 실행 없음
- 대상: `tests/work/scope-audit-20260828` 안의 합성 사용자 `.hive/index` 12개
- 용도: 과거 벡터 검색 시험의 SQLite 검색 색인·다운로드 모델 복제본
- 정본: 같은 합성 폴더의 Markdown·시험 입력, 소스 저장소의 연구·결과 기록
- 삭제 제외: `.hive/knowledge`·연구 스크립트·요약 JSON·수용 기록·Git 추적 파일
- 함께 정리할 재생성 가능 경로: `tests/work/vector-research/qdrant-edge-bench/target`, `tests/work/vector-research/model-cache`, `tests/work/vector-research/venv`
- 별도 Rust 실험의 Cargo.toml·Cargo.lock·src 보존, 모델·환경은 기존 입력과 도구로 재생성
- 이전 조사 완료·현재 재사용 없음, 현재 프로세스·연결 경로·목록 지문 검사를 삭제 직전 재확인
- 정확 경로·파일 수·바이트·정리 결과는 `cleanup/`의 실행 기록 참조
- 본 기록은 캐시 정리 판단 근거, 과거 시험의 새 통과 판정이나 공개 수용 근거에서 제외
- 원시 캐시의 직접 복구 보장 없음, 정본과 검사 결과의 보존 확인
