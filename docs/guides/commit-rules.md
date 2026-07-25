# 커밋 메시지 규칙

## 기본 원칙

- 커밋 하나에 하나의 명확한 관심사
- 필요한 파일만 명시적으로 stage
- 제목은 한국어 Conventional Commits
- AI·bot·agent `Co-Authored-By` trailer 금지
- 제목 끝 문장 부호 금지

## 제목 형식

```text
<type>: <간결한 한국어 설명>
```

예시:

```text
build: Hive source workspace 기반 구성
docs: v1.4 구현 계획 정리
feat: Markdown index 재구축 명령 추가
fix: shared marker 충돌 검사 보강
delete: 폐기된 projection 문서 제거
```

## 타입

| 타입 | 용도 |
| --- | --- |
| `feat` | 사용자 기능 추가 |
| `fix` | 버그 또는 회귀 수정 |
| `docs` | 문서만 변경 |
| `style` | 동작 없는 포맷 변경 |
| `refactor` | 기능 변화 없는 코드 구조 변경 |
| `perf` | 성능 개선 |
| `test` | 테스트만 변경 |
| `build` | 빌드, 도구, 설정, 지침, `.gitignore` |
| `delete` | 삭제 자체가 목적인 변경 |
| `revert` | 이전 커밋 되돌리기 |

## 본문

여러 파일을 설명해야 하면 경로 기준 bullet 사용:

```text
build: Hive source workspace 기반 구성

- crates/, Cargo.toml:
  Rust workspace와 source guard 추가
- .agents/, AGENTS.md:
  개발 지침과 Git 계약 추가
```

긴 배경 문단과 `Constraint:`, `Rejected:`, `Tested:` 같은 일반 trailer 사용 금지.

## 커밋 전후 확인

```bash
git status --short
git diff --cached --check
git diff --cached --stat
git commit
git log -1 --format=%B
```

의도한 파일만 포함되었는지, 메시지에 co-author trailer가 없는지 확인.
