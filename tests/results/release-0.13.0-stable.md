# 0.13.0 정식 공개 독립 확인

검증 범위·실제 소스·공개 파일·공지 요청 수락의 구조화 기록. 실제 사용자 설치나 새 실시간 중단 시험과 구분.

```json
{
  "schema_version": 1,
  "phase": "published-stable-verified",
  "product_version": "0.13.0",
  "source_commit": "44de144722a6f99a79adcb512418d44d19f0d239",
  "product_tree_sha256": "sha256:f1c14a3998bc456ead2a201274a357255533dfa9d457ddc9ea090c016f9d4c3d",
  "promoted_public_test": "0.13.0-test.1",
  "public_acceptance_run_id": 39078351729,
  "main_pull_request": 76,
  "main_ci_run_id": 39080233513,
  "candidate_run_id": 39080948835,
  "initial_publication_run_id": 39081850375,
  "initial_publication_result": "failed-registry-tag-observation-before-github-release",
  "publication_run_id": 39082219733,
  "public_verification": {
    "schema_version": 1,
    "product_version": "0.13.0",
    "package_version": "0.13.0",
    "phase": "passed-independent-public-verification",
    "channel": "stable",
    "source_commit": "44de144722a6f99a79adcb512418d44d19f0d239",
    "release_url": "https://github.com/gvm1229/aigent-hive/releases/tag/v0.13.0",
    "github_prerelease": false,
    "github_asset_count": 29,
    "packages": [
      {
        "package": "aigent-hive",
        "version": "0.13.0",
        "latest": "0.13.0",
        "test": "0.13.0-test.1",
        "dist_integrity": "sha512-GbUcbpv+Wtww0rxEhCg9nuiLSEg6wEejD7JjY1Db4l3Llasdu+StHvnAAlAJt8tMg/SaIu/+82Srr3chsSH+yw==",
        "archive_sha256": "a18ea19509a3a251e535f0d41b3a327090c19a236c7da37d1710efab09cdc147",
        "github_asset_sha256": "sha256:a18ea19509a3a251e535f0d41b3a327090c19a236c7da37d1710efab09cdc147",
        "length": 17247,
        "asset": "aigent-hive-0.13.0.tgz"
      },
      {
        "package": "@aigent-hive/darwin-arm64",
        "version": "0.13.0",
        "latest": "0.13.0",
        "test": "0.13.0-test.1",
        "dist_integrity": "sha512-Jriq5qzim07ttnO9UPfdb68AT0Oq1gQnFXGGCoHkfyQjNnHe9Ct56DNjA0kXJKN1RULYyiloE4gHoF76pMQE3w==",
        "archive_sha256": "f1befd25c6c7c73f5320bdf285fe0d77790987318c1184e4289e14eda9db15ee",
        "github_asset_sha256": "sha256:f1befd25c6c7c73f5320bdf285fe0d77790987318c1184e4289e14eda9db15ee",
        "length": 10547528,
        "asset": "aigent-hive-darwin-arm64-0.13.0.tgz"
      },
      {
        "package": "@aigent-hive/darwin-x64",
        "version": "0.13.0",
        "latest": "0.13.0",
        "test": "0.13.0-test.1",
        "dist_integrity": "sha512-tCDbvCRKjBi6LpxUiapRec/9q/5Ry7doP6Mtr9jP4tkw9leuG7WET22aRAzkmBJWN/VXlHpNfPC5PJFZqvtmqg==",
        "archive_sha256": "53fa3203ed1a7b46082f1129e0ff909d52772927273b2f5ee62738d7d7d5112f",
        "github_asset_sha256": "sha256:53fa3203ed1a7b46082f1129e0ff909d52772927273b2f5ee62738d7d7d5112f",
        "length": 11306731,
        "asset": "aigent-hive-darwin-x64-0.13.0.tgz"
      },
      {
        "package": "@aigent-hive/linux-arm64",
        "version": "0.13.0",
        "latest": "0.13.0",
        "test": "0.13.0-test.1",
        "dist_integrity": "sha512-b0WMXXiiszh0qLEsXNy4m6YXPin8DB1HrvIuQ1ooy+CPtCnyD6YQ/h++zLpE1xF7EZITXc1HO4/UQe05w1i5Bw==",
        "archive_sha256": "2f5a9b3c58a35a1e548f236d407ef56d00d69aabcece2e16da146629d684d9b8",
        "github_asset_sha256": "sha256:2f5a9b3c58a35a1e548f236d407ef56d00d69aabcece2e16da146629d684d9b8",
        "length": 11436342,
        "asset": "aigent-hive-linux-arm64-0.13.0.tgz"
      },
      {
        "package": "@aigent-hive/linux-x64",
        "version": "0.13.0",
        "latest": "0.13.0",
        "test": "0.13.0-test.1",
        "dist_integrity": "sha512-vwojaq/hN1A269gx3aoLm9t6X+k/53PTtHLzIcwqbkhiq/x743UR7oVcvz0D2jNcFInrvwzfrn2F+b/bLF3tdg==",
        "archive_sha256": "fb339cedfb44fedf698f31c4f1fbe92a47096935b0ada1754725a7da6f6b8d17",
        "github_asset_sha256": "sha256:fb339cedfb44fedf698f31c4f1fbe92a47096935b0ada1754725a7da6f6b8d17",
        "length": 11910916,
        "asset": "aigent-hive-linux-x64-0.13.0.tgz"
      },
      {
        "package": "@aigent-hive/win32-x64",
        "version": "0.13.0",
        "latest": "0.13.0",
        "test": "0.13.0-test.1",
        "dist_integrity": "sha512-xghneyU4BKeMKOnCvXSvtqNSHtPXYU9QsEMTJs6LrlXH+OnIMyDsyMLbZwfEC1CTihL+MXeiB8pFHXAzjqugrA==",
        "archive_sha256": "9a0b454b8953ddc300cfcccd8eeaaf1362bc6290606e0d8a78ce1a5685f85f5f",
        "github_asset_sha256": "sha256:9a0b454b8953ddc300cfcccd8eeaaf1362bc6290606e0d8a78ce1a5685f85f5f",
        "length": 10906144,
        "asset": "aigent-hive-win32-x64-0.13.0.tgz"
      }
    ],
    "limits": [
      "Registry versions, tags, asset bytes and source verified; functional acceptance is a separate result."
    ]
  },
  "subscriber_notice": {
    "step_conclusion": "success",
    "approved_characters": 529,
    "approved_sha256": "e770122d668dc42552c2409afd1924e1e6c0c788fd6dc7c6edf5be67b28f3b81",
    "banner_and_summary_http_accepted": true,
    "reader_delivery_or_read_receipt_not_claimed": true
  },
  "current_user_monitor": {
    "installed_identity": "0.12.1-dev",
    "unchanged": true,
    "enabled": true,
    "monitoring_verified": true,
    "observation_gap_seconds": 1,
    "new_interruption_requests_by_qualification": 0
  },
  "real_user_stable_installation": false,
  "limits": [
    "Public three-OS package acceptance and current-Windows public read-only qualification are distinct from prior live native interruption evidence.",
    "No new 0.13 integrated live halt, real provider refill or simultaneous server/billing cutoff measurement.",
    "Live interruption supports only the qualified Windows 11 x64 Codex 26.1013.4894.0; remote/cloud/other OS unsupported."
  ]
}
```
