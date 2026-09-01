# 과거 시험 결과 보존

```json
{
  "purpose": "벡터 기능의 과거 Windows 합성 시험",
  "original_locator": "tests/work/vector-native-f6f5t3n2/receipt.json",
  "original_sha256": "41b69dbdc52a19838257ca37ab5cce322f5341e9cba4d29dce6a6ef1dd89ceba",
  "archived_at": "2026-08-28T16:44:57.700215+00:00",
  "archive_host": "Windows-11-10.0.26200-SP0",
  "result": "passed",
  "source_commit": "ddbd5908d45882dbddf88580e3587a8a290d91f4",
  "attachments": [
    {
      "path": "tests/results/legacy/6ccb1ca0d6ff6f512e0a/receipt.json",
      "sha256": "a0f5dc5cc5db64f4bbeef9fd5b5b1ee8bb122bcff61edcc730b9d8156cb78518"
    }
  ]
}
```

## 증명 범위와 한계

- 원본 receipt의 실제 실행 범위와 proven/not_proven·limitations 한정. 대규모 성능·현재 소스·다른 운영체제 통과 추론 금지
- 기존 JSON의 값 보존, 현재 코드 재실행·재검증 근거에서 제외
- 원본에 없는 실행 시각·명령·소스·통과 수치 추정 없음
- 개인 경로·비밀 값 치환으로 원본 전체 바이트와 보존 파일 지문 구분

[보존 JSON](6ccb1ca0d6ff6f512e0a/receipt.json)
