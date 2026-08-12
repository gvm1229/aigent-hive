# 출시 검증용 빌드

대상: Aigent Hive 정식판 후보를 검증하는 개발자·기여자.
일반 사용자 경로: README의 stable 설치 절차.

## 성격과 목적

- Developer test version: 정식판 게시 전 실제 npm·GitHub Release·운영체제별 설치 경로를
  검증하는 번호형 prerelease
- 제품 기능, package metadata, 직접 installer, 문서가 바뀌면 영향 범위를 공개 시험판에서
  먼저 확인
- npm `test` channel과 GitHub prerelease만 사용, npm `latest` 변경 없음
- 실제 결함이나 배포 byte 변경이 없고 tree가 같다면 commit identity나 상태 기록만을 이유로
  새 시험판 생성 금지
- 시험판은 장기 지원 대상 아님. 정식판 게시 뒤 일반 사용자는 정식판으로 이동

## 설치

최신 시험판:

```console
npm install -g aigent-hive@test
hive --version
```

특정 번호 재현 시에만 exact package version 사용:

```console
npm install -g aigent-hive@0.9.2-test.5
hive --version
```

GitHub Release의 직접 installer를 검증할 때는 해당 numbered prerelease에 첨부된
`install.sh`, `install.ps1`, `install.cmd` 사용. 각 파일은 release artifact의 exact version과
SHA-256에 고정.

## 수용 기준

- Native targets (5) and npm package candidate: PASS
- GitHub prerelease·npm `test`의 candidate SHA와 artifact digest 일치
- npm `latest` 불변
- 지원 운영체제의 clean install·upgrade·recovery·version date 확인
- 지식·사용자 설정 보존
- README·설치 안내·npm README의 stable 기본 경로 유지

정식판 생성 기준: 위 검증이 끝난 accepted tree. 정식판 자체를 탐색 시험에 사용하는 행위 금지.
