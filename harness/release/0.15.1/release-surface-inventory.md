# 0.15.1 공개 표면과 별칭 목록

- 기계 검증 정본: [release-surface-inventory.json](release-surface-inventory.json)
- 배포 분류 정본: [release-contract.json](release-contract.json), `bugfix`·버전 `0.15.1` 유지
- 엄격한 이전 버전 판독기와의 호환을 위한 JSON 필드 추가 제외. 이 문서의 별칭 설명과 기존 JSON의 기능 식별자 목록 구분

| 입력 | 정본 동작 | 분류 근거 |
| --- | --- | --- |
| `hive project upgrade` | 기존 프로젝트의 미리보기·적용·검증·복구 | 기존 공개 동작 |
| `hive project refresh` | `hive project upgrade`의 문서화된 완전 별칭 | 같은 인수 판독기·처리기·재감지·권한·출력·복구, 별도 기능 없음 |

- 새 답안 필드: 선택적 project_profile·compact_layout. 기존 답안 호환 읽기와 기존 판별의 오류 보정 자료, 새 모델 실행·권한·스킬·필수 스키마 식별자 추가 없음
- 질문 revision 2: 기존 종류 선택의 의미와 확인 절차 보정. 명시 선택·거부 보존
- 최소 구성: 첫 생성 선택의 저장과 기존 true 유지, 갱신·refresh의 자동 축소 금지. 기존 파일을 지우는 신규 기능 아님
- 종전 “기존 명령 불변” 설명의 철회. 새 입력 철자는 존재하나 새 동작이 없는 별칭이라는 한정
- 증명: 다섯 실행 모드의 별칭 인수 동등성 회귀 시험. 기존 JSON 표면 목록의 추가·삭제 없음, 버전 규칙 완화 없음
