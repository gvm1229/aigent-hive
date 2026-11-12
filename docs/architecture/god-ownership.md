# 기능과 신의 소속

- 범위: 0.15.0 개발 소스의 공개 최상위 명령과 내장 Skill 전수. 신은 기능 입구의 이름표, 실행 권한이나 새 제품명 아님
- 명령 열의 기존 하위 명령도 같은 소속. 모든 신의 `summon`은 공통 봉투 입구, `hive hades plan`은 삭제 없는 prune 미리보기
- 표시 공통: `--help`·`--version`. 내부 `__*` 명령은 사용자 입구 밖
- 정본: `harness/gods.toml`의 자료형 표, `python scripts/generate-gods.py`로 Rust·Skill·표 생성, `--check`로 일치 확인
- 신 입구: 정본 이름·별칭 명시 요청 전용, 일반 작업 문장은 기존 기능별 Skill 사용. 실제 호스트 선택 성공률 미검증

| 정본 | 맡는 일 | CLI 명령 | Skill |
| --- | --- | --- | --- |
| athena | 계획·검증 | `roster`, `judge`, `report`, `release`, `discord` | `athena`, `verified-workflow`, `multi-goal`, `adversarial-judge`, `judge-evidence`, `run-checkpoint`, `run-resume` |
| artemis | 조사·진단 | `doctor`, `check-target`, `route` | `artemis`, `research-best-practices`, `quick-answer` |
| hephaistos | 구현 | `role`, `agent`, `run`, `loop` | `hephaistos`, `code-polish`, `custom-subagent-create`, `ship` |
| apollon | 글쓰기 | `prompt`, `korean` | `apollon`, `humanize-kor`, `prompt-refine` |
| eileithyia | 설정·갱신 | `setup`, `install`, `update`, `project` | `eileithyia`, `user-setup`, `project-setup`, `product-update`, `project-refresh`, `project-transition`, `amend-directive` |
| poseidon | 조율 | `orchestration`, `session`, `hook`, `policy`, `summon` | `poseidon`, `team-execution`, `run-handoff` |
| demeter | 지식 | `knowledge`, `source-wiki`, `index` | `demeter`, `knowledge-capture`, `knowledge-recall`, `knowledge-scan`, `knowledge-maintain`, `knowledge-promote`, `knowledge-transfer` |
| aither | 사용량 관리 | `usage` | `aither`, `usage-guard` |
| hades | 정리 | `storage`, `prune`, `uninstall` | `hades` |

## 역할 기본값과 이전

- 모델 ID 정본: `harness/roles/defaults.toml`의 `[providers.codex.roles.<역할>]`·`[providers.claude.roles.<역할>]`. 기존 6개 역할의 모델·수준 유지
- 역할 JSON의 `roster-default`: 내장 정본 참조. 원본 지문 검증 뒤 구체 모델로 해석, 실제 투영과 승인에는 구체 모델의 재계산 지문 사용
- 기존 사용자 JSON의 고정 모델: 기존 방식 그대로 읽기, 파일 자동 수정 없음. 새 참조와 같은 구체 모델·수준인 기존 역할은 같은 호스트 파일과 정의 지문
- 사용자 선택: 전역 `~/.hive/roster.toml` 위에 프로젝트 `.hive/roster.toml` 필드별 적용. 작업마다 재읽기, 알 수 없는 필드 거부. 기존 사용자 파일의 자동 이전·삭제 없음

```toml
[workspace]
provider = "codex"

[roles.hive-routine-implementer]
model = "gpt-6.1-sol"
effort = "low"

[gods]
hades = "hive-routine-implementer"
```

- 위 예시는 명시적 사용자 선택 형식, 계정의 해당 모델 지원 증명 아님. `hive roster check`의 공식 호스트 자료 검사와 실제 왕복 수용 필요
- `hive roster --target <프로젝트> --user-root <홈> --output json`: 병합된 구성. `check --role <역할> --capabilities <자료.json>`: 호스트가 전달한 버전·도움말·로그인 상태·모델/수준·훅 자료의 일치 검사
- 호스트 자료의 출처 인증·실제 모델 실행 증명 제외, 검사만으로 활성화 불가. Gemini 0.35.3의 정확한 수준과 공식 구독 상태 미검증 조합 거부
- `hive summon <역할|신> --task <내용|@프로젝트파일> ...`: 기본 미리보기·지문, 외부 서명 Plan 권한과 Prepare 사건을 전달한 경우만 비활성 봉투 게시. 전체 인자는 `hive summon --help` 참조
- task·inbox 게시 뒤 사건 head 결합, 중간 중단의 같은 바이트 재게시 가능. 호스트의 head 확인·별도 실행 권한·typed receipt 확인 전 모델 호출 불가
- 현재 기본 비활성. [OV150-001](../research/overhaul/OV150-001.md)의 실제 Claude Stop→Codex→반환 미검증, OV150-011 왕복 수용 미완료
