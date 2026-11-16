# 단계 5 검토 — 0.15.3 기억 회수

- 기준일: 2026-11-16, 대상: `feature/overhaul-5-0.15.3-memory`의 `origin/develop` 대비 전체
- 검토자: Fable 5.2 일반 하위 에이전트, 기본 수준. 이 세션에 advisor 하위 에이전트 없음, [위임 지침](../../../.agents/directives/references/agent-delegation.md#stage-review-and-release)의 대체 경로
- 요청에 포함한 유지관리자 원문(2026-11-12): "This code base is yours. What it's for and how it's built are your call, and you'll keep building on it. If you think that editing something big would contribute to the goal, then you are free to point that out and delegate tasks."
- 판정: 1차 REQUEST CHANGES, 수정 뒤 같은 검토자의 2차 재검토 APPROVE WITH FIXES. 2차 지적 반영 뒤 전체 시험으로 확인

## 지적과 처리

| ID | 지적 | 처리 |
| --- | --- | --- |
| B1 | 의미 검색 결합에서 주제 있는 어휘 적중과 주제 없는 벡터 적중의 정체성 불일치로 `fusion citation changed` | 벡터 적중도 주제 조회, 결합 정체성 비교에서 주제·잘림 표시 제외. `fusion_accepts_a_subject_on_one_side_and_keeps_it` |
| S1 | 같은 키·같은 사실을 주제와 함께 다시 기록하면 활성 사실 2개 | 주제를 사실 ID 계산에서 제외. 같은 사실의 새 주제는 `update` 처리(제자리 주제 변경), 주제 없는 재기록은 no-op. `restating_a_fact_with_a_subject_relabels_it_in_place` |
| S2 | 2자 ASCII 접두 일치의 영어 정밀도 손상 | 접두 일치는 한국어 등 비ASCII 2자 이상·ASCII 4자 이상만. 부정 질의 8개 반환 0개 |
| S3 | 주제 묶음 16개 무표시 절단, 보이지 않는 구성원이 자리 차지 | 보이기 판정 뒤 사실 단위 16개, 최신 순, 첫 적중의 `subject_truncated`. 기밀 구성원 비노출 시험 포함 |
| S4 | 주제 추천이 기록 뒤에만 도착, 같은 끝 단어 오탐 | `hive knowledge subjects list` 추가와 Skill의 기록 전 확인, 오탐 규칙 제거(철자만 다른 경우만), 묶음 안 앞선 사실 반영 |
| S5 | `--expansions`의 `;` 구분자가 셸 명령 분리 | `,` 구분, Skill·지침 예시를 따옴표로 |
| S6 | 주제 있는 사실이 생기면 0.15.2 이하가 저장소 전체 거부, 미고지 | 출시 노트 COMPAT-03 고지. 하위 버전 되돌림 미지원 |
| S7 | 복원 기록 크기 한계 초과·검증 전 기록 | 최대 1,000개(기록 약 2MB), 중복 ID를 기록 전 거부 |
| S8 | `replace_active_claims`가 주제 밖 필드 변경 허용 | 주제·revision·지문 외 변경 거부. `replace_active_claims_rejects_changes_beyond_the_subject` |
| S9 | `general/<kind>` 일괄 주제, 미리보기 분량, Skill 경로 없음 | 규칙 없는 사실은 제안 없음(`null`), `--limit`·`--offset`, knowledge-maintain Skill 절차 추가 |
| S10 | OV153-007 결론 과장 | 결론을 "조사·어형 일치로 상승, 주제는 회수율 중립"으로 정정, 부정 질의·CI 하한 시험 추가, 그래프 판정 보류 |
| O1 | 명시 스캔 승격의 주제 누락 | 주제 유지 |
| O2 | 주제의 자격 증명 검사 없음 | 24자 이상 16진 단계와 자격 증명 탐지 거부 |
| O3 | 묶음 거부 시 복구·임시 파일 삭제 지시 없음 | Skill에 1회 재시도·삭제 추가, 낡은 문구 정정 |
| O4 | 주제 ASCII 전용 | Skill에 영어 주제 사용 명시 |
| O5 | 주제 적용의 revision 미증가 | 유지. 복원 바이트 일치 우선, 변경 없음 |
| O6 | 누락 시험 | 기밀 구성원·절단·S1·S8 시험 추가 |

## 2차 재검토 지적과 처리

| ID | 지적 | 처리 |
| --- | --- | --- |
| N1 | 같은 키·같은 사실이면 공개 범위·상태·종류·시각이 달라도 no-op | 그 값이 모두 같을 때만 no-op·주제 변경, 다르면 기존 "explicit supersedes" 충돌. `restatement_is_noop_or_relabel_only_when_the_truth_metadata_matches` |
| S7 잔여 | 바뀐 사실 하나의 복원 전체 차단, 실패한 적용의 복원 기록 잔존 | 바뀐 사실은 건너뛰고 `skipped_changed`로 보고, 적용 실패 시 기록 삭제. 660개 시험에 바뀐 사실 1개 포함 |
| O2 잔여 | 주제 자격 증명 검사가 기록 경로에만 | `validate_subject`로 이동해 `subjects apply` 포함. `subject_validation_rejects_credential_like_segments` |
| N2 | 자동 스캔 승격이 주제 변경 계획에 스캔 메타데이터를 덧씌울 잠재 위험 | `update`는 덧씌우기 제외 |
| N3 | 부정 질의가 짧은 영어 접두 회귀를 잡지 못함 | `fts_prefix_matching_skips_short_ascii_words` |
| O3 잔여 | "턴당 1회"와 재시도 문구 충돌 | "거부된 묶음 재시도 1회 추가"로 정정 |
| N4 | 여러 조각인 첫 사실의 나머지 조각 미반환 | 유지. 사실 1개는 보통 조각 1개, 다른 조각은 다음 조회 대상 |
| 기타 | 512조각 스캔 상한의 절단 미표시, 분리된 collection·대체된 첫 사실 시험 | 유지·후속. 첫 사실은 활성 색인 조각만 대상 |

- P1 미채택 판단: 재검토자 동의

## 큰 제안

- P1 주제를 별도 파일로 분리: 미채택. 0.15.2는 `Claims/`만 읽어 별도 파일은 무시하지만, 색인 최신성 확인을 위해 생성 매니페스트가 그 파일을 덮어야 하고 0.15.2는 알 수 없는 매니페스트 항목을 거부. S1·S8은 사실 ID 분리와 제자리 주제 변경으로 해결, 하위 버전 거부는 COMPAT-03 고지
- P2 그룹 구조 결과 계약(`groups`), P3 검토형 한·영 별칭 표, P4 60개 이상 사실·150개 이상 질의의 상시 무작위 평가, P5 주제 이름 변경·병합·제거: 채택 보류, 다음 기억 단계 계획 검토 시 항목화. [0.15.3 계획 후속 제안](../../plans/active/0.15.3-memory.md#후속-제안)에 기록

## 검증

- 대상 시험(새 시험 포함 11개 묶음·OVX-002 하한)·clippy(Rust 1.97·1.100)·fmt·출시 노트 검사 통과. 전체 시험과 재검토 결과는 이어서 기록
