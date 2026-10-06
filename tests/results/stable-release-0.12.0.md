# 0.12.0 안정판 게시 독립 확인

- 실행: 2026-10-06 Windows·현재 Codex에서 공개npm·GitHub 조회. 후보의 다섯 플랫폼은 원격실행, 현재 사용자 설치0.11.1 보존
- 결과: 여섯 npm 정확한버전·latest0.12.0·시험채널test.3 유지·지문과 정식GitHub출시의 정확한main소스 확인. 정상 정식 태그 생성 성공
- 첫 게시: 여섯 업로드 성공 뒤 darwin-arm64의latest 공개반영 지연으로 실패. 이후 darwin-x64·linux-arm64·win32-x64의 버전·채널 지연도 확인, 전부 보이는 상태에서 동일 후보·기존 지문 복구 성공. 패키지 교체 없음
- Discord: 전송 선택false, 실제 단계skipped. 승인된829자 안내문·지문 보존, 승인 없는 외부 전송 없음
- 범위: 완료6개 기준의 출시·배포 증명, 미완료9개는 별도후속. 실제 계정0·유료전환·현재호출 중단·전체 의미분석·원래macOS오류·소비자갱신 증명 제외

```json
{
  "schema_version": 1,
  "product_version": "0.12.0",
  "package_version": "0.12.0",
  "release_date": "2026-10-06",
  "promoted_test_version": "0.12.0-test.3",
  "promoted_test_source": "e8d4fa891dd42b9594b42b3dd48fc085d9bade32",
  "acceptance_run_id": 37385151833,
  "product_tree_sha256": "sha256:aa9793423ad8b7bd9dd82553189bc1d8cf8ea2e5442ce6a3528580e818307098",
  "integration_pull_request": 72,
  "integration_ci_run_id": 37391954244,
  "source_commit": "a257b4157048b07b40c53ce0f2d50542267772f5",
  "exact_source_ci_run_id": 37393213908,
  "candidate_run_id": 37394215331,
  "publication_run_id": 37396583273,
  "publication_recovery": "Existing recovery with recover_published_packages=true verifies the identical candidate and six already-published package integrities. Independent queries confirmed all six stable versions/latest tags before recovery; no package bytes changed.",
  "failed_publication_run_id": 37395646102,
  "publication_failure_reason": "All six npm uploads succeeded; latest-tag verification for darwin-arm64 failed during public registry propagation. GitHub tag/release creation and Discord send did not run. Exact candidate/package bytes remain unchanged.",
  "send_subscriber_update": false,
  "approved_summary_digest": "sha256:69eada69306704dcf292ffea1ea29dfe7dfab7239f65f4a5e2dffd7a7dc40156",
  "user_installation": "0.11.1; no installation or consumer update requested",
  "status": "published",
  "npm": [
    {
      "package": "aigent-hive",
      "version": "0.12.0",
      "integrity": "sha512-++6wwyngJGTnT5PGkgvrPjEq8BO9LQ2bT17QZYv2JUudPag93s0uDlKdIo0MECDxabpUU22iNCEoXpmr1HUMDQ==",
      "tags": {
        "latest": "0.12.0",
        "test": "0.12.0-test.3"
      }
    },
    {
      "package": "@aigent-hive/darwin-arm64",
      "version": "0.12.0",
      "integrity": "sha512-f5sN0jQDM3tMrOlHmAz8FEnqnGT3WKmcMFTgSakPnEoZ61j+ClqigmcG3sexikSEYuVVqmVFQYRLfTO/TdqIHg==",
      "tags": {
        "latest": "0.12.0",
        "test": "0.12.0-test.3"
      }
    },
    {
      "package": "@aigent-hive/darwin-x64",
      "version": "0.12.0",
      "integrity": "sha512-NQ4yG5/biaRIIXiG01hNvLqnr4jNDKrfk8UWzEk5BusEEeLWHE11ewHc4W/MzTna4dcroA2dzUBZRl8NkegchQ==",
      "tags": {
        "latest": "0.12.0",
        "test": "0.12.0-test.3"
      }
    },
    {
      "package": "@aigent-hive/linux-arm64",
      "version": "0.12.0",
      "integrity": "sha512-caf2YQRAsiKm7UMfEjdnnNBLkGWuSCzXMjQiC7dnMfvzB5pVHg9UYT0v+nHmaQvHGOzQKU9cLXms8rTOi8nhZg==",
      "tags": {
        "latest": "0.12.0",
        "test": "0.12.0-test.3"
      }
    },
    {
      "package": "@aigent-hive/linux-x64",
      "version": "0.12.0",
      "integrity": "sha512-P+SNeOOSFex2gL7iLvtO00na8xns6F1xpotMX0YKdk/axlL112uhc7aq5hm/lFH8j7aZswbFPbT2KSNCap09sQ==",
      "tags": {
        "latest": "0.12.0",
        "test": "0.12.0-test.3"
      }
    },
    {
      "package": "@aigent-hive/win32-x64",
      "version": "0.12.0",
      "integrity": "sha512-7R+0Lcg4ST57romjMjWz6DM9VkCx7sjpWPjUJY1N4zgHPCLYzqUMgJyWlg+nh0gNCmNlISdKb5vAkCI9bhUVDw==",
      "tags": {
        "latest": "0.12.0",
        "test": "0.12.0-test.3"
      }
    }
  ],
  "github_release": {
    "assets": [
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049640",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:45Z",
        "digest": "sha256:0f9cec0890de8bce8a942f598ef3b77932a39092750a6badcd00b5879c8fb32a",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmado",
        "label": "",
        "name": "aigent-hive-0.12.0-aarch64-apple-darwin.attestation.jsonl",
        "size": 11020,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:45Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-aarch64-apple-darwin.attestation.jsonl"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049506",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:41Z",
        "digest": "sha256:9375ebaf4efefdd76a3d1d4a8422db011be3681a2ba3250c28cc8ec87154e0d9",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmabi",
        "label": "",
        "name": "aigent-hive-0.12.0-aarch64-apple-darwin.tar.gz",
        "size": 10205961,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:42Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-aarch64-apple-darwin.tar.gz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049542",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:43Z",
        "digest": "sha256:a103f8feadc70267c17d777d13e707f0b6a3b2f2c4e2858624ea8e931bd0dbd1",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmacG",
        "label": "",
        "name": "aigent-hive-0.12.0-aarch64-apple-darwin.tar.gz.sha256",
        "size": 113,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:43Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-aarch64-apple-darwin.tar.gz.sha256"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049646",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:45Z",
        "digest": "sha256:c1f850382a546de2ec37ec0ce5428dd3f698921bc26a5e36eaf106da453a91c9",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmadu",
        "label": "",
        "name": "aigent-hive-0.12.0-aarch64-unknown-linux-musl.attestation.jsonl",
        "size": 10769,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:45Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-aarch64-unknown-linux-musl.attestation.jsonl"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049511",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:41Z",
        "digest": "sha256:25c79e5065d971d72d83a5b7837bd2357930d4e5f16f243cf251dee26936afa2",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmabn",
        "label": "",
        "name": "aigent-hive-0.12.0-aarch64-unknown-linux-musl.tar.gz",
        "size": 11076382,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:42Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-aarch64-unknown-linux-musl.tar.gz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049549",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:43Z",
        "digest": "sha256:97dad65854a3e02979ba8d79ba702670a2e2598760e8063566dd794bde94676c",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmacN",
        "label": "",
        "name": "aigent-hive-0.12.0-aarch64-unknown-linux-musl.tar.gz.sha256",
        "size": 119,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:43Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-aarch64-unknown-linux-musl.tar.gz.sha256"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049652",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:45Z",
        "digest": "sha256:7a85fa5d9eeff9ff724dc1cfffe58899f04bfa9487b58a580389aa6786d31adb",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmad0",
        "label": "",
        "name": "aigent-hive-0.12.0-npm-umbrella.attestation.jsonl",
        "size": 10976,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:45Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-npm-umbrella.attestation.jsonl"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049653",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:45Z",
        "digest": "sha256:1b6f6aaf67facf52a65a4f4dd9d68c21ae37d2e6b07b41392851d6b12c5b17b3",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmad1",
        "label": "",
        "name": "aigent-hive-0.12.0-release-bundle.attestation.jsonl",
        "size": 10738,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:45Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-release-bundle.attestation.jsonl"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049522",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:41Z",
        "digest": "sha256:536395822db80ef1744b2268665e1b753b32a3e98481d90eaa8cc36c14ba986b",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmaby",
        "label": "",
        "name": "aigent-hive-0.12.0-release-bundle.tar.gz",
        "size": 110125652,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:46Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-release-bundle.tar.gz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049550",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:43Z",
        "digest": "sha256:c6f25a18f3249662025a2f1ac44db8e91304a0379b61ed2597be80e68a382d5b",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmacO",
        "label": "",
        "name": "aigent-hive-0.12.0-release-bundle.tar.gz.sha256",
        "size": 107,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:43Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-release-bundle.tar.gz.sha256"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049658",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:45Z",
        "digest": "sha256:1fa124f9f1daacb03d9c7b46bfaea023e9858bfa9043610eeac23f28768214a8",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmad6",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-apple-darwin.attestation.jsonl",
        "size": 11012,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:46Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-apple-darwin.attestation.jsonl"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049520",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:41Z",
        "digest": "sha256:13050f7c314df9a1309de5f5f8ea24aef878b0f18133b7d87ced0d162d6ec37f",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmabw",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-apple-darwin.tar.gz",
        "size": 11125859,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:42Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-apple-darwin.tar.gz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049556",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:43Z",
        "digest": "sha256:2b450cca68b1d0c6d91ab9d693127324f7a359839d2a8e35bf357fa1cbc19791",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmacU",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-apple-darwin.tar.gz.sha256",
        "size": 112,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:43Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-apple-darwin.tar.gz.sha256"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049660",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:46Z",
        "digest": "sha256:394e00c42f4760a9ee4625de917b461e9775f6c9fa92d2294726c0d1c5173715",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmad8",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-pc-windows-msvc.attestation.jsonl",
        "size": 10852,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:46Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-pc-windows-msvc.attestation.jsonl"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049544",
        "contentType": "application/zip",
        "createdAt": "2026-10-06T00:58:43Z",
        "digest": "sha256:b867a1e8c718fd42fa5ebb2fcd3fb6fb1d363ad8946c7be005582c3717b0639e",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmacI",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-pc-windows-msvc.zip",
        "size": 10688610,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:43Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-pc-windows-msvc.zip"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049572",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:43Z",
        "digest": "sha256:c4150c8ca1ea5bf6f4e7d189f85ad906595255823d283570a059c8fa44a9a366",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmack",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-pc-windows-msvc.zip.sha256",
        "size": 113,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:44Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-pc-windows-msvc.zip.sha256"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049659",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:46Z",
        "digest": "sha256:066b58bb86f78cd9810dc5ef696b0ba3edf71eb3d38c7a6e0cdbcea631b7e934",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmad7",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-unknown-linux-musl.attestation.jsonl",
        "size": 11047,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:46Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-unknown-linux-musl.attestation.jsonl"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049523",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:42Z",
        "digest": "sha256:df0ce87dd9098465d4f5a487283b7d10e9c68ed6dd28e6094c8ec97ed554142e",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmabz",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-unknown-linux-musl.tar.gz",
        "size": 11717797,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:42Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-unknown-linux-musl.tar.gz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049574",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:43Z",
        "digest": "sha256:74214e91a1ac593ad2761f09797e7595ad94e5b287075ce85c272b48d3073cf7",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmacm",
        "label": "",
        "name": "aigent-hive-0.12.0-x86_64-unknown-linux-musl.tar.gz.sha256",
        "size": 118,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:44Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0-x86_64-unknown-linux-musl.tar.gz.sha256"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049580",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:44Z",
        "digest": "sha256:1b4c36b70f5a0afaebd209843423e40fb3454ee9d923cb89c495fa29e05df2a0",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmacs",
        "label": "",
        "name": "aigent-hive-0.12.0.tgz",
        "size": 17164,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:44Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-0.12.0.tgz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049591",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:44Z",
        "digest": "sha256:a6cbd3f5f62edb93f726595788446c22c0ef9ccd40c23a9e07a4eddf9fff6a52",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmac3",
        "label": "",
        "name": "aigent-hive-darwin-arm64-0.12.0.tgz",
        "size": 10499434,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:44Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-darwin-arm64-0.12.0.tgz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049615",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:44Z",
        "digest": "sha256:8d63c3c48dc9c2099d82d2b0e4c2a7e706744e559028753265f22f689f0f2132",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmadP",
        "label": "",
        "name": "aigent-hive-darwin-x64-0.12.0.tgz",
        "size": 11243415,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:44Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-darwin-x64-0.12.0.tgz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049618",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:44Z",
        "digest": "sha256:973fce456fcde5012d9878aa990b04fd763f583278201251a7e01f3ab1c7d2e4",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmadS",
        "label": "",
        "name": "aigent-hive-linux-arm64-0.12.0.tgz",
        "size": 11354375,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:45Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-linux-arm64-0.12.0.tgz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049621",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:44Z",
        "digest": "sha256:f4faa550484ff4cba52802b386c397f8e6758e76e46c4f36e75c8d68da11560f",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmadV",
        "label": "",
        "name": "aigent-hive-linux-x64-0.12.0.tgz",
        "size": 11828520,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:45Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-linux-x64-0.12.0.tgz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049637",
        "contentType": "application/x-gtar",
        "createdAt": "2026-10-06T00:58:45Z",
        "digest": "sha256:364b1e21a445280e20704621a1b54d99dc7ef140a9f15f66bb9e3115aad08fc5",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmadl",
        "label": "",
        "name": "aigent-hive-win32-x64-0.12.0.tgz",
        "size": 10593130,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:45Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/aigent-hive-win32-x64-0.12.0.tgz"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049507",
        "contentType": "application/octet-stream",
        "createdAt": "2026-10-06T00:58:41Z",
        "digest": "sha256:09042f6f51f56515366d1fbf869cedd7cc1a20bbe403a249c1e9b953ec633997",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmabj",
        "label": "",
        "name": "install.cmd",
        "size": 820,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:41Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/install.cmd"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049509",
        "contentType": "application/x-powershell",
        "createdAt": "2026-10-06T00:58:41Z",
        "digest": "sha256:5deaee892c28c26b7f9c22fecf9884fb848e815db14208a3fa27efa39d3a56bc",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmabl",
        "label": "",
        "name": "install.ps1",
        "size": 17706,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:41Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/install.ps1"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049505",
        "contentType": "application/x-shellscript",
        "createdAt": "2026-10-06T00:58:41Z",
        "digest": "sha256:caef4894cadac88793545638ca3c2cf29f6bb48e7f9cce88f8dd51d9cf62692f",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmabh",
        "label": "",
        "name": "install.sh",
        "size": 14456,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:41Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/install.sh"
      },
      {
        "apiUrl": "https://api.github.com/repos/gvm1229/aigent-hive/releases/assets/614049671",
        "contentType": "application/json",
        "createdAt": "2026-10-06T00:58:46Z",
        "digest": "sha256:f13978e2f3ed6f8268aa47ab963dba7fd7433f53f5bd083409115c9ddc0c66b5",
        "downloadCount": 0,
        "id": "RA_kwDOTg1R7s4kmaeH",
        "label": "",
        "name": "release-integrity-receipt.json",
        "size": 775,
        "state": "uploaded",
        "updatedAt": "2026-10-06T00:58:46Z",
        "url": "https://github.com/gvm1229/aigent-hive/releases/download/v0.12.0/release-integrity-receipt.json"
      }
    ],
    "isDraft": false,
    "isPrerelease": false,
    "tagName": "v0.12.0",
    "targetCommitish": "a257b4157048b07b40c53ce0f2d50542267772f5",
    "url": "https://github.com/gvm1229/aigent-hive/releases/tag/v0.12.0"
  },
  "limits": "Verified stable distribution promotes accepted test.3 product sources. Nine actual host/semantic/original-macOS criteria remain explicitly deferred; no actual account exhaustion, paid-credit transition or current-turn interruption proof. Current-user installation and Orireki consumers remain unchanged."
}
```
