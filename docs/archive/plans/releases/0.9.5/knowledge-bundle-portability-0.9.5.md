# `0.9.5` 지식 번들 이식 안내

> Checklist owner: `KBP95-*`
> 대상: 전역 `.hivekb` 내보내기·가져오기 안내와 macOS 실제 수용

## 범위

- 사용자 루트: `.hive` 하위 폴더가 아닌 사용자 홈 디렉터리
- 번들 경로: 운영체제별 셸 변수 혼용 없는 명시적 절대 경로
- 가져오기 순서: 파일·SHA-256 확인 → `--dry-run` → 충돌·검증 오류 `0건`일 때만 `--apply`
- 번들 내용: 전역 정본 Markdown만 이동, project-private collection·SQLite·runtime·비밀 값 제외

## Checklist

- [x] `KBP95-001` `README.md`와 `docs/hive-install-guide.ko.html`에 macOS/Linux 셸과 Windows PowerShell의 독립 명령 예시, 사용자 루트 의미, 절대 bundle 경로, SHA-256 확인과 dry-run 선행 조건 추가
- [x] `KBP95-002` 문서·패키지 정적 검증에 Windows 전용 `$env:USERPROFILE`과 macOS/Linux `$HOME`의 교차 사용 금지, 정확한 `hive knowledge export|import` 계약 추가
- [x] `KBP95-003` 현재 macOS의 격리 사용자 루트에서 전역 지식 내보내기 → SHA-256 확인 → dry-run import → apply import → lint 수용. 원본 사용자 루트·소스 저장소·SQLite 번들 복사 없음 확인

## 수락 기준

- 사용자는 현재 운영체제의 셸 예시만 복사 가능
- `--user-root`는 항상 홈 디렉터리, `.hive`는 Hive가 그 아래에 관리
- 실패한 SHA-256·dry-run·충돌에서 `--apply` 실행 `0회`
