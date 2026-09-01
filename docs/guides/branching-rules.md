# 브랜치 규칙

## 장기 브랜치

- `main`: 안정적이고 공개 가능한 기준
- `develop`: 일반 개발과 통합, 검증된 fast-forward direct push 허용

named developer branch 유지 없음.
기본 정책에서 다른 purpose, feature, snapshot branch 생성 금지.

명시 승인 예외 branch: 작업 성격 접두사 사용. `feature/`, `fix/`, `release/`, `docs/`,
`test/`, `refactor/`, `build/`, `chore/` 중 가장 좁은 분류 선택. agent·model·assistant·사람
이름 접두사 사용 금지.

`staging`: 정식 릴리스 계획에서 별도 사전 운영 환경이 필요하고 사용자가 승인한 경우만
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
- 안정 릴리스는 `develop → main` Pull Request로 반영
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
- `main`, `develop`, 활성 릴리스 `staging` 삭제 금지
