# Aigent Hive

Aigent Hive는 Codex, Claude Code, Gemini 계열 host 위에 설치하는 로컬 agent harness를 개발하는 source workspace다.

## 현재 범위

- Rust 기반 CLI와 안전한 setup/update 경계
- provider-neutral `AGENTS.md`와 host별 얇은 projection
- 지속형 역할 정의와 host-owned subagent/session 연결
- Markdown 지식·role·run 정본, tracked YAML/TOML 설정·동의, 재생성 가능한 SQLite 검색 인덱스
- 단순 질문 격리, 사용량 guard, 독립 adversarial verification 계약
- OMX·OMC 선택적 공존

Hive는 모델 API를 호출하지 않는다. 사용자가 이미 로그인한 정액제 Codex·Claude Code·Gemini host가 모델 실행과 인증을 소유한다.

## 저장소 구조

- `crates/`: Rust source
- `harness/`: 소비자 프로젝트에 출하할 template·Skill·projection source
- `.agents/`: Hive 자체 개발 지침
- `docs/`: 현재 계획, 결정, 연구와 운영 가이드
- `schemas/`: provider-neutral machine-readable contract
- `tests/`: synthetic fixture와 conformance test

활성 계획은 [`docs/plans/PLAN.md`](docs/plans/PLAN.md), 현재 작업 상태는 [`docs/state/CURRENT.md`](docs/state/CURRENT.md)에서 확인한다.

## 개발 브랜치

- `main`: 안정 기준
- `develop`: 일반 개발과 통합

초기 bootstrap 이후 변경은 `develop`에서 진행하고, `main` 반영은 Pull Request로 수행한다.

## 상태

Phase 0 source scaffold 단계. Rust toolchain이 없는 환경에서도 source 구조와 문서를 검토할 수 있지만, 코드 변경 완료 판정에는 CI 또는 로컬 Cargo 검증이 필요하다.

## 라이선스

공개 배포 라이선스는 아직 확정되지 않았다. 라이선스 결정 전에는 저장소 공개가 사용·수정·재배포 허가를 의미하지 않는다.
