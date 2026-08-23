# Source `draft-devlog`

> Checklist owner: `SDB10-*`
> 역할: PortareFolium production MCP 기반 한국어 기술 블로그 초안·명시 발행
> 경계: Source workspace 전용, `harness/`·release bundle·consumer projection 제외

## Checklist

- [x] [SDB10-001] `draft-devlog` 이름, 비공개 초안 기본값, 현재 요청의 명시 발행, 사용자 제공 임시 Bearer, Hive 내부 정보 금지 범위 유지보수자 승인
- [x] [SDB10-002] Source-only Skill·UI metadata·MCP·콘텐츠 정책 reference와 token 없는 runtime state 계약 구현 — `f1c89f09`; quick validation 통과
- [x] [SDB10-003] `tools/list`·`get_schema`·reference 조회·validate·create/update·read-back을 수행하는 표준 라이브러리 helper와 typed auth·rate-limit·uncertain 결과 구현 — mock MCP 17개 회귀 통과
- [x] [SDB10-004] `update-summary|draft-devlog` source inventory·지침·source layout·Skill 문서·제품 결정·bilingual fact 정합화, 제품 projection `0건` — product Skill 26개 byte parity 유지
- [x] [SDB10-005] Mock MCP 기반 token·publication·slug·job field·JSON escaping·MDX·Hive 정보·timeout·read-back 회귀와 Skill quick validation 통과 — focused contract 60개·documentation lane 통과
- [ ] [SDB10-006] 사용자 제공 유효 token으로 기존 비공개 vector 글의 내부 ID 제거, metadata·`published=false`·본문 digest 재조회 수용

## PortareFolium 현재 계약

- Production endpoint: `https://gvm1229-portfolio.vercel.app/api/mcp`
- Source 확인: `C:/Users/hojin/Documents/PortareFolium`, `main@640056c`
- 인증: Raw token SHA-256 저장, revoked·expired SQL filter, `401|-32001`, invalid attempt `429|-32002`
- 순서: `tools/list` 먼저, `get_schema`는 `tools/call`, mutation 뒤 같은 read tool 재조회
- Post 기본값: `published=false`; `published=true`·공개 post 수정은 현재 사용자 요청의 명시 권한 필요
- 현재 호환성: Post `job_field` DB는 `text[]`, MCP schema는 string, handler 정규화 부재. Exact `malformed array literal`에서만 `{<safe-id>}` 1회 대체 입력
- MDX: MCP 작성 content는 trusted evaluate 대상. Markdown 기본, schema 허용 component 외 JSX 금지

## 콘텐츠 금지 범위

- Aigent Hive·Hive·`aigent-hive`, 개발 product version·시험판·release 상태
- Branch·commit·CI run·source workspace 경로·내부 plan·checklist ID
- Bearer·Authorization·secret-shaped 값
- 내부 근거는 방법·수치·한계를 보존한 일반 기술 실험으로 변환

## 성공 기준

- Token 원문: tracked·runtime·stdout·stderr·receipt·fact `0건`
- Create 기본 `published=false`, 명시 `--allow-publish` 없는 발행·공개 글 수정 `0건`
- Production schema drift fail-closed, slug collision overwrite `0건`, mutation uncertainty 재생성 `0건`
- Backtick·ANSI escape·한글·code fence JSON round-trip, 위험 MDX·내부 정보 차단
- Existing vector draft의 `KRG10-014` 제거와 금지 정보 `0건`
