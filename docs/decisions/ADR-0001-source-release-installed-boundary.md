# ADR-0001: source, release, installed harness 분리

- 상태: accepted
- 날짜: 2026-07-23

## 결정

Hive source, immutable release bundle, consumer installed harness를 별도 artifact class로 유지.

## 이유

- source 개발 지침이 사용자 프로젝트로 누출되는 문제 방지
- plugin cache 삭제와 사용자 data lifecycle 분리
- setup/update write ownership 검증 가능
- 동일 source에서 host별 projection 재현 가능

## 결과

- source root의 `hive-source.json` 발견 시 consumer setup 거부
- root `.agents/` 출하 금지
- consumer artifact test는 `tests/work/`에서만 수행
- release는 source 경로를 runtime에 참조하지 않음
