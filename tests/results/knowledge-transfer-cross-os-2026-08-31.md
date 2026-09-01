# 지식 이전 교차 운영체제 수용

- [CI 실행](https://github.com/gvm1229/aigent-hive/actions/runs/33366642438): 전체 필수 검사 성공
- [PR #41](https://github.com/gvm1229/aigent-hive/pull/41): `develop` 병합 완료
- 검증한 기능 소스: `e208b2b701fdd68058fbb7164f6efb0bf0908474`
- 병합 커밋: `e1890c87bec3d13b3ee5525ed7c9d46d56879fed`
- 방식: 생산 작업의 동일 `.hivekb`를 GitHub artifact로 전달, 수신 운영체제에서 원문과 FTS 조회 확인

| 생산 | 수신 | 원문 수 | 원문 지문 | 기본 검색 |
| --- | --- | ---: | --- | --- |
| Windows | macOS | 100 | 모두 일치 | 통과 |
| Windows | Linux | 100 | 모두 일치 | 통과 |
| macOS | Windows | 100 | 모두 일치 | 통과 |

세 수신 결과의 `cross_os=true` 확인. 같은 운영체제 자료를 사용할 경우 검사 실패.
공통 묶음: `sha256:a0464b89584357eb224ea1c3c0f82bd11fcb2b9c99518e261eee715b988c5ec6`.

## 실행 파일·결과 지문

```json
{
  "run_id": 33366642438,
  "source_commit": "e208b2b701fdd68058fbb7164f6efb0bf0908474",
  "results": [
    {
      "producer": "Darwin",
      "consumer": "Windows",
      "artifact": "transfer-results-macos-latest-to-windows-latest",
      "binary_sha256": "1e2eedec0d89fbac7d4946eb4febccb6eba4641f5b1f745ddfba30d86da2c76e",
      "receipt_sha256": "5577d2083776c77d01ab6867a495ed93573738a1647db6016ec5314b2d3faeea",
      "status": "passed"
    },
    {
      "producer": "Windows",
      "consumer": "Darwin",
      "artifact": "transfer-results-windows-latest-to-macos-latest",
      "binary_sha256": "3bade50a9cde23d98a22c0e41b8c2ca2fabc854ba2d24ad105765e840135cdb9",
      "receipt_sha256": "96b3caaa26e092ebdf1ca78aae863e500d3b97b05a8f9680ab2445e5214367ad",
      "status": "passed"
    },
    {
      "producer": "Windows",
      "consumer": "Linux",
      "artifact": "transfer-results-windows-latest-to-ubuntu-latest",
      "binary_sha256": "507cc0bdce459b3d109d0c952351044914cea90428c242f09fa3a739df557c85",
      "receipt_sha256": "97a009c74c01ce2ec8e7e0f6e0cb9903fcdc7213b1a5ad6286f9f641c677f70a",
      "status": "passed"
    }
  ]
}
```

## 증명 범위

- 서로 다른 실제 운영체제의 CLI와 파일시스템에서 같은 파일 전달·원문 복원·FTS 조회 검증
- 합성 원문 100개·한글·공백 사용자 루트. 실제 사용자 기기의 모든 설정·권한·디스크 상태 보장 제외
- Windows의 1,000·5,000파일 비교와 실제 모델 재생성은 [별도 실측](knowledge-transfer-2026-08-31.md) 근거
- 공개 npm 설치 시험은 별도 번호 시험판 수용 대상. 안정판 게시·설치 없음
