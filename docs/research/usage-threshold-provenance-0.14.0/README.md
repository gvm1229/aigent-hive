# 임계값 변경과 미완료 설정의 원인 분리

## 판정

공개0.14.0의 전용 임계값 변경 명령에 제품 결함 확인. 현재 값과 맞는 정상 확인 기록이 있는 가짜 설정에서도 실패. 미완료 설정 기록·개인 선호·Windows 개인 설치의 손상을 전제로 하지 않는 재현.

확인 기록은 값의 출처와 지문을 담는 `PreferenceDecision`. 지문은 값이 바뀌었는지 비교하는 표시. 전용 명령이 숫자만 바꾸고 후보 설정의 확인 지문을 갱신하지 않아, 다시 읽은 설정과 자신이 쓴 후보의 비교 실패.

별도 문제: 확인한 미완료 설정과 다른 값을 일반 설정 CLI로 넣으면 기존 의도 보호 검사에서 거부. 이 거부는 전용 임계값 결함과 독립. 이전 진행 기록을 정리하는 방법은 임시 우회이며 제품 결함의 수정과 구분.

## 환경과 재현

- Windows 11 x64·Codex·PowerShell, 소스 `dd581c30013516a6649e74e3c0d80bf7d46cb278`
- 제품 소스는 `v0.14.0` 태그 `e17b400be19979c421be82acdf6e496139f60173`와 일치
- 실행 파일 SHA256: `7a2b66d135015b822ad546ad2c8d959715ab01cc72e123b07084bf55cdfe8af1`
- 가짜 설정20→25·26, 기존 고정 선택 자료·기존 가짜 Codex 재사용. 실제 사용자 설정·진행 기록·백업·식별자의 복사·시험 입력 사용 0건
- HOME·USERPROFILE·앱·임시·설치·캐시 경로와 명시 user-root 격리. 실제 호스트·감시·설치·제공자 API 호출 제외
- [최종 실행](../../../tests/results/runs/20261104T101442-c92f2ca4d54f.md): 9개 관측·예상 결과 일치, 실행 중 소스 변경 없음. [선별 결과](evidence.json)

```powershell
python docs/research/usage-threshold-provenance-0.14.0/reproduce.py --binary target/debug/hive.exe --work tests/work/usage-threshold-provenance-20261104c
```

재실행은 존재하지 않는 새 `tests/work/` 하위 경로 사용. 실제 기록 실행은 `python scripts/test-artifacts.py run --purpose <목적> --path <정확한 경로> --command` 사용. 개인 설정에 같은 시험 반복 금지.

| 가짜 조건·명령 | 관측 | 인과 판단 |
| --- | --- | --- |
| 확인 기록 없음·미완료 없음, threshold20→25 | 성공·25 저장 | 숫자 범위·일반 파일 접근 문제와 구분 |
| 현재20과 맞는 explicit 기록·미완료 없음, threshold→25 | 종료3·원래20 복원 | 오래된 미완료 기록 없이도 결함 재현 |
| 현재20과 맞는 legacy-unknown 기록·미완료 없음 | 같은 실패·20 복원 | 특정 출처 종류만의 문제와 구분 |
| 유효 기록·현재와 같은20 요청 | 무변경 성공 | 실제 값 변경 경로에서 발생 |
| 유효 기록·미완료 없음, 일반 부분 답안으로25 적용 | 성공·값과 지문 일치 | 일반 설정 경로의 확인 기록 작성은 정상 |
| 앞선 일반 설정 완료 뒤 전용 명령으로26 요청 | 같은 실패·25 복원 | 진행 기록 정리·일반 설정 적용만으로 제품 결함 해결 불가 |
| 가짜 미완료 applying20에 일반 부분 답안25 | 다른 의도 오류·20 보존 | 별도 보호 검사 정상 작동 |
| 같은 미완료 상태에서 전용 threshold→25 | 확인 정보 오류·20 복원 | 전용 명령은 미완료 의도 검사와 별개 경로 |

9개 관측에는 가짜 미완료 기록의 생성 단계 1개 포함. 예상 오류의 재현 성공과 제품 명령의 정상 동작 성공을 구분.

## 정확한 호출 흐름

