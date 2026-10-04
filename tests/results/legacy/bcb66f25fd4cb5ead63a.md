# 과거 시험 결과 보존

```json
{
  "purpose": "test.8 최초 darwin-arm64 공개 vector 수용",
  "original_locator": "tests/work/test8-acceptance-first/korean-public-test-darwin-arm64/aigent-hive/aigent-hive/tests/work/vector-native-188438ee/receipt.json",
  "original_sha256": "0100a069bd517f4035d64a79cd612cbd98c307de727b79acaa29fb4fa3705d23",
  "archived_at": "2026-10-04T17:25:22.427795+00:00",
  "archive_host": "Windows-11-10.0.26300-SP0",
  "result": "failed",
  "source_commit": "06d56360748fd4202fec894bfa92c7ddbfe703df",
  "attachments": [
    {
      "path": "tests/results/legacy/bcb66f25fd4cb5ead63a/receipt.json",
      "sha256": "835e377fe1f0f49a8027014956c4c3ea854026643ad96558ef91df8d1283cb92"
    }
  ]
}
```

## 증명 범위와 한계

- GitHub 공개 수용 37220067876의 첫 실행, 각 JSON에 실제 운영체제와 결과 보존; macOS 검색 실패 포함·진단 성공을 원래 실패 대체 근거로 사용 금지·실제 모델 호출 미증명
- 기존 JSON의 값 보존, 현재 코드 재실행·재검증 근거에서 제외
- 원본에 없는 실행 시각·명령·소스·통과 수치 추정 없음
- 개인 경로·비밀 값 치환으로 원본 전체 바이트와 보존 파일 지문 구분

[보존 JSON](bcb66f25fd4cb5ead63a/receipt.json)
