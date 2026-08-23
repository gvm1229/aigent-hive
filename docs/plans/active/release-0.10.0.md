# `0.10.0` 시험·정식 출시

> Checklist owner: `REL10-*`
> 완료 선행: `SCP10-001–003`, `CON10-*`, `VWF10-*`, `JDG10-*`, `SKM10-*`, `KRG10-001–016`

## Checklist

- [x] [REL10-001] exact version·build date·release note·package·plugin metadata 정합화 — `1b755a9`, `0ba5dfb`; `0.10.0` source와 `0.10.0-test.1` package metadata
- [x] [REL10-002] Rust·Python·문서·보안·upgrade·rollback 전체 local gate 통과 — Rust workspace·Clippy·fmt 통과, Python documentation 45·security 103·contract 380·integration 84·release 58 통과
- [ ] [REL10-003] 번호 공개 `0.10.0-test.N` candidate·publication과 `latest` 불변 확인
- [ ] [REL10-004] Windows x64·macOS arm64·Linux musl의 승인 제품 범위 공개 시험 수용
- [ ] [REL10-005] accepted test exact source의 protected `main` 통합과 stable candidate
- [ ] [REL10-006] 같은 product bytes의 stable publication·설치·의존 검사
- [ ] [REL10-007] 공개 시험·수용 증거가 갖춰진 뒤 유지보수자의 명시적 `0.10.0` 안정판 승인 수령

## 출시 차단

- `SCP10-002–003`, `KRG10-001–016` 미완료
- pre-`0.10.0` canonical 지식·프로젝트 보존 증거 부재
- 지원 predecessor direct upgrade의 retired Skill·projection closure 증거 부재
- 게시된 stable release 집합과 historical built-in Skill registry parity 증거 부재
- public test 뒤 product·package·installer 변경
- 유지보수자의 명시적 안정판 승인 부재 — 구현·시험은 승인되었으나 tag·publication은 금지

## 현재 실행 제외

- `REL10-005–007`: 유지보수자 명시 제외. `0.10.0` 안정판 protected `main` 후보·게시·설치·승인 작업 시작 금지
- `REL10-001–004`: 구현·검증·번호 공개 시험과 세 운영체제 시험 수용 범위
