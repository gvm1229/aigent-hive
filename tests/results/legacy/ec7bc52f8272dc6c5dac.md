# 과거 시험 결과 보존

```json
{
  "purpose": "지식 이전 5000파일 baseline consume 비교",
  "original_locator": "tests/work/knowledge-benchmark-baseline-5000-consume/receipt.json",
  "original_sha256": "b2260cba99f989147882176f28d48bb045d02ef0dfdf7c6350134f3138f737ce",
  "archived_at": "2026-08-31T06:17:46.516569+00:00",
  "archive_host": "Windows-11-10.0.26200-SP0",
  "result": "passed",
  "source_commit": "not specified in original",
  "attachments": [
    {
      "path": "tests/results/legacy/ec7bc52f8272dc6c5dac/receipt.json",
      "sha256": "26599fd999f953f0b2c98940950c0eb9a8faf1401f07946a92b9df5e021ed027"
    }
  ]
}
```

## 증명 범위와 한계

- Windows x64·같은 release 설정·각 1회 측정. 반복 분포·다른 하드웨어 성능 보장 제외
- 기존 JSON의 값 보존, 현재 코드 재실행·재검증 근거에서 제외
- 원본에 없는 실행 시각·명령·소스·통과 수치 추정 없음
- 개인 경로·비밀 값 치환으로 원본 전체 바이트와 보존 파일 지문 구분

[보존 JSON](ec7bc52f8272dc6c5dac/receipt.json)
