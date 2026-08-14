# 0.9.4 응답 언어 계약

> 상태: 완료
> 정본: `docs/plans/PLAN.md`
> 범위: 원본 개발 지침과 소비자 하네스의 영어·한국어 응답 규칙

## 완료 기준

- [x] `LGC94-001` 원본 `.agents` 지침의 영어 ASD-STE100·한국어 단일 언어 기준 명시
- [x] `LGC94-002` 소비자 전역·프로젝트 하네스와 사용자 설정 Skill의 동일 기준 반영
- [x] `LGC94-003` 정본과 설치 projection 간 동일 byte 검증
- [x] `LGC94-004` 렌더된 소비자 `AGENTS.md`와 설치 Skill의 계약 회귀 검증

## 범위 제외

- 기존 번역 문서 전면 재작성
- 표준 사전 전체의 저장·배포·자동 언어 검사기 추가
- 새 정식 또는 시험판 게시

## 완료 증거

- 원본 `.agents/directives/01-behavior.md`·`08-human-documentation-style.md`: 영어 ASD-STE100·한국어 의미 중심 문장 규칙
- 소비자 `AGENTS.md` 정본·`0.9.0` project base와 `user-setup` 4개 투영: 같은 계약 반영·byte 일치
- `python -m unittest tests.conformance.test_phase3_static_contracts tests.conformance.test_v09_hive_skills tests.conformance.test_connected_setup_lifecycle -v`: 31개 통과, Antigravity 기본 경로 1개 기존 정책 건너뜀
- `python scripts/check-human-documentation-style.py --all --output json`: finding `0`
- `python scripts/check-markdown-links.py --output json`: failure `0`
- `hive source-wiki index --target . --output json` 뒤 `hive source-wiki lint --target . --output json`: 오류·경고 `0`
