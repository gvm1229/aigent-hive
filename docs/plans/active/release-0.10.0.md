# `0.10.0` 시험·정식 출시

> Checklist owner: `SCP10-*`, `REL10-*`
> 완료 선행: `DOC10-*`, `TST10-*`, `GPH10-*`, `HWD10-*`

## Checklist

- [ ] [SCP10-001] 관계·검색 제품 범위와 수락 기준의 사용자 확인·계획 반영: Hive-native Markdown 관계 graph, optional Graphify full-rebuild code-only adapter, FTS·relation query planner, metadata-first retrieval, graph scope 격리·drift gate
- [ ] [SCP10-002] host-owned 프로젝트 Skill 경로 세션 예약 계약 구현: Codex·Antigravity의 `.agents/skills/<skill>/...`, Claude의 `.claude/skills/<skill>/...` 예약 허용, 다른 host 경로의 명시적 `unsupported`, live·unverifiable reservation 충돌 안내의 한정, forbidden path 불변, three-host 회귀·문서 계약 검증
- [ ] [SCP10-003] registered nested-project knowledge scan 복구: 상위 Git repository와 다른 project root 허용, 등록 root 밖 sibling read·write 차단, 전역 `safe.directory` mutation `0건`, symlink·junction·reparse point 탈출 거부, nested Vault 회귀 검증
- [ ] [REL10-001] exact version·build date·release note·package·plugin metadata 정합화
- [ ] [REL10-002] Rust·Python·문서·보안·upgrade·rollback 전체 local gate 통과
- [ ] [REL10-003] 번호 공개 `0.10.0-test.N` candidate·publication과 `latest` 불변 확인
- [ ] [REL10-004] Windows x64·macOS arm64·Linux musl의 승인 제품 범위 공개 시험 수용
- [ ] [REL10-005] accepted test exact source의 protected `main` 통합과 stable candidate
- [ ] [REL10-006] 같은 product bytes의 stable publication·설치·의존 검사

## 출시 차단

- `SCP10-001` 미완료
- `SCP10-002` 미완료
- `SCP10-003` 미완료
- pre-`0.10.0` canonical 지식·프로젝트 보존 증거 부재
- public test 뒤 product·package·installer 변경
