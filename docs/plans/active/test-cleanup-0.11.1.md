# 시험 산출물 용량과 매일 정리

> Plan version: 0.11.1
> Scope: source

## 목적과 경계

- 사용자 요청: 100GB 초과 원인 제거, 시험 종료 정리 강화, 매일 정리 실행
- 2026-10-04 Windows 실측: 112.217GiB, `target/debug` 64.607GiB·`tests/work` 43.398GiB
- 결과 문서·소스·Git·동결 원본 보존, 미사용 빌드와 합성 시험 복제본만 정확 경로 검토 후 삭제
- 삭제된 중간 파일의 직접 복구 보장 없음, 원래 소스·입력으로 재생성
- 현재 0.11.1 결합 검증 재개와 다음 test.7 유지, 소비자 파일 변경·설치 권한 확대 없음

## 수용 기준

- [x] [TCL-001] 큰 산출물의 근거 보존·미사용 검토·실제 용량 축소
  - state: complete; evidence: repo:tests/results/daily-cleanup-20261004.md#sha256:e1704b3441df95e2658383dcd41dfb58babcc3abb6827bb6109470847915f58c
- [x] [TCL-002] 용량 상한·보존 만료 확인과 안전한 반복 정리 회귀
  - state: complete; evidence: repo:tests/results/daily-cleanup-20261004.md#sha256:e1704b3441df95e2658383dcd41dfb58babcc3abb6827bb6109470847915f58c
- [x] [TCL-003] 매일 한국 시간 09시 정리 등록·실행 명령 검증
  - state: complete; evidence: repo:tests/results/daily-cleanup-20261004.md#sha256:e1704b3441df95e2658383dcd41dfb58babcc3abb6827bb6109470847915f58c

## 실행 순서

1. TCL-001: 기존 `scripts/test_artifacts.py`의 `Manager.inventory/review/cleanup` 재사용. 이전 실행 결과를 Git에 보존한 뒤, `target/debug/incremental`과 큰 과거 합성 자료를 조사. 작은 JSON 근거는 기존 `archive_legacy`로 보존. 현재 사용·72시간 내 재사용이 없는 정확 경로만 종료 검토, 기존 넓은 보존 예약은 종료한 작업의 예약만 해제. 미리보기 뒤 실제 정리와 전후 바이트 기록. 원본 증거 미보존·실행 중·연결 경로는 삭제 제외.
2. TCL-002: 같은 도구에 `daily` 작업과 합계 20GiB 점검 상한 추가. 겹치는 경로의 용량 중복 제외. `daily`는 이미 종료 검토된 항목만 기존 정리 절차로 처리하고 남은 미검토·만료·상한 초과를 실패로 보고. 새 자동 소유권 추정·기간만으로 삭제 금지. `Run.execute`는 기본 `CARGO_INCREMENTAL=0`을 자식에게 전달하고 명시 설정 보존. 정상 종료·취소·실패 상태 구분과 정리 성공 여부 회귀. 소유 파일: `scripts/test_artifacts.py`, `scripts/test_artifacts_checks.py`, 검증 지침과 시험 안내.
3. TCL-003: `docs/guides/test-cleanup.md`에 일일 절차·정확 명령·실패 복구 기록. 현재 대화의 앱 자동화로 매일 09시 실행 등록. 근거 없는 보존 연장 금지; 정리 실패·용량 초과·사용자 판단 필요 시 알림, 변화 없는 반복 알림 제외. 미래 예약 실행 자체는 등록·수동 첫 실행 증거와 구분.

## 검증과 완료

- `python -m unittest tests.conformance.documentation.test_test_artifacts -v`: 합성 폴더에서 실제 삭제·외부 파일 보존·실행 중·지문 변경·연결 파일·미커밋 근거·상한·반복 정리 시험
- 문체·Markdown 링크·계획 집계·Source Wiki 검사와 정확 변경 파일 커밋
- 원시 정리 자료는 `.agents/work`에만 저장, 검토된 작은 결과는 `tests/results/` 보존
- 이전 SGM 제품 검증과 본 소스 도구 검증의 결과 범위 구분

## 실행 결과

- [실제 용량·회귀·일일 실행 결과](../../../tests/results/daily-cleanup-20261004.md)
- 보존 근거 부족 자료 313개는 별도 검토 대상, 미래 예약 실행 성공 주장 제외
