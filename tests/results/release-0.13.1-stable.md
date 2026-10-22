# 0.13.1 정식 공개 독립 확인

현재 Windows에서 실제 공개 파일·태그·소스·승인 공지 요청 수락의 대조. 제품 동작 수용은 공개test.4와 별도 기록, 실제 사용자 설치 변경 없음.

```json
{
  "schema_version": 1,
  "phase": "published-stable-verified",
  "product_version": "0.13.1",
  "source_commit": "c37882ca7a66e895961f630a3d40c584d910279a",
  "product_tree_sha256": "sha256:f52f565ffebb025fd2ccb55df51ce31d9fdacee57d03a453f6c5abaaecceb8f3",
  "promoted_public_test": "0.13.1-test.4",
  "public_acceptance_run_id": 39764305649,
  "earlier_nested_acceptance_run_id": 39659417976,
  "failed_candidate": {
    "run_id": 39763515230,
    "reason": "Wrong parent workflow identity supplied; native-runtime wrapper is not the required directly dispatched Public test acceptance."
  },
  "failed_direct_acceptance": {
    "run_id": 39763808385,
    "reason": "Prepared stable documentation said 0.13.1 while registry latest remained 0.13.0; failed before functional tests."
  },
  "main_pull_request": 78,
  "main_ci_run_id": 39762073421,
  "candidate_run_id": 39765168957,
  "failed_publication_run_ids": [
    39767478065
  ],
  "publication_run_id": 39768286874,
  "public_verification": {
    "schema_version": 1,
    "status": "passed",
    "product_version": "0.13.1",
    "source_commit": "c37882ca7a66e895961f630a3d40c584d910279a",
    "release_url": "https://github.com/gvm1229/aigent-hive/releases/tag/v0.13.1",
    "packages": [
      {
        "package": "aigent-hive",
        "version": "0.13.1",
        "latest": "0.13.1",
        "test": "0.13.1-test.4",
        "sha256": "689fa33e8dc82ca0826c45b9ec3090e9837002b01bb836d8c776a9c5f65baaea",
        "sha512_integrity": "sha512-CrFWP87mMZkV2e2NobNIjyXys76H67EtDwbuom8iv3oYIHAuIe30Py7qMAURbdk2vcvvwcr/NUA6uYlY6gmaXw==",
        "size": 17549,
        "npm_github_bytes_match": true
      },
      {
        "package": "@aigent-hive/win32-x64",
        "version": "0.13.1",
        "latest": "0.13.1",
        "test": "0.13.1-test.4",
        "sha256": "0ca6f5f54232163d896fc0c36c8368617cda10b1356bcae8322fba161f8fc348",
        "sha512_integrity": "sha512-RRA/+0jh9yVH+00nWtETpr2uZJNaa4p5ZXmo8132ir+pNsFV97bnYUt483GkIKjvSKJPLTvbwHna1ll7LgbvTA==",
        "size": 11026467,
        "npm_github_bytes_match": true
      },
      {
        "package": "@aigent-hive/darwin-arm64",
        "version": "0.13.1",
        "latest": "0.13.1",
        "test": "0.13.1-test.4",
        "sha256": "fe456dc499f02d0c5258bef877078e0c3a18f744ca4a20878510e15e15c5e9e9",
        "sha512_integrity": "sha512-07C2pPXYwxbC/RiJ+s09K8nddWRgZS217ufMOKufFMU3rFZTZlTXqFiDiEHE5bBJg2kJkx+6yJ338/ynR2Gf+w==",
        "size": 10624317,
        "npm_github_bytes_match": true
      },
      {
        "package": "@aigent-hive/darwin-x64",
        "version": "0.13.1",
        "latest": "0.13.1",
        "test": "0.13.1-test.4",
        "sha256": "9d84942dabeb0f514fd2ea3ca65eeaee6ecc842002f141fd2ac5aa4b931f7274",
        "sha512_integrity": "sha512-1uM277U09JdbeyCEc+1YVs/JRt6yT2UruggJlO1/0nv8haXhqTOiRoUzdqTB1Y+8MAMl5ELaSCfUaixYGCu19g==",
        "size": 11378219,
        "npm_github_bytes_match": true
      },
      {
        "package": "@aigent-hive/linux-x64",
        "version": "0.13.1",
        "latest": "0.13.1",
        "test": "0.13.1-test.4",
        "sha256": "bc051755b68219798763e03e54bd275a37099cf761e4ccae5c735f5eb09f641c",
        "sha512_integrity": "sha512-XJjl1SQeRhXBj0o8RvR4aJ/lt39X8hv4mQEXvKn5AAHOA//ZpZDVhx/8r0tW8WOOcVh4jfMVZVk6n0DRHQQDhw==",
        "size": 11988926,
        "npm_github_bytes_match": true
      },
      {
        "package": "@aigent-hive/linux-arm64",
        "version": "0.13.1",
        "latest": "0.13.1",
        "test": "0.13.1-test.4",
        "sha256": "7fe8aa37509404857089e2d2579cb51d73a5290b71cb08dba8cd882ec45bf366",
        "sha512_integrity": "sha512-uy0Bq2H6hXkIXRCdTNs1IUrJcDT+QUYXoZ6kRjZzgD8FNo82mhSXeGTi0nGZsc8/gt9CAVe8/1VWT8tMSxcVJg==",
        "size": 11507420,
        "npm_github_bytes_match": true
      }
    ],
    "github_asset_count": 29,
    "assets": [
      {
        "name": "aigent-hive-0.13.1-aarch64-apple-darwin.attestation.jsonl",
        "size": 10859,
        "sha256": "45021ff02b00457d34ce403ec5bc3d756a85c9884940cf8f4b8d2cbe36c8ebc8"
      },
      {
        "name": "aigent-hive-0.13.1-aarch64-apple-darwin.tar.gz",
        "size": 10326054,
        "sha256": "3401aa486b46c2898b7b0e69a9c6540e6d8182644cd5379fc69912f326087b9c"
      },
      {
        "name": "aigent-hive-0.13.1-aarch64-apple-darwin.tar.gz.sha256",
        "size": 113,
        "sha256": "1409a37a643b579ac480723f0f44e87a18027711adaef6892fa46a5c59def60a"
      },
      {
        "name": "aigent-hive-0.13.1-aarch64-unknown-linux-musl.attestation.jsonl",
        "size": 10914,
        "sha256": "78cfd2828c925415080748c89f0aa19693601c87a8e92e920959dbea9a4906b8"
      },
      {
        "name": "aigent-hive-0.13.1-aarch64-unknown-linux-musl.tar.gz",
        "size": 11229606,
        "sha256": "69d4887571b2a1cde9bb4b8a35ed62f81390b6219e8b8e466549dfabff06deed"
      },
      {
        "name": "aigent-hive-0.13.1-aarch64-unknown-linux-musl.tar.gz.sha256",
        "size": 119,
        "sha256": "885e0195969a899a4ebdcf79a0d9b9624c611496a34b75eee6c4e592e8c7dc59"
      },
      {
        "name": "aigent-hive-0.13.1-npm-umbrella.attestation.jsonl",
        "size": 10898,
        "sha256": "ef2f31ccf6f7eae738fda02726e1ce7ea20213de3543dac05a407c25a49502b5"
      },
      {
        "name": "aigent-hive-0.13.1-release-bundle.attestation.jsonl",
        "size": 10718,
        "sha256": "0ab115133b9b8d60f822cf76fb7ac134ec510cf7962395db41bbcbbb48e50777"
      },
      {
        "name": "aigent-hive-0.13.1-release-bundle.tar.gz",
        "size": 112121128,
        "sha256": "28d31c847d9c2476db256cc3e296e864f6c1190f120b5d265704d0ba75973de6"
      },
      {
        "name": "aigent-hive-0.13.1-release-bundle.tar.gz.sha256",
        "size": 107,
        "sha256": "526d74d1ff5d3fbd2de9c68fb79024e01fa7d4b7eae7d0fdca93ce4c0bb9d52d"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-apple-darwin.attestation.jsonl",
        "size": 10996,
        "sha256": "d3943a2dd507092039d756b7a3f8d942f25153571097acc90954d23deb5a8cb0"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-apple-darwin.tar.gz",
        "size": 11261315,
        "sha256": "d1a6ba49dc158edf05831ce01c66a3ba3c0603ad78ba7906a38db7381c072046"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-apple-darwin.tar.gz.sha256",
        "size": 112,
        "sha256": "8e857b3918306d7b0e18953532542e55b44773d85e891d55987536bac02bf057"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-pc-windows-msvc.attestation.jsonl",
        "size": 10915,
        "sha256": "4542c5bbd20bad2ecfa8e381efa49afd57fa6c9b3205be84ff189d364919c828"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-pc-windows-msvc.zip",
        "size": 11124786,
        "sha256": "434d03b07be17e34f4e3dfff8d776677851186d65c06eb3493ed316cd7c94717"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-pc-windows-msvc.zip.sha256",
        "size": 113,
        "sha256": "49ec0789f14a51c2d585468f55c3ebc11bba921c5dd7144348c05157ea49487e"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-unknown-linux-musl.attestation.jsonl",
        "size": 11000,
        "sha256": "d629273df7bca7c1fb6fbed0f2b1dbaee063026dcb32512574c10f9c7c71744c"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-unknown-linux-musl.tar.gz",
        "size": 11876971,
        "sha256": "dbbd62eca56cbde7e367caf8661c83baead2f789268d4ea870c72453842b34a3"
      },
      {
        "name": "aigent-hive-0.13.1-x86_64-unknown-linux-musl.tar.gz.sha256",
        "size": 118,
        "sha256": "ae14eb5381adea1cfd2cbb4fb5ab547e6c4985d450f29372fc4bf288a1293ea0"
      },
      {
        "name": "aigent-hive-0.13.1.tgz",
        "size": 17549,
        "sha256": "689fa33e8dc82ca0826c45b9ec3090e9837002b01bb836d8c776a9c5f65baaea"
      },
      {
        "name": "aigent-hive-darwin-arm64-0.13.1.tgz",
        "size": 10624317,
        "sha256": "fe456dc499f02d0c5258bef877078e0c3a18f744ca4a20878510e15e15c5e9e9"
      },
      {
        "name": "aigent-hive-darwin-x64-0.13.1.tgz",
        "size": 11378219,
        "sha256": "9d84942dabeb0f514fd2ea3ca65eeaee6ecc842002f141fd2ac5aa4b931f7274"
      },
      {
        "name": "aigent-hive-linux-arm64-0.13.1.tgz",
        "size": 11507420,
        "sha256": "7fe8aa37509404857089e2d2579cb51d73a5290b71cb08dba8cd882ec45bf366"
      },
      {
        "name": "aigent-hive-linux-x64-0.13.1.tgz",
        "size": 11988926,
        "sha256": "bc051755b68219798763e03e54bd275a37099cf761e4ccae5c735f5eb09f641c"
      },
      {
        "name": "aigent-hive-win32-x64-0.13.1.tgz",
        "size": 11026467,
        "sha256": "0ca6f5f54232163d896fc0c36c8368617cda10b1356bcae8322fba161f8fc348"
      },
      {
        "name": "install.cmd",
        "size": 820,
        "sha256": "6f332f34325249d4b5ee4a9a41cc687d51977a026c2faf7bc54fee86002048ad"
      },
      {
        "name": "install.ps1",
        "size": 17706,
        "sha256": "20a7acf1007cae609f96f5be83922f253e42321f6fa571bbb712343a21e2adec"
      },
      {
        "name": "install.sh",
        "size": 14456,
        "sha256": "93c3c9ab273a664b412c0ce5971049c42f89fcc7f7b0c3e23d34664fc978b98d"
      },
      {
        "name": "release-integrity-receipt.json",
        "size": 775,
        "sha256": "ce4e2e5154025545f069413c552dceebbfaae82be3a4fa9be536922ef23d5f4e"
      }
    ],
    "annotated_tag_source_verified": true,
    "real_user_installation_changed": false,
    "limits": [
      "Distribution bytes and source only; functional acceptance is separately bound to public test.4."
    ],
    "collector_sha256": "447b510088fa21eb3bc927ca152d0480dc42a0d1c91a3edcff3e6064f4de6859"
  },
  "subscriber_notice": {
    "approved_characters": 854,
    "approved_sha256": "ed0d81b02b1dfac31393280287bcc3befc5686320e860cff8c4781de6582f1d6",
    "banner_and_summary_http_accepted": true,
    "reader_delivery_or_read_receipt_not_claimed": true
  },
  "current_user": {
    "installed_version": "0.13.0",
    "global_threshold_remaining_percent": 5,
    "zero_quota_protection_preserved": true,
    "monitor_task_disabled": true,
    "real_user_stable_installation": false,
    "local_monitor_activation_authority_pending": true,
    "fresh_update_discovery": {
      "code": "hive.update-available",
      "current_version": "0.13.0",
      "installed": false,
      "latest_version": "0.13.1",
      "update_available": true,
      "settings_monitor_history_binary_preserved": true,
      "prior_daily_check_cache_backed_up": true
    },
    "saved_monitor_choice": true,
    "stored_monitor_version": "0.12.1"
  },
  "limits": [
    "Functional qualification belongs to accepted public test.4 and current Windows native interruption evidence.",
    "Ordinary external commands already started may continue; local AI sessions and subagents remain stop targets.",
    "Other OS, remote/cloud direct interruption unsupported; simultaneous server/billing termination not guaranteed.",
    "Previous unexpected Codex browser crash cause remains unknown; no app-termination fallback added."
  ],
  "failed_publication_reason": "All six npm uploads accepted, but latest channel propagation exceeded the initial wait; same candidate recovered after all six exact versions became visible."
}
```
