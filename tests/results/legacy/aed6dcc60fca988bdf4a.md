# 과거 시험 결과 보존

```json
{
  "purpose": "test.8 최초 linux-x64 공개 vector 수용",
  "original_locator": "tests/work/test8-acceptance-first/korean-public-test-linux-x64/aigent-hive/aigent-hive/tests/work/vector-native-1pu741ay/receipt.json",
  "original_sha256": "92ff8411a5c8801f515f7541b55f06295104a4339d917743642bb58491988e48",
  "archived_at": "2026-10-04T17:25:22.603955+00:00",
  "archive_host": "Windows-11-10.0.26300-SP0",
  "result": "passed",
  "source_commit": "06d56360748fd4202fec894bfa92c7ddbfe703df",
  "attachments": [
    {
      "path": "tests/results/legacy/aed6dcc60fca988bdf4a/receipt.json",
      "sha256": "02078b884103e63a3950ab52f86be2b85532549843ff2b74a8a2d0c90c9c0d5f"
    }
  ]
}
```

## 증명 범위와 한계

- GitHub 공개 수용 37220067876의 첫 실행, 각 JSON에 실제 운영체제와 결과 보존; macOS 검색 실패 포함·진단 성공을 원래 실패 대체 근거로 사용 금지·실제 모델 호출 미증명
- 기존 JSON의 값 보존, 현재 코드 재실행·재검증 근거에서 제외
- 원본에 없는 실행 시각·명령·소스·통과 수치 추정 없음
- 개인 경로·비밀 값 치환으로 원본 전체 바이트와 보존 파일 지문 구분

[보존 JSON](aed6dcc60fca988bdf4a/receipt.json)
