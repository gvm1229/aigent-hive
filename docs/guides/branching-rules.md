# 브랜치 규칙

## 장기 브랜치

- `main`: 안정적이고 공개 가능한 기준
- `develop`: 일반 개발과 통합

named developer branch는 유지하지 않는다.
기본 정책에서 다른 purpose, feature, snapshot branch도 만들지 않는다.

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
- `main`, `develop` 삭제 금지
