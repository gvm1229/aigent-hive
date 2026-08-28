# 선택형 의미 검색 사용

`0.10.0` 개발 브랜치의 기능 안내. 공개 `0.10.0-test.4`에는 벡터 기능이 없으며,
대규모·운영체제 수용 검증은 진행 중. 최종 판정은 [활성 계획](../plans/active/hybrid-vector-search-0.10.0.md) 참조.

## 언제 사용하나

- 정확한 식별자·날짜·수치·인용문: 기본 FTS 검색
  - 예: `DOC-142`, 특정 오류 문구, 설정 항목 이름
- 표현은 다르지만 뜻이 비슷한 자료: 선택형 벡터 결합 검색
  - 예: “지운 자료를 되찾는 방법”으로 “백업 복원 절차” 찾기
  - 예: 한국어 질문으로 관련 영어 문서 찾기
- 실제 연결·영향 관계: 문서의 명시적 연결이나 관계 그래프 확인
  - 벡터에서 가깝다는 이유만으로 인과관계나 사실을 확정하지 않음

기존 `knowledge-recall`이 질문에 맞는 경로를 선택. 새 Skill은 필요하지 않으며,
질문을 답하기 위해 모델을 자동 설치하거나 새 색인을 생성하지 않음.

## 처음 활성화

필수 환경: CPython 3.12 또는 3.13과 SQLite 확장 불러오기 지원. Hive의 Python 설치·교체 없음.
일부 macOS Python 배포본은 확장 미지원. 이 경우 지원되는 별도 Python 실행 파일을 `--python`으로 지정.
미지원 환경에서도 기존 FTS 사용 가능. 제공자 API·API key·상시 서버 사용 없음.
Windows 혼합 CPU에서는 보조 계산만 같은 성능 등급의 코어로 제한. 부모·전역 설정은 불변이며,
사용자가 허용한 코어에 해당 등급이 없거나 혼합 다중 processor group이면 벡터 실행 거부·FTS 유지.
대상은 Windows x64·macOS arm64·Linux x64의 glibc Python. Linux musl CLI도 이 Python과
함께 사용할 수 있지만 Alpine의 musl Python은 현재 대상이 아님. 운영체제별 실제 수용은 별도 확인.

먼저 정확한 범위와 다운로드 내역 확인:

```text
hive knowledge vector preview --user-root <user-root> --target <project-root> --collection user-root --visibility shared --python <absolute-python-executable> --output json
```

미리보기의 패키지·모델·용량·저장 경로를 승인한 뒤, 반환된 동의 지문을 그대로 사용:

```text
hive knowledge vector enable --user-root <user-root> --target <project-root> --collection user-root --visibility shared --python <absolute-python-executable> --consent-digest <consent-digest> --output json
hive knowledge vector rebuild --user-root <user-root> --target <project-root> --collection user-root --visibility shared --max-seconds 30 --output json
```

`complete=false`이면 같은 범위의 `rebuild`를 이어서 실행. 검증된 중간 복사본에서 재개하며,
완료 전에는 새 색인을 활성화하지 않음. 기밀 모음은 매번 별도의 현재 작업 승인이 필요.

## 검색과 결과 확인

```text
hive knowledge retrieve --user-root <user-root> --target <project-root> --scope auto --query "지운 자료를 되찾는 방법" --mode semantic --top-k 5 --byte-budget 16384 --output json
```

`--mode`를 생략하면 기존 FTS. 의미 검색도 같은 프로젝트·공개 범위 권한을 사용하며,
현재 프로젝트의 비공개 자료를 다른 프로젝트로 넓혀 검색하지 않음.

- `search.used`: 실제 사용한 검색 경로. 요청만 했다고 벡터가 사용된 것은 아님
- `search.fallback`: 벡터 미설치·손상·오래된 상태 등으로 FTS를 사용했는지 확인
- 각 결과의 정본 위치·지문·공개 범위: 원문과 접근 범위 확인
- 결합 점수: 이번 질문 안에서 후보를 정렬하는 상대값. 사실일 확률이 아님
- `search.fusion.fts_order_preserved`: 정확한 별칭·식별자·문구 때문에 FTS 순서를 보호했는지 확인
- `fusion_rank`: 최종 순위. 순서 보호가 적용되면 점수 크기 순서와 다를 수 있으므로 점수만으로 재정렬하지 않음