```text
usage_control::set_threshold
  user_setup::set_operational_usage_threshold
    기존 설정 읽기: 값20, 확인 지문20
    숫자만25로 변경: 확인 지문20 유지
    canonical_config: 그대로 직렬화·게시
    load_operational_config
      parse_and_validate_config_inner
        reconcile_decisions: 지문25·ExternalChange로 정규화
    다시 읽은 설정 != 게시 전 후보
    validation 실패 → 기존 설정·투영 되돌림
```

| 근거 | 함수·줄 | 의미 |
| --- | --- | --- |
| [user_setup.rs](../../../crates/hive-cli/src/user_setup.rs) | `set_operational_usage_threshold` 3609, 숫자 변경3647, 다시 읽기 비교3678–3688, 되돌림3691–3707 | 값과 확인 기록을 함께 바꾸지 않는 경로 |
| [같은 파일](../../../crates/hive-cli/src/user_setup.rs) | `canonical_config` 2637, `parse_and_validate_config_inner` 1887–1927 | 직렬화는 기록 보정 없음, 읽기는 보정 수행 |
| [preferences.rs](../../../crates/hive-core/src/preferences.rs) | `reconcile_decisions` 329–391 | 달라진 값에 ExternalChange·새 지문 적용, 과거 답변 시각·질문 증거 제거 |
| [file_input.rs](../../../crates/hive-cli/src/user_setup/setup_engine/file_input.rs) | `annotate` 267–332 | 일반 설정의 값과 확인 기록 동시 생성 |
| [file_apply.rs](../../../crates/hive-cli/src/user_setup/setup_engine/file_apply.rs) | `require_same_intent` 137–160 | 확인한 기존 작업과 새 의도의 차이 거부. 확인 기록을 빼고 값 자체 비교 |
| [application.rs](../../../crates/hive-cli/src/user_setup/setup_engine/application.rs) · [progress_store.rs](../../../crates/hive-cli/src/user_setup/progress_store.rs) | `finish` 17–31, `record_failure` 437–439 | 적용 실패를 복구 가능 이력에 기록, 원래 applying 진행은 재개용으로 유지 |

`reconcile_decisions`의 불일치 검사를 제거하는 해결책 제외. 잘못된 후보가 게시되기 전에 같은 기록 규칙을 적용하는 수정 필요.

## 이전 진행 기록과의 관계

- 전용 threshold 경로의 미완료 진행 생성·의도 검사 호출 부재. 이번 오류가 이전 applying 기록을 만들었다는 인과 주장 제외
- applying 진행과 recoverable-failure 이력의 동시 존재는 `record_failure`의 보존 방식으로 설명 가능. 현재 값이 같다는 사실만으로 성공 완료·안전한 자동 삭제 추론 금지
- 과거 설정 이전 작업이 처음 실패한 상세 원인은 해당 완료 이력의 오류 원문 부재로 미확인. 이번 확인 정보 결함과 같은 원인이라는 결론 제외
- 사용자 승인을 받은 진행 기록 백업·공식 정리·일반 설정 적용은 운영 복구. 개인 자료와 백업은 소스 밖에 유지. 이 절차의 성공을 설치된 제품 코드 수정으로 표현 금지

## 수정과 버전

[수정 후보](../../plans/backlog/usage-threshold-provenance.md): 숫자 변경 후 기존 공통 확인 기록 정규화를 후보에 적용하고, 값·기록·투영을 함께 게시. 정상 재읽기 검사·되돌림·동시 변경 검사 유지.

[버전 규칙](../../decisions/ADR-0006-version-lifecycle.md)의 호환 오류 수정은 다음 패치 번호. 현재0.14.0에서 이 결함만 고치는 후보는0.14.1. 읽기 전용 추가 옵션은 별도 기능 버전 검토, 공통 연결 수정도 별도 승인 범위. 버전 확정·제품 구현·태그·게시·실제 설치 승인 없음.

이번 실행은 Windows의 진단·우회 경로 증거. Linux·macOS, 수정 후 성공, 모든 설치 형태의 수용은 미실행. 최초 가짜 호스트 부재로 일반 설정 비교가 멈춘 [시험 수단 실패](../../../tests/results/runs/20261104T100842-55708eee289d.md)는 기존 호스트 대역을 사용한 뒤 해결. 개인 설정에서 그 시험을 반복하지 않는 경계.
