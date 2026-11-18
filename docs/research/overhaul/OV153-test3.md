# 0.15.3-test.3 공개 시험

- 기준일: 2026-11-19, 실행 장소: local 세션(이 PC의 `gh` 워크플로 실행)
- 대상: develop `4d13fca3`(OV153-014·015·016과 단계 검토 반영 병합 뒤), 패키지 `0.15.3-test.3`, 출시일 값 `2026-11-19`
- 시험 의도: `docs/test-release-intent.json`, 제품 지문 `sha256:88f41dc875e2fa0f17b857cae810970ce83cbc5365931e3424af0d197f4cf2da`, 계획 OV153-014·015·016. `check-test-release-gate.py` authorized(기준 test.2 `e82cf758`)
- 병합 뒤 CI: [44406309045](https://github.com/gvm1229/aigent-hive/actions/runs/44406309045) 실패(contract lane의 quick-answer 고정물 지문) → 수정 뒤 `4d13fca3` CI success

| 단계 | 결과 |
| --- | --- |
| 후보 | [44418850332](https://github.com/gvm1229/aigent-hive/actions/runs/44418850332) success |
| 첫 게시 | [44421650427](https://github.com/gvm1229/aigent-hive/actions/runs/44421650427) 실패, `@aigent-hive/linux-arm64`·`win32-x64` test 태그 전파 지연 |
| 복구 게시 | [44423046559](https://github.com/gvm1229/aigent-hive/actions/runs/44423046559) success, 같은 후보(`recover_published_packages=true`) |
| npm | `aigent-hive`·`@aigent-hive/{linux-x64,linux-arm64,darwin-arm64,darwin-x64,win32-x64}` 모두 test=0.15.3-test.3·latest=0.15.2 |
| GitHub prerelease | `v0.15.3-test.3` 시험판·초안 아님·파일 25개, 태그 → `4d13fca3` |
| 세 OS 수용 | [44423555430](https://github.com/gvm1229/aigent-hive/actions/runs/44423555430) success(Windows x64·Linux musl x64·macOS arm64) |

## 순서

- test.2와 같이 정식 공개 문서 준비를 후보 커밋 전에 되돌리고 수용 뒤 다시 적용
- 후보 커밋에서 OV153-014·015·016은 구현 근거로 `complete`, 수용 뒤 `awaiting-external-evidence; owner: local-pc`

## 한계

- 세 OS 수용은 GitHub 실행기의 공개 npm 설치 시험. 실제 호스트 확인은 [OV153-010](OV153-010.md)
