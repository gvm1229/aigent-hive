# 0.17.0 local 인계 — local 세션 시작점

- 작성: 2026-11-28 cloud 세션. 브랜치 `feature/overhaul-7-0.17.0-overnight-loop`, PR [#114](https://github.com/gvm1229/aigent-hive/pull/114)
- 규칙: [실행 장소](../../plans/overhaul/venue.md#넘기기). 결과는 각 항목의 `<ID>.md`에 기록, 통과하면 `complete`, 실패하면 `agent-owned`로 되돌리고 근거 첨부
- 금지: `claude -p`, 공급자 API 호출, 자격 증명 파일 작성·검사, 에이전트의 전역 `~/.claude`·`~/.codex`·`~/.gemini` 직접 수정. 백업은 `~/hive-backups/`

## 대기 항목

| 항목 | local 단계 | 기대 결과 |
| --- | --- | --- |
| OV170-014 | 0.17.0 시험판 설치 뒤 Codex CLI·Claude Code·Antigravity·Codex 앱·Claude 앱에서 Hive Skill 목록 표시 확인 | 33개 이름(`autonomous-workflow`·`ai-slop-cleaner`·`design-curate`·`aphrodite` 포함), 옛 이름 `verified-workflow`·`code-polish` 미표시 |
| OV170-023 | 0.17.0 main 통합 뒤 Actions의 Upstream sync 수동 실행, 필요하면 저장소 비밀값 `UPSTREAM_SYNC_TOKEN` 등록 | 바뀐 원본이 없으면 "No upstream change.", 있으면 `chore/upstream-sync` → develop PR과 CI 실행 |

## 0. 준비

```powershell
git switch develop
git pull --ff-only
hive --version
```
