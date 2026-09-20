# 브랜치 규칙 강제와 호스트 훅 조사

> Plan version: 0.11.0
> Scope: source

> 소유 항목: `BR-001–005`
> 범위: 소스 개발 도구 구현, 저장소 Git 훅·GitHub 규칙 적용, 제품 훅 조사
> 제품 코드·설치된 소비자 지침의 변경 제외

- [x] [BR-001] 브랜치 이름 교정과 현재 문서·기본 규칙의 작업 종류 접두사 정합화
  - state: complete; evidence: repo:docs/research/host-policy-hooks-2026-09-18.md#sha256:f0a2d0633f974eeda6fe84e9eb8f9c062b9fd3ba4311c17d6b4239ada1350def
- [x] [BR-002] 단일 이름 검사·생성 도구·Git 참조 및 게시 훅 구현과 실제 Git 회귀
  - state: complete; evidence: repo:docs/research/host-policy-hooks-2026-09-18.md#sha256:f0a2d0633f974eeda6fe84e9eb8f9c062b9fd3ba4311c17d6b4239ada1350def
- [x] [BR-003] 가벼운 자동 검사와 원격 이름 제한 적용·효과 확인, 기존 기본 브랜치 보호 보존
  - state: complete; evidence: repo:docs/research/host-policy-hooks-2026-09-18.md#sha256:f0a2d0633f974eeda6fe84e9eb8f9c062b9fd3ba4311c17d6b4239ada1350def
- [x] [BR-004] 세 호스트의 공식 훅 계약·기존 구현·실행 가능성 조사와 `0.11.0` 설계 반영
  - state: complete; evidence: repo:docs/research/host-policy-hooks-2026-09-18.md#sha256:f0a2d0633f974eeda6fe84e9eb8f9c062b9fd3ba4311c17d6b4239ada1350def
- [x] [BR-005] 소스 저장소 훅 설치·거부 확인, 문서·시험·근거·상태 정합화
  - state: complete; evidence: repo:docs/research/host-policy-hooks-2026-09-18.md#sha256:f0a2d0633f974eeda6fe84e9eb8f9c062b9fd3ba4311c17d6b4239ada1350def

## 순서와 검증

1. 기존 `codex/refactor-hive-foundations`를 `refactor/hive-foundations`로 교정, 커밋 불변 확인
2. 하나의 정책 모듈에서 이름·Git 문법·PR 방향 판정, 생성·훅·CI·원격 설정 재사용
3. 격리 저장소의 생성·이름 변경·원격 목적지·태그·추적 참조·기존 훅 보존 시험
4. 소스 저장소의 Git 훅 설치와 GitHub 규칙 적용, 이전 보호 설정의 무변경 확인
5. 호스트별 공식 계약·설치본 가능성·실제 수용의 구분, 후속 제품 계획 작성

## 완료 조건과 한계

- Git 작업 전 실패 시 새 브랜치·커밋·원격 변경 0건 확인
- 잘못된 기존 이름의 정상 이름 교정과 태그·원격 추적 참조에 대한 오탐 0건
- 사후 훅을 생성 차단으로 오인하지 않는 `reference-transaction` 준비 단계 검사
- 서버 이름 규칙의 실제 사용 가능 여부 확인 후 적용; 미지원 규칙의 우회 성공 주장 금지
- 훅·원격 규칙은 이번 사용자 요청에 따른 적용, 다른 저장소·사용자 전역 설정 변경 제외
- 테스트용 경로는 `tests/work/` 안에 한정, 산출물 기록·검토 후 관리
- 호스트 훅은 공식 지원·실제 로드·차단 수용을 구분. 조사 완료만으로 제품 기능 완료 처리 금지
- 전체 `0.11.0` 구현·번호 시험판·안정판 출시는 이번 소스 도구 작업과 별도

## 완료 근거

- 소스 구현: `1ccd352a`, Windows Git 실제 회귀와 관련 검사 35개 성공
- 저장소 Git 훅 설치, 금지 이름 생성 거부와 참조 불변 확인
- 원격 이름 규칙 `23612617` 활성화, API 생성 거부와 기존 기본 브랜치 보호 보존
- CI 정의 구현 완료; 원격 push·실행 미수행으로 실행 성공 주장 제외
- 호스트 조사: [공식 계약과 실제 한계](../../research/host-policy-hooks-2026-09-18.md)
- Git 2.45 직접 이름 변경의 훅 우회는 검사 도구·커밋·게시 거부로 보완
- 제품 훅 구현과 다른 운영체제 수용은 `HK-*`의 미완료 범위 유지
