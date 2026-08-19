# `0.9.4` 프롬프트 영어 기본값

> Checklist owner: `PML94-*`
> 대상: `0.9.4` patch
> 표면: 원본 개발 지침, `prompt-refine` Skill, 전역·프로젝트 하네스 투영

## 문제

선택 응답 언어가 한국어인 경우 생성 프롬프트도 한국어로 작성 가능. 프롬프트의 기본 언어와
설명·질문의 응답 언어 분리 필요.

## 원칙

- 응답 언어: 사용자 선택 언어 유지
- Hive 작성·개선·복사용 프롬프트: 현재 프롬프트 언어의 명시 요청이 없으면 영어
- 현재 프롬프트 언어의 명시 요청: 기본값보다 우선
- 영어 프롬프트: ASD-STE100 Simplified Technical English
- 이 규칙: 설명·질문 언어 변경 근거로 사용 금지

## Checklist

- [x] [PML94-001] 원본 개발 지침과 `prompt-refine` 정본의 영어 기본값·명시 언어 우선 규칙 반영
- [x] [PML94-002] 전역 사용자 지침 네 투영과 프로젝트 `AGENTS.md` 생성 경로의 같은 규칙 반영
- [x] [PML94-003] Rust 렌더러·사용자 지침·소비자 수명주기·정적 계약 회귀 검증
- [x] [PML94-004] `0.9.4-test` 설치본의 한국어 응답 환경 default English prompt·명시 한국어 prompt 수용

## 수락 기준

- 한국어 응답 환경의 설명·질문: 한국어 유지
- 언어 지정 없는 정제 프롬프트: 영어
- `한국어 프롬프트` 같은 현재 프롬프트 언어 명시: 한국어
- 전역·프로젝트 하네스의 렌더 결과: 정본 규칙 포함

## 범위 제외

- 기존 대화 기록·사용자 작성 프롬프트 번역
- 인터페이스 언어 기본값 변경
- provider API 또는 자동 실행 경로 추가

## 완료 증거

- 원본 `.agents/directives/01-behavior.md`·`prompt-refine` 정본: 응답 언어 분리, 명시 없는 프롬프트 영어 기본값, 현재 프롬프트 언어 명시 우선
- `user_setup.rs`의 영어·한국어 사용자 지침과 `hive-render` 프로젝트 marker: 같은 규칙 렌더링
- `python scripts/sync-user-plugin.py`: plugin·Codex·Claude Skill 투영과 active Skill digest 동기화
- `cargo test -p hive-cli user_directive_uses_the_selected_interface_language`: 1개 통과
- `cargo test -p hive-render`: 61개 통과
- `python -m unittest tests.conformance.test_v09_hive_skills tests.conformance.test_phase3_static_contracts tests.conformance.test_connected_setup_lifecycle -v`: 32개 통과·기존 Antigravity 정책 건너뜀 1개
- public `0.9.4-test.1` Windows x64 user setup: Korean response directive와 English-default
  prompt directive 설치 확인. Installed CLI `prompt validate`: default English·explicit Korean
  copy-ready prompt contract 각각 `hive.prompt-refinement-valid`
