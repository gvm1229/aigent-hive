# 컴퓨터 간 지식 이전

개발 중인 `0.10.0`의 검토용 안내. 공개 안정판 설치 안내와 구분.

## 이전 범위

- 대상: 이식 가능한 Markdown 원문·출처·모음 식별자·접근 범위·억제 기록
- 제외: SQLite·벡터 색인·모델·인증 정보·호스트 설정·기밀 자료
- 비공개 모음: 접근 범위 보존, 새 프로젝트 연결 전 공유 검색 제외
- 파일 전달: 사용자가 선택한 USB·파일 공유 수단
- Skill 구분: 기존 지식 이전 `knowledge-transfer`, 새 저장소 분석 `knowledge-scan`, 검색 색인 정비 `knowledge-maintain`

## 이전 컴퓨터

설치된 사용자 설정에서 루트 자동 해석. 필요한 경우 검증한 경로를 `--user-root`로 명시.
사용자 루트는 `.hive`의 상위 폴더. 현재 작업 폴더를 사용자 루트로 임의 대체 금지.

```text
hive knowledge transfer export --preview --bundle ./knowledge.hivekb --output json
hive knowledge transfer export --apply --bundle ./knowledge.hivekb --output json
```

- 기본 범위: `all-portable`; 미리보기의 포함 수·제외 수·용량 확인
- 내보낸 파일과 응답의 `archive_sha256`을 함께 전달
- 같은 파일이 이미 있으면 동일 바이트는 재사용, 다른 바이트는 기본 거부
- 공간 확인: 묶음뿐 아니라 복원할 원문·FTS·작업 중 임시 파일을 위한 여유 공간 확인
- 제외 자료가 있으면 전체 지식 이전 완료로 표현 금지

## 새 컴퓨터

명령은 PowerShell과 POSIX 셸에서 동일. 공백 경로는 따옴표로 감싸고 실제 파일 위치로 변경.

```text
hive knowledge transfer import --preview --bundle ./knowledge.hivekb --expected-sha256 sha256:<전달받은_지문> --output json
hive knowledge transfer import --apply --bundle ./knowledge.hivekb --expected-sha256 sha256:<전달받은_지문> --preview-digest sha256:<미리보기_지문> --output json
```

1. 파일 무결성·모음·충돌 경로 검토
2. `conflict_paths`가 있으면 현재 자료 유지와 충돌 항목 제외, 또는 취소 선택
3. 제외 선택에만 적용 명령에 `--exclude-conflicts` 추가
4. 파일·대상 원문이 바뀌면 적용 거부, 새 미리보기 검토
5. `transfer.complete=true` 확인과 대표 지식의 FTS 조회

원문 복원 후 벡터 준비 실패는 원문 복원의 취소 사유에서 제외. 기존 Wiki 설정이 꺼져 있으면
FTS 미준비 상태를 별도 표시하고 사용 설정은 보존.

## 선택형 벡터 재생성

새 컴퓨터의 벡터 사용 설정이 `yes`인 경우에만 지금 재생성 여부 질문.
이전 응답의 `transfer.id`·`receipt_digest`로 선택을 정확한 작업에 연결.

```text
hive knowledge transfer status --id <이전_ID> --output json
hive knowledge transfer vector --id <이전_ID> --receipt-digest sha256:<현재_기록_지문> --answer yes --output json
```

- `yes`: 이미 승인된 해당 모음만 즉시 재생성. 완료된 색인은 검증 후 재사용
- `no`: 해당 이전 작업을 `deferred`로 기록, 전역 벡터 사용 설정은 `yes` 유지
- `cancel`: 거절 기록 없이 중단, 원문·FTS 복원 상태 보존
- 반복 가져오기·새 세션: 지연 선택 유지, 같은 질문 반복 금지
- 재생성 미완료: 상태 조회 후 새 기록 지문으로 `yes` 재실행, 같은 사용 의사 재질문 불필요
- 새 모델 설치·모음 활성화·프로젝트 연결: 기존 별도 승인 절차 적용
- 기밀 자료 자동 포함·Python 자동 설치·제공자 API 호출 없음

## 여러 묶음 통합

컴퓨터 A·B 등의 `.hivekb`를 C의 한 사용자 루트에 합칠 때는 `import` 반복 대신
`merge` 사용. 입력 묶음 원본 변경 없음.

```text
hive knowledge transfer merge preview --bundle ./computer-a.hivekb --bundle ./computer-b.hivekb --user-root <C의_사용자_루트> --output json
```

- `exact_duplicate_count`: 같은 바이트 지식의 자동 중복 제거 수
- `conflict_paths`: 같은 위치·식별자의 서로 다른 내용. 전체 적용 전 해결 대상
- `semantic_candidates`: 내용은 같아 보이나 검토가 필요한 지식 묶음
- `conflicts`: 같은 경로의 서로 다른 수정본과 선택 가능한 SHA-256 목록
- `merge_input_digest`: 검토 파일과 적용 요청의 입력 지문
- `merge_preview_digest`: 현재 대상 상태까지 포함한 미리 보기 지문

의미 후보는 활성 host가 조건·수치·날짜·부정 표현·출처를 검토. 확정 동등 후보만
`equivalent`, 나머지는 `separate` 선택. `equivalent` 원본은 활성 검색에서 제외하되
이식 가능한 merge provenance로 보존.

```json
{
  "schema_version": 1,
  "merge_preview_digest": "sha256:<merge_input_digest>",
  "decisions": [
    {
      "action": "equivalent",
      "candidate_id": "sha256:<candidate>",
      "primary_path": ".hive/portable/collections/user-root/Wiki/<page>.md"
    },
    {
      "action": "choose",
      "path": ".hive/portable/collections/user-root/Wiki/<page>.md",
      "selected_sha256": "sha256:<선택한_수정본>"
    }
  ]
}
```

```text
hive knowledge transfer merge review --bundle ./computer-a.hivekb --bundle ./computer-b.hivekb --preview-digest sha256:<merge_input_digest> --review ./merge-review.json --user-root <C의_사용자_루트> --output json
hive knowledge transfer merge apply --bundle ./computer-a.hivekb --bundle ./computer-b.hivekb --preview-digest sha256:<merge_input_digest> --review ./merge-review.json --review-digest sha256:<review_digest> --user-root <C의_사용자_루트> --output json
```

- `review`: 검토 지문·결정 범위 확인. 원문·FTS 변경 없음
- `apply`: 모든 입력·대상·검토 지문 재대조 뒤 한 번 적용. FTS 한 번 생성
- 입력·대상·검토 파일 변경: 적용 거부와 새 미리 보기
- 해결 불가 모순: 사용자에게 충돌 목록을 한 번에 제시. 부분 적용 금지
- 접근 범위가 다른 모음·기밀 자료·새 모델 설치·새 모음 활성화: 자동 통합 범위 제외

## 검증 경계

- 로컬 Windows의 원문·FTS·질문·지연·취소·실제 모델 재생성 검증과 다른 운영체제 수용을 구분
- 교차 운영체제 시험: 생성 측의 동일 `.hivekb`를 전달받아 원문 지문·검색 결과 대조
- 작은 파일 100·1,000·5,000개 시험과 기존 긴 문서·5만 청크 시험을 구분
- 과거 안정판 기준 파일·기존 사용자 수정 자료 보존