예를 들어 정확한 버전, `자료 00500` 같은 이름과 번호, 원문 문구를 의미 모드로 조회해도 보호 조건에 해당하면
기존 FTS 결과가 뒤로 밀리지 않도록 처리. 문구 일치는 사실의 참·거짓 판정과 별개.

기밀 조회 승인은 해당 질문 1회에만 사용. 조회 승인을 색인 생성 승인으로 재사용하지 않음.
기밀 생성은 `authorize-build`의 현재 작업·실행 예산 계약 적용.

## 갱신·중지·복구

원본 문서를 바꿨다면 기존 FTS를 갱신한 뒤, 필요한 모음의 벡터를 재생성.
외부 편집을 감시하는 상시 프로세스는 없으며 원본 Markdown을 자동 변환하지 않음.

```text
hive knowledge refresh --user-root <user-root> --output json
hive knowledge vector status --user-root <user-root> --target <project-root> --collection user-root --visibility shared --output json
hive knowledge vector disable --user-root <user-root> --target <project-root> --collection user-root --visibility shared --output json
```

`disable`은 벡터 사용 중지이며 원본 지식 삭제가 아님. `rollback`은 현재 정본과 호환되는
이전 정상 벡터 세대로 복구. 원본 자체가 달라졌다면 오래된 색인을 억지로 되살리지 않고 재생성.
새 모델·실행 환경으로 갱신할 때는 새 미리보기와 별도 동의 필요.

## 여러 공유 모음 갱신

같은 실행 환경을 이미 승인한 공유 모음만 명시적으로 묶어서 갱신 가능.
예시는 `user-root`와 `shared-notes`라는 기존 모음 두 개를 선택한 경우:

```text
hive knowledge vector rebuild --user-root <user-root> --target <project-root> --visibility shared --collections '["user-root","shared-notes"]' --max-seconds 60 --workers 12 --output json
```

- 최대 100개를 지정하고 16개씩 처리. 모델·색인 읽기는 구간 안에서 공유, 모음별 저장소·권한은 분리
- `--collection`과 혼용 금지. 소스·비공개·기밀 모음의 묶음 처리 없음
- `scopes`의 완료 모음은 다음 요청에서 제외. 묶음 중 실패가 발생하면 종료 코드도 실패로 반환
- 준비 비용 때문에 계산을 시작하지 못했다면 시간 예산을 늘려 재시도. 원본·FTS 자동 수정 없음

| 모음별 상태 | 의미 |
| --- | --- |
| `complete` | 새 색인 준비 완료 |
| `checkpoint` | 검증된 중간 결과에서 재개 가능 |
| `prepared-not-started` | 작업 파일 준비 가능, 새 계산은 미시작 |
| `not-started` | 해당 구간의 작업 미시작 |
| `failed-or-unpublished` | 실패 기록·`status` 확인 후 재시도 필요 |

`--rebuild-mode fresh`는 기존 벡터 대신 새 계산을 요청하는 모드.
시간 제한 뒤에도 미시작 모음은 `fresh` 유지, 이번 새 계산의 검증된 `checkpoint`만 `resume` 사용.
미시작 모음까지 일괄 `resume`하면 호환되는 기존 벡터 재사용 가능. 전체 재계산의 완료 증거와 구분.

## Hive 소스 지식

소스 저장소에서는 소비자 경로 대신 다음 명령 사용:

```text
hive source-wiki vector status --target <source-root> --language ko --output json
hive source-wiki vector query --target <source-root> --language ko --query "중단된 작업을 복구하는 방법" --output json
```

소스의 설치·재생성도 같은 `source-wiki vector` 경로 사용.
세부 보존·격리·복구 경계는 [구현 계약](../architecture/vector-search.md) 참조.
