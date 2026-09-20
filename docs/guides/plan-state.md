# 계획 상태 검사와 생성

## 정본

- `docs/plans/PLAN.md`: 목표 버전과 `Active fragments` 표의 등록 문서만 현재 범위
- 상세 계획: 항목 ID·상태·의존성·완료 근거의 유일한 소유 위치
- 과거 완료 문서·보관 자료·미등록 파일: 현재 집계와 출시 허가에서 제외
- 파서: `scripts/plan_state.py`, 검사·생성 명령: `scripts/check-plan-state.py`

## 상세 계획 형식

```markdown
> Plan version: 0.11.0
> Scope: product

- [ ] [EXAMPLE-001] 구현 기준
  - state: agent-owned
```

- `Plan version`: 상위 계획의 `Product version`과 동일
- `Scope`: `product`, `source`, `release`. 공개 시험의 구현 근거는 `product` 완료 항목만 허용
- ID: 대문자로 시작하는 영문·숫자 접두사와 하이픈·숫자 세 자리
- 체크박스 바로 아래 메타데이터 한 줄 필수, 필드 구분은 `; `
- 필수 필드: `state`. 선택 필드: `depends`, `evidence`, `owner`, `reason`
- `depends`: 쉼표로 구분한 현재 항목 ID, 순환·자기 참조·누락·중복 거부
- `complete`: 체크 표시와 내용 지문이 결합된 Markdown 근거 필수, 의존 항목도 완료 상태
- 대기·차단: `awaiting-user-authority`, `awaiting-external-evidence`, `blocked`; 담당 `owner`와 구체적인 `reason` 필수
- 근거 형식: `repo:<저장소 상대 Markdown 경로>#sha256:<내용 지문>`
- 시험 기록은 `status: passed`와 `exit_code: 0` 확인. 근거의 의미와 충분성은 별도 검토 책임

## 검사와 생성

```sh
python scripts/check-plan-state.py
python scripts/check-plan-state.py --write
```

- 기본 검사: 파일 변경 없음, 잘못된 상태·오래된 생성 결과에 실패 종료
- `--write`: `PLAN.md`·`CURRENT.md`의 `HIVE:PLAN-STATE` 생성 구간만 갱신
- 같은 입력의 두 번째 생성은 변경 0건. 생성 구간 밖 바이트·줄바꿈 보존
- 모든 입력·출력을 검증한 뒤 파일별 교체, 도중 변경은 거부. 여러 파일의 단일 원자성 주장 제외
- 부분 적용 오류는 실패로 보고, 재실행으로 정합화. 별도 영구 상태 파일 생성 없음
- 심볼릭 링크·경로 탈출·누락 파일·8KiB 이상 계획 문서 거부

## 출시 연결

- `check-test-release-gate.py`도 같은 파서 사용, 별도 폴더 전체 검색으로 완료 항목 수집 금지
- 요청 제품 버전은 활성 계획과 동일, 계획·근거의 내용은 해당 후보 커밋에 포함 필수
- 소스 도구·출시 절차 완료, 과거 계획의 체크 표시, 미커밋 완료 변경으로 제품 공개 시험 허가 금지
- 번호 시험판의 기존 제품 변경·버전·이력 검사 유지, 안정판 승인의 대체 근거로 사용 금지
