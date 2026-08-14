# 0.9.4 응답 언어 계약

> 상태: 완료
> 정본: `docs/plans/PLAN.md`
> 범위: 원본 개발 지침과 소비자 하네스의 영어·한국어 응답 규칙

## 완료 기준

- [x] `LGC94-001` 원본 `.agents` 지침의 영어 ASD-STE100·한국어 단일 언어 기준 명시
- [x] `LGC94-002` 소비자 전역·프로젝트 하네스와 사용자 설정 Skill의 동일 기준 반영
- [x] `LGC94-003` 정본과 설치 projection 간 동일 byte 검증
- [x] `LGC94-004` 렌더된 소비자 `AGENTS.md`와 설치 Skill의 계약 회귀 검증
- [x] `LGC94-005` 원본 지침의 한국어 혼용 금지 규칙·대체 예시 보강
- [x] `LGC94-006` 소비자 전역·프로젝트 하네스와 사용자 설정 Skill의 같은 금지 규칙·예시 반영
- [x] `LGC94-007` 정본·네 Skill 투영의 byte 일치와 금지·대체 예시 정적 검증
- [x] `LGC94-008` 렌더된 소비자 `AGENTS.md`의 금지·대체 예시 수명주기 검증

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

## 보강 기준

- 금지 규칙 우선: 한국어 문장 안의 치환 가능한 영어 명사·형용사·동사, 한글·영어 혼합 합성어, 강조용 영어 괄호 표기, 영어 어순 직역 금지
- 필요한 영어 예외 한정: 고유명사·제품·패키지 이름·명령·식별자·경로·스키마 키·정확한 화면 문구·명확한 한국어 대체어 없는 전문 용어
- 예시 필수: 금지 표현과 같은 뜻의 자연스러운 한국어 대체 표현을 짝으로 제시
