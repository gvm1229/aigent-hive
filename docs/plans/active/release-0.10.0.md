# `0.10.0` 시험·정식 출시

> Checklist owner: `REL10-*`
> 완료 선행: `SCP10-001–003`, `CON10-*`, `VWF10-*`, `JDG10-*`, `SKM10-*`, `KRG10-001–016`, `VEC10-*`, `VQR10-*`, `KOR10-*`

## Checklist

- [ ] [REL10-001] 벡터 사용 안내 포함 다음 공개 시험 입력·변경 기록 정합화
- [ ] [REL10-002] 사용자 답변·설정 안내 구현 뒤 Rust·Python·문서·보안·갱신·복구 전체 검사
- [ ] [REL10-003] 다음 공개 시험 게시와 npm `latest=0.9.5` 불변 확인
- [ ] [REL10-004] Windows x64·macOS arm64·Linux musl의 답변·새 세션 설정 공개 시험 수용
- [ ] [REL10-005] accepted test exact source의 protected `main` 통합과 stable candidate
- [ ] [REL10-006] 같은 product bytes의 stable publication·설치·의존 검사
- [ ] [REL10-007] 공개 시험·수용 증거가 갖춰진 뒤 유지보수자의 명시적 `0.10.0` 안정판 승인 수령

## 출시 차단

- `SCP10-002–003`, `KRG10-001–016` 미완료
- `KOR10-002–012` 미완료 또는 새 한국어 product bytes의 번호 시험판·세 host·세 운영체제 수용 부재
- `VQR10-002–010` 미완료 또는 재검증 통과 뒤 `VEC10-008–012` 구현·세 운영체제 수용 부재
- pre-`0.10.0` canonical 지식·프로젝트 보존 증거 부재
- 지원 predecessor direct upgrade의 retired Skill·projection closure 증거 부재
- 게시된 stable release 집합과 historical built-in Skill registry parity 증거 부재
- accepted public test 뒤 product·package·installer 변경. `0.10.0-test.1` 뒤 승인된 한국어 core 범위 추가로 새 번호 시험판 필요
- `VON10-*` 제품 변경 뒤 `test.6` 수용 재사용 금지. 다음 번호 공개 시험과 세 운영체제 수용 필요
- `KTX10-*` 이전·FTS·벡터 재생성 변경 뒤 `test.6` 수용 재사용 금지. 실제 교차 운영체제 이전과 다음 번호 공개 시험 수용 필요
- 유지보수자의 명시적 안정판 승인 부재 — 구현·시험은 승인되었으나 tag·publication은 금지

## 현재 실행 제외

- `REL10-005–007`: 유지보수자 명시 제외. `0.10.0` 안정판 protected `main` 후보·게시·설치·승인 작업 시작 금지
- `REL10-001–004`: 벡터 포함 test.6으로 재수용 완료. [정확한 근거와 한계](../../research/vector-public-test6-2026-08-29.md)

## 비벡터 수정 수용

- `test.5` 후보 `33188952482`: 게시 전 취소. 새 복제본의 시험 경로·Unix 코드 검사·문서 검사 수정 뒤 `test.6`으로 재검증

- `0.10.0-test.4`, 소스 `5cedbed4`: 후보 `33101040467`, 게시 `33102125677`, 공개 설치 수용 `33102491627` 통과
- 실제 실행: Windows x64·macOS arm64·Linux musl x64. 수정 한국어 기능·팩·직접 갱신·복구 검사
- npm 시험 채널 test.4, 안정 채널 `latest=0.9.5` 유지. 벡터 포함 제품의 수용 근거로 재사용 금지
- Windows 전체 Rust·Clippy와 Python 710개 목록 대조: 671 통과·39 운영체제별 건너뜀. 단일 Python 전체 실행 성공이 아닌 수정 재실행 합산 근거

## 이전 공개 시험 근거

- Candidate `32633724977`, publication `32634206001`, `0.10.0-test.1`, Windows 실제 설치·세 운영체제 수용: 당시 승인 범위의 유효 기록
- `0.10.0-test.1`: `KOR10-*` 제외. stable `0.10.0`의 현재 product acceptance 승격 불가

## 2026-08-28 재개방

- 이전 시험 통과 기록은 당시 실행 근거로만 보존
- 활성 팩 미반영·규칙 구조 검사 결손·부정 의미 반전 반례로 현재 완료 판정 취소
- 수정된 소스·새 반례 시험·다음 번호 공개 시험의 근거로 재수용
