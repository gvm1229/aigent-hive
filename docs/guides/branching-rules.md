# 브랜치 규칙

## 장기 브랜치

- `main`: 안정적이고 공개 가능한 기준
- `develop`: 일반 개발과 통합, 검증된 fast-forward direct push 허용

named developer branch 유지 없음.
기본 정책에서 다른 purpose, feature, snapshot branch 생성 금지.

명시 승인 예외 branch: 작업 성격 접두사 사용. `feature/`, `fix/`, `release/`, `docs/`,
`test/`, `refactor/`, `build/`, `chore/` 중 가장 좁은 분류 선택. agent·model·assistant·사람
이름 접두사 사용 금지.

접두사 없는 이름: `main`·`develop`만 허용. `codex/` 등 도구 기본 제안의 예외 적용 금지.

`release/staging`: 정식 릴리스 계획에서 별도 사전 운영 환경이 필요하고 사용자가 승인한 경우만
생성. 생성 시 Pull Request·필수 상태 검사·삭제 차단·force-push 차단의 엄격한
ruleset 적용.

## 초기화

1. `main`에서 검토된 초기 프로젝트 커밋 생성
2. 해당 커밋에서 `develop` 생성
3. 두 브랜치를 `origin`에 push

```bash
git init -b main
git add <reviewed-paths>
git commit
git push -u origin main
git switch -c develop
git push -u origin develop
```

## 일반 작업

- 일반 변경은 `develop`에서 수행
- 일반 검증 커밋은 `develop`에 직접 push
- `develop`의 Pull Request·필수 상태 검사 강제 없음
- 모든 변경 branch는 먼저 `develop`으로 병합
- `main` 대상 Pull Request의 head는 항상 `develop`
- 안정 릴리스는 검증된 `develop → main` Pull Request로만 반영
- `release/` 문서·후보 branch도 `main` 직접 병합 금지, `develop` 통합 뒤 최종 PR 사용
- `main` 직접 일반 커밋 금지
- 다른 branch는 특정 작업에 대한 사용자 명시 승인이 있을 때만 예외적으로 생성

## Push 안전

```bash
git status --short --branch
git remote -v
```

- 대상 remote와 ref 확인
- history rewrite는 명시 요청 때만 수행
- force push가 필요하면 `--force-with-lease`만 사용
- `main`, `develop`, 활성 릴리스 `release/staging` 삭제 금지

## 이름 검사와 설치

```sh
python scripts/branch-policy.py check refactor/hive-foundations
python scripts/branch-policy.py create refactor/example
python scripts/branch-policy.py rename refactor/example refactor/new-name
python scripts/branch-policy.py install-hooks
python scripts/branch-policy.py install-hooks --apply
```

- `check`: 접두사와 Git 문법 검사. `create`: 명시 승인된 작업의 새 브랜치 생성
- `install-hooks`: 미리 보기, `--apply`: 현재 저장소의 참조 변경·게시 전 훅 설치
- 기존 훅·사용자 `core.hooksPath` 발견 시 보존과 수동 통합 안내, 덮어쓰기 제외
- 로컬 참조 검사: 변경 확정 전 거부, 게시 검사: 로컬 이름과 다른 원격 목적지도 검사
- `dev-check.py pre-push`: 이름 검사 후 기존 전체 시험 실행
- 원격 규칙 정본: `scripts/branch-policy.py`; `.github/branch-policy-ruleset.json`은 생성 결과
- 로컬 훅의 우회 가능성 때문에 GitHub 이름 규칙과 PR 검사 병행
- Git 2.45.1의 직접 `git branch -m`은 참조 훅 우회 가능. 이름 변경은 `rename` 도구 사용, 우회된 이름의 커밋·게시도 별도 거부
- 소스 개발 도구에 한정, 소비자 프로젝트에 소스 규칙 자동 설치 제외
