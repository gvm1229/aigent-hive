# `0.10.0` 시험·정식 출시

> Checklist owner: `REL10-*`
> 선행: `DOC10-*`, `TST10-*`, `GPH10-*`, `HWD10-*`

## Checklist

- [ ] [REL10-001] exact version·build date·release note·package·plugin metadata 정합화
- [ ] [REL10-002] Rust·Python·문서·보안·upgrade·rollback 전체 local gate 통과
- [ ] [REL10-003] 번호 공개 `0.10.0-test.N` candidate·publication과 `latest` 불변 확인
- [ ] [REL10-004] Windows x64·macOS arm64·Linux musl의 Graphify 비활성·활성 공개 시험 수용
- [ ] [REL10-005] accepted test exact source의 protected `main` 통합과 stable candidate
- [ ] [REL10-006] 같은 product bytes의 stable publication·설치·의존 검사

## 출시 차단

- Graphify 하드 게이트 미판정 또는 실패 뒤 대체 범위 미승인
- pre-`0.10.0` canonical 지식·프로젝트 보존 증거 부재
- public test 뒤 product·package·installer 변경
