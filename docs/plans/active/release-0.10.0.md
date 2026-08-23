# `0.10.0` 시험·정식 출시

> Checklist owner: `REL10-*`
> 완료 선행: `SCP10-001–003`, `CON10-*`, `VWF10-*`, `JDG10-*`, `SKM10-*`, `KRG10-001–016`, `VEC10-*`, `VQR10-*`, `KOR10-*`

## Checklist

- [ ] [REL10-001] 한국어 언어 core를 포함한 exact version·build date·release note·package·plugin metadata 정합화 — 기존 `0.10.0-test.1` 근거는 과거 범위로 보존
- [ ] [REL10-002] 새 product bytes의 Rust·Python·문서·보안·upgrade·rollback 전체 local gate 통과
- [ ] [REL10-003] 번호 공개 `0.10.0-test.2` 이상 candidate·publication과 npm `latest=0.9.5` 불변 확인
- [ ] [REL10-004] Windows x64·macOS arm64·Linux musl에서 한국어 core·`humanize-kor`·upstream pack update를 포함한 공개 시험 수용
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
- 유지보수자의 명시적 안정판 승인 부재 — 구현·시험은 승인되었으나 tag·publication은 금지

## 현재 실행 제외

- `REL10-005–007`: 유지보수자 명시 제외. `0.10.0` 안정판 protected `main` 후보·게시·설치·승인 작업 시작 금지
- `REL10-001–004`: 한국어 core 구현 뒤 새 번호 공개 시험과 세 운영체제 시험 수용 범위로 재개방

## 이전 공개 시험 근거

- Candidate `32633724977`, publication `32634206001`, `0.10.0-test.1`, Windows 실제 설치·세 운영체제 수용은 당시 승인 범위의 유효한 기록임
- `0.10.0-test.1`에는 `KOR10-*`가 없으므로 stable `0.10.0`의 현재 product acceptance로 승격하지 않음
