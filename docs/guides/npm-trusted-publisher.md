# npm Trusted Publisher 연결

`aigent-hive`의 npm publication은 GitHub Actions OIDC Trusted Publishing만 사용.
`NPM_TOKEN`·개인 access token·로컬 `npm login`은 필요하지 않음.

## 최초 연결

npm 계정 `gvm1229`로 로그인한 뒤 아래 여섯 package 각각에 같은 Trusted Publisher를 등록.

- `aigent-hive`
- `@aigent-hive/darwin-arm64`
- `@aigent-hive/darwin-x64`
- `@aigent-hive/linux-arm64`
- `@aigent-hive/linux-x64`
- `@aigent-hive/win32-x64`

각 package에서 `Settings` → `Trusted Publisher` → `GitHub Actions`를 열고 다음 값 입력.

- GitHub owner: `gvm1229`
- Repository: `aigent-hive`
- Workflow filename: `release-publish.yml`
- Environment: `release-publication`
- Allowed action: `npm publish`

저장 후 여섯 package가 모두 같은 workflow를 가리키는지 다시 확인. npm은 package마다
Trusted Publisher 설정을 하나만 보유하므로, test와 stable을 서로 다른 workflow로 분리하지 않음.

## 시험판 게시

`develop`의 성공한 `Release candidate` run ID를 사용해 GitHub Actions에서
`Publish npm distribution`을 수동 실행.

- `product_version`: 예: `0.9.0`
- `package_version`: 예: `0.9.0-test.6`
- `candidate_run_id`: 해당 `develop` candidate run ID
- `channel`: `test`

workflow는 candidate가 `develop`에서 성공했는지, package version이 시험판 형식인지,
artifact와 attestation이 해당 commit에 묶였는지 확인. 여섯 package를 npm `test` tag에 게시하고
prerelease·`v0.9.0-test.6` tag를 생성. `latest`는 게시 전후 값이 같아야 성공.

## 정식판 게시

`main`의 성공한 `Release candidate` run ID를 사용.

- `product_version`: 예: `0.9.0`
- `package_version`: `0.9.0`
- `candidate_run_id`: 해당 `main` candidate run ID
- `channel`: `stable`

workflow는 `main` candidate, stable 버전 형식, artifact·attestation·commit 일치를 확인한 뒤
여섯 package를 npm `latest`에 게시하고 normal Release·`v0.9.0` tag를 생성.

## 첫 성공 뒤 token 경로 차단

첫 test publication이 성공한 뒤 npm의 각 package `Publishing access`에서
`Require two-factor authentication and disallow tokens`를 활성화. 그 다음 GitHub repository의
`NPM_TOKEN` secret을 삭제. OIDC Trusted Publishing은 계속 동작하지만 token publication은 차단됨.

공식 설정 화면·제약 조건: <https://docs.npmjs.com/trusted-publishers/>.
