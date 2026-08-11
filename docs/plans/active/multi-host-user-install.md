# 복수 호스트 사용자 설치 계획

## 목표

- `hive install`·`hive update`의 기존 단일 `--host` 호환성 유지
- 쉼표 구분 `--hosts codex,claude`와 반복 `--host codex --host claude` 지원
- 복수 호스트 실행의 요청 순서·입력 오류·부분 실패 결과를 구조화된 JSON으로 명시
- 한국어 설치 HTML과 디자인 원칙의 실제 CLI 계약 동기화

## 범위

- CLI: `crates/hive-cli/src/user_install.rs`, `crates/hive-cli/src/main.rs`
- 문서: `docs/hive-install-guide.ko.html`, `docs/guides/public-html-design-principles.md`
- 단일 호스트 출력 code·data 호환성 유지
- Host 설치·로그인·version qualification과 기존 host별 rollback 경계 유지

## Checklist

- [ ] [MHI-001] `--hosts` CSV·반복 `--host` parser와 empty·unknown·duplicate 입력 거부
- [ ] [MHI-002] 복수 host dry-run·apply·validate·update의 순차 실행과 성공·부분 실패 aggregate result 구현
- [ ] [MHI-003] 단일 host 호환·두 입력형·선행 preflight·부분 실패·문서 예시 회귀 검증
- [ ] [MHI-004] 설치 HTML·디자인 원칙·Source Wiki fact·current state 동기화와 `develop` push

## 완료 기준

- `--hosts codex,claude`와 `--host codex --host claude`의 동일 host 순서·결과
- 중복·빈 CSV 항목·지원 밖 host의 mutation 전 exit 2
- 복수 apply 전 전체 host dry-run preflight
- 단일 host ActionResult 계약 변경 0건
- HTML·문서 말투·Markdown link·Source Wiki·Rust test 통과

