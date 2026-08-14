# Aigent Hive 출시 안내

제품 버전별 변경점, 배포 범위, 검증 경계와 알려진 제약의 정본.

## `0.9.4`부터의 GitHub Release 설명 형식

새 Release 설명: English-first 순서의 `## English`와 `## 한국어` section. 두 section: `범위`·`호환성`·
`검증`·`게시`의 같은 fact ID와 같은 제품 사실. English는 ASD-STE100 Simplified Technical English의
짧고 직접적인 문장, 한국어는 자연스러운 한국어 문장. `scripts/check-release-notes.py`: 순서·필수
field·동등성·언어 계약 검사.

## 제품 후보

| 제품 버전 | 상태 | 문서 |
| --- | --- | --- |
| `0.9.4` | Skill 식별자·전역 검증·지식 기록·공개 안내 patch | [`0.9.4`](0.9.4.md) |
| `0.9.2` | usage guard 정본 전환과 공개 문서 동기화 patch | [`0.9.2`](0.9.2.md) |
| `0.9.3` | 명시 프로젝트 지식 조회와 색인 시 자동 일반화 | [`0.9.3`](0.9.3.md) |
| `0.9.2` 베타 | 공개 설치·설정 피드백 수집 | [`베타 안내`](0.9.2-beta-newsletter.md) |
| `0.9.0` | 정식 출시 준비 | [`0.9.0`](0.9.0.md) |
| `0.9.1` | 미등록 project Wiki lint 호환 patch | [`0.9.1`](0.9.1.md) |
| `0.8.0` | npm 시험 배포 후보 | [`0.8.0`](0.8.0.md) |

## 관련 문서

- [제품 출시 결정](../decisions/product-release-decisions.md)
- [`0.8.0` 배포 범위](../decisions/ADR-0013-0.8-release-scope.md)
- [시험 배포 실행 계획](../plans/active/release-0.8.0.md)
- [`0.9.0` 정식 릴리스 범위](../decisions/ADR-0017-0.9-full-release.md)
- [`0.9.0` 정식 릴리스 계획](../plans/active/release-0.9.0.md)
- [Release·update 절차](../guides/release-update.md)
