# 안정판 공개 문서 동기화

> Checklist owner: `PUBDOC-*`
> 대상: 공개 안정판 `0.9.5` 교정과 이후 stable release gate
> 제외: test 채널 비공개화, `0.10.0` stable release·tag·publication

## Checklist

- [x] [PUBDOC-001] 공개 안정판 대장과 Scope·Compatibility coverage 계약 추가
- [x] [PUBDOC-002] README·한국어 README·설치 HTML·제품 개요·문서 색인을 `0.9.5`로 교정
- [x] [PUBDOC-003] 일반 사용자 공개 문서의 번호 시험판·test 설치 경로 제거
- [x] [PUBDOC-004] 공개 안정판 문서 검사기와 단위·패키지 README 회귀 추가
- [x] [PUBDOC-005] test·stable 후보/게시 workflow의 channel별 gate 추가
- [ ] [PUBDOC-006] 현재 검증된 정리 커밋과 함께 `develop` 반영·CI 확인
- [ ] [PUBDOC-007] `origin/main` 기준 docs-only PR로 `0.9.5` 공개 문서 교정

## 계약

- 시험판: npm `test`·GitHub prerelease 보존. 일반 사용자 설치 안내 노출 제외
- test channel: 공개 문서는 현재 stable 대장 유지, registry `latest`와 대장 일치
- stable channel: stable candidate source의 대장·날짜·coverage가 요청 stable과 일치
- stable publication 뒤 registry `latest`와 대장 일치
- 공개 안정판 문서: root/한국어 README, 설치 HTML, 제품 개요, 문서 색인. 유지보수자 release evidence·research·plan은 제외
- `0.9.5` main 교정은 문서 전용; 제품·tag·npm·설치 mutation 없음
