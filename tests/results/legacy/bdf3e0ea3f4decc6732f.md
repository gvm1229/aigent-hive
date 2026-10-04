# 과거 시험 결과 보존

```json
{
  "purpose": "실제 Codex 관리 서버의 프로젝트 스킬 발견",
  "original_locator": "tests/work/skill-merge-public-test8/native-project-discovery.json",
  "original_sha256": "77caa0f472d0d6bcadb4d0a0e200f017b19901eb1fb8ffbdcdd95290072c30e5",
  "archived_at": "2026-10-04T17:30:12.787104+00:00",
  "archive_host": "Windows-11-10.0.26300-SP0",
  "result": "passed",
  "source_commit": "not specified in original",
  "attachments": [
    {
      "path": "tests/results/legacy/bdf3e0ea3f4decc6732f/receipt.json",
      "sha256": "a04f40978a8d8bf0874c62d3500529c0e05940f01ad912db692ae1b1743615ba"
    }
  ]
}
```

## 증명 범위와 한계

- Windows·Codex 실제 실행, 각 기록에 파일 보존·발견 범위 명시; Unity 동작·GUI 새로 고침·명시 및 자연어 모델 호출 미증명
- 기존 JSON의 값 보존, 현재 코드 재실행·재검증 근거에서 제외
- 원본에 없는 실행 시각·명령·소스·통과 수치 추정 없음
- 개인 경로·비밀 값 치환으로 원본 전체 바이트와 보존 파일 지문 구분

[보존 JSON](bdf3e0ea3f4decc6732f/receipt.json)
