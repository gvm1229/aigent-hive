# 0.15.3-test.4 공개 시험

- 기준일: 2026-11-20, 실행 장소: local 세션(이 PC의 `gh` 워크플로 실행)
- 대상: develop `8a000ffe`(OV153-017 병합 뒤), 패키지 `0.15.3-test.4`, 출시일 값 `2026-11-20`
- 시험 의도: `docs/test-release-intent.json`, 제품 지문 `sha256:f3bb332dc45f608e1e474229d16b64fa9ce303629f8837325ff8c46231b72db7`, 계획 OV153-017. `check-test-release-gate.py` authorized(기준 test.3 `4d13fca3`)
- 병합 뒤 CI: `963fb36d` 문서 lane 실패(실수로 추적한 로컬 메모 4개가 문서 홈에서 미연결) → 추적 제외 `8a000ffe`. 첫 실행의 Windows 플랫폼 확인이 감시 등록 시간 초과로 실패 → 실패 작업 재실행 success([44589067422](https://github.com/gvm1229/aigent-hive/actions/runs/44589067422))

| 단계 | 결과 |
| --- | --- |
| 후보 | [44598058011](https://github.com/gvm1229/aigent-hive/actions/runs/44598058011) success |
| 첫 게시 | [44603087810](https://github.com/gvm1229/aigent-hive/actions/runs/44603087810) 실패, `@aigent-hive/darwin-arm64` 외 test 태그 전파 지연 |
| 복구 게시 | [44603970413](https://github.com/gvm1229/aigent-hive/actions/runs/44603970413) success, 같은 후보(`recover_published_packages=true`) |
| npm | `aigent-hive`·플랫폼 패키지 5개 모두 test=0.15.3-test.4·latest=0.15.2 |
| 세 OS 수용 | [44604537702](https://github.com/gvm1229/aigent-hive/actions/runs/44604537702) success. Windows x64는 첫 시도에서 `win32-x64` 압축 파일의 npm 배포망 404로 실패, 실패 작업 재실행 success |

## 순서

- test.3과 같이 정식 공개 문서 준비를 후보 커밋 전에 되돌리고 수용 뒤 다시 적용
- 이 PC 확인: [OV153-010](OV153-010.md)의 test.4 절

## 한계

- 세 OS 수용은 GitHub 실행기의 공개 npm 설치 시험. 실제 호스트 확인은 에이전트 대리 수행
