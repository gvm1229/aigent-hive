# 재현과 관측 근거

[진단](README.md) · [소스 경로](source-map.md) · [선별 관측·지문](evidence-summary.json)

## 환경과 재현 수단

Windows 11 x64·Codex·PowerShell, 공개0.14.0과 제품 경로가 같은 로컬 빌드. 별도 가짜 사용자 설정과 프로젝트만 사용. 전역 설정은 기존 적합성 시험의 운영 YAML 형식으로 직접 준비한 가짜 입력. 전역 설치·실제 Codex 설정 절차 전체의 수용과 구분. 시험 내부의 호스트 설치 대역은 임시 폴더에 한정.

현재 실행 명령:

```powershell
$env:CARGO_INCREMENTAL='0'
cargo +1.99.0 build --locked -p hive-cli --bin hive
python docs/research/project-local-global-readonly-0.14.0/reproduce.py --binary target/debug/hive.exe --work tests/work/knowledge-boundary-20261104-d
```

재실행은 존재하지 않는 새 `tests/work/` 하위 이름 사용. 기존 폴더 사용 거부. 저장소 지침에 따른 실제 기록 실행은 각 명령 앞에 `python scripts/test-artifacts.py run --purpose <목적> --path <정확한 산출물 경로> --command` 사용.

`reproduce.py`: 외부 호스트 명령 실행 없는 가짜 기능 자료, HOME·USERPROFILE·APPDATA·LOCALAPPDATA·CODEX_HOME·설치·캐시·임시 경로 격리, 빈 PATH, 모든 명령의 명시 user-root. 원문 대화 대신 가짜 정규화 선호만 입력.

각 직렬 명령 전후 전체 가짜 사용자 루트의 파일 내용 SHA256·수정 시각 비교. `ReadDirectoryChangesW`로 생성·삭제·수정·이름 변경도 관측. 64KiB 감시 버퍼, 넘침·감시 오류는 시험 실패. 관측기록의 원문 파일 내용·일회용 승인 값 저장 제외. 병렬 묶음은 해당 묶음 전체 결과로만 귀속, 개별 사건의 실행 순서 추정 금지.

## R1: 새 프로젝트와 저장소 선택

- 가짜 전역 사실 1개 준비 후 A·B 새 설정. 한국어 응답·한국어 위키·위키 활성, `root_knowledge_promotion_categories: []` 수락
- A 설정의 사용자 루트 지속 변경 9개: `projects.yml`, `collections.yml`, `rag-trust.json`, `hive.sqlite3`, `rag-generation.json`, 설정 잠금·진행·수명 기록 4개
- 지식 동작과 직접 관련한 앞의 5개를 단순 등록 메타데이터로 재분류 금지. `projects.yml`도 색인 소스 선택의 권한을 가진 등록부
- 순수 설정 진행·복구 기록은 지식 본문과 구분. 현재 전역 보관, 새 모드에서는 프로젝트 내부 보관 계획
- `knowledge query --target A --text globalbeacon`: 종료5, 인증된0.7 이하 전용 경로로 거부, 사용자 루트 사건 0건
- `knowledge query --target A --user-root EMPTY --text globalbeacon`: 종료5, 전역 운영 설정 부재로 거부. 이 결과만으로 모든 별도 루트 구성의 미지원 주장 제외

## R2: 조회의 최종 무변경과 실제 쓰기의 차이

```text
hive knowledge retrieve --target A --user-root G --query globalbeacon --output json
```

조회 성공·결과1개, 지속 파일 변경 0개. 그러나 `.hive/index/.knowledge.lock`과 임시 잠금의 생성·삭제 및 디렉터리 수정 사건 6종 관측. 최종 지문 일치만으로 엄격한 읽기 전용 주장 불가.

SQLite 부수 파일 `-wal/-shm/-journal`은 이 가짜 실행에서 관측 0건. 해당 파일을 일부러 둔 모든 경우까지 무쓰기 증명으로 확대 금지. 저장소 코드의 메모리 SQLite 사용과 사용자 루트 파일 잠금 쓰기의 구분.

## R3: 매 명령 전 해제한 직렬 시험

`authorize-collection`으로 매번 새 승인 발급 후 `collection detach` 실행. 승인 기한60초 이내·일회용 값, 다음 명령 전에 Detached 확인. `projects.yml`의 A enabled 행 유지.

| 다음 명령 | 종료·결과 | A 재연결 | 사용자 루트 지속 변경 | 증명 한계 |
| --- | --- | --- | ---: | --- |
| session begin/check/update/close 각각 | 각0·성공 | 없음 | 각0 | 프로젝트 세션 파일 변경은 정상 로컬 동작 |
| setup --show --summary | 0·성공 | 없음 | 0 | 상태 조회만 확인 |
| setup --validate | 0·유효 | 없음 | 0 | 잠금 사건6종, 읽기 전용 증거와 구분 |
| knowledge query/retrieve/lint 각각 | 각0·성공 | 없음 | 각0 | 각 잠금 사건6종 |
| knowledge graph status | 0·성공 | 없음 | 0 | 기본 그래프 상태만, 벡터·의미 관계 생성 수용 제외 |
| 새 CLI 프로세스 retrieve | 0·성공 | 없음 | 0 | 실제 Codex 앱 종료·재시작·후크 실행과 구분 |
| index rebuild --user-root G | 0·성공 | 있음 | 4 | 빈 collection에서도 재현 |
| knowledge refresh --user-root G | 0·성공 | 있음 | 4 | 공유 정본 동기화 경로 |
| 같은 답안으로 setup --apply 재적용 | 0·성공 | 없음 | 0 | 변경 없는 재적용의 빠른 경로. 모든 재설정 일반화 제외 |
| 기존 전역 사실과 동일한 remember | 0·noop | 있음 | 4 | 사실 삽입0건이어도 앞선 등록 동기화 수행 |
| 다른 새 프로젝트 C setup --apply | 0·성공 | 있음 | 8 | C 등록과 함께 A도 재연결 |
| 유효 source·잘못된 wiki의 ingest | 2·입력 오류 | 있음 | 4 | 수집 실패보다 공유 준비가 먼저 발생 |
| 유효 가짜 source·wiki ingest | 0·성공 | 있음 | 4 | Wiki·Raw는 A, 색인은 G |

재연결 시 공통 변경: `.hive/config/collections.yml`, `.hive/config/rag-trust.json`, `.hive/index/hive.sqlite3`, `.hive/index/rag-generation.json`. 재구축 명령 앞뒤 네 지문은 [선별 관측](evidence-summary.json)의 `index-rebuild.selected_hashes`에서 확인 가능.

원인: [S10](source-map.md)의 무조건 Attached 대입. 동시 실행이 없는 직렬 재현만으로 해당 경로의 원인 확인. 사용자가 관찰한 실제 사건의 촉발 명령까지 확정한 결과와 구분.

## R4: 정본·추출 사실·검색 공개 범위

- A의 가짜 `reviewed-artifact` 선호 기억 저장: 성공·`insert`. G의 `Claims/<A>/`에 정본 사실 생성, G의 색인·세대·검증 정보 변경
- 같은 입력 반복: `noop`, 지속 변경 0건. 가짜 정정의 `supersedes`·`expected_active_digest` 전달: `supersede`, 이전 사실과 새 사실의 교체 관계 보존
- A에서 `localbeacon` 자동 검색 1건, B에서 0건. 이 시험 범위에서 자동 검색 유출 증거 없음
- 가짜 Wiki 수집 뒤 물리 검사: A 로컬 SQLite 부재, A Wiki 정본 존재. G SQLite의 A 행은 문서1·사실2·조각2, G 정본 Claims 2개
- 결론: 검색 공개 범위의 격리 성공과 전역 물리 저장의 동시 존재. “프로젝트 전용”의 두 의미를 섞지 않는 판정
- 반복된 상속 대화 전체의 정규화 수용은 미실행. 원문 수집 없는 가짜 검토 입력의 중복·명시 정정 기반만 실행

## R5: 통제된 동시 실행과 실패

| 묶음 | 횟수 | 관측 |
| --- | ---: | --- |
| A 해제 후 rebuild + B retrieve | 5 | 두 명령 모두 성공, A 재연결5회. 무관한 collection 보존 |
| A 해제 후 setup status + retrieve | 5 | 모두 성공, A 해제 유지5회 |
| A attach 상태에서 새 detach 승인 후 detach + 새 프로젝트 등록 | 5 | 두 명령 모두 성공, 최종 A 연결5회. 새 collection 모두 보존 |

장벽 없는 묶음의 내부 실행 순서 미증명. 반복 횟수를 모든 경합 안전성의 증거로 사용 금지.

추가로 공개 Rust API에서 결정적 순서 재현:

```text
register A → 이전 projects 목록 읽기 → register B → sync(이전 목록)
projects_rows=2 enabled_b=true collection_b_state=Detached
```

[reconcile-interleaving.rs](reconcile-interleaving.rs)와 [실행 기록](../../../tests/results/runs/20261104T063706-f4031ee151af.md). 새 B 등록이 현재 projects에는 남지만 collections에서는 해제되는 결과. 스냅샷 읽기와 동기화 사이의 순서를 강제한 함수 경계 증거. 실제 CLI의 정확한 선점 시점 재현과 구분. 연결 상태 변경 외의 본문 손실·행 삭제·소비자 사건 인과 확정 없음.

같은 소스의 `hive-wiki` 라이브러리를 연결한 실행 파일의 빌드 기록: [Windows 의존성 연결](../../../tests/results/runs/20261104T063645-67f36078d196.md). 재실행은 해당 `reconcile-interleaving.rs`를 Rust1.99.0·같은 빌드의 rlib·native 의존성으로 연결한 뒤, 존재하지 않는 절대 `tests/work/` 경로 인자 전달.

G의 색인 누락·손상·변경 중 표시를 각각 만든 뒤 retrieve: 각 종료5, 자동 복구·지속 변경 0건. 누락은 감시 사건0종, 손상·변경 중 표시는 잠금 사건 각6종. 시험 입력 조작은 명령 실행 전에 수행하고, 명령 변경과 별도 계산. 파일을 복원한 뒤 정상 전역 검색1건 확인. 실제 강제 종료·전원 손실·모든 중단 지점의 시험 제외.

## R6: 기존 시험과 한계

| 명령·기록 | 실제 결과 | 증명 범위·미증명 범위 |
| --- | --- | --- |
| [소스 빌드](../../../tests/results/runs/20261104T062441-fc199fa3b975.md) | 성공, 제품 소스 변경 없음 | Rust1.99.0·Windows 로컬 빌드. 공개 배포 파일·다른 OS와 구분 |
| [완전한 가짜 CLI 재현](../../../tests/results/runs/20261104T063148-eb2ee686fb8a.md) | 성공, 기록79건 | 상태 준비·승인·본 시험 포함79건. 새 모드의 수용 시험 수와 구분 |
| [기존 hive-wiki --lib](../../../tests/results/runs/20261104T063308-771169c67484.md) | 199통과·1실패·1명시 제외 | Windows 잠금 경합 `os error 5`. 제외는 5만개 초과 비밀 조각 규모 시험. 전체 합격 주장 금지 |
| [잠금 경합 단독 재시험](../../../tests/results/runs/20261104T063408-29c61bfcae39.md) | 1통과 | 앞선 실패의 삭제·원인 해결 근거와 구분. 운영체제 경합의 간헐 실패 가능성 유지 |
| [기존 knowledge::tests](../../../tests/results/runs/20261104T063426-9b6345b0bd82.md) | 28통과 | 인증·공유·조회·실패 복구 경로. 나머지 CLI·역사적 통합 시험은 필터 제외 |
| [공유·승격·세션 CLI](../../../tests/results/runs/20261104T063532-427c17967138.md) | 13통과·1건너뜀 | Windows에서 POSIX 권한 모드 경합 제외. 별도 기록 도구가 결과 색인을 갱신해 source_changed_during_run=true; 제품 수용의 깨끗한 소스 근거로 계산 제외 |
| [격리 CLI 최종 재검증](../../../tests/results/runs/20261104T065215-aa5c3643e0b7.md) | 13통과·1건너뜀 | 같은 POSIX 한계. 실행 중 소스 변경 없음, CLI의 실행 전후 지문 일치. 기존 공유 방식의 회귀 범위만 증명 |

최초 재현 입력의 프로젝트 스킬 형식과 승인 기한 오류 2회, Rust 단독 연결의 native 경로 누락 1회는 시험 수단 오류. 원인 교정 후 새 임시 경로에서 성공, 원래 실패 기록 보존. 제품 결함으로 집계 제외.

문서 검증: 작성 문서·계획·사실·연결의 문체 오류0건, Markdown 링크 오류0건, 기존 출시 집계15개 유지. `current-project` 범위의 검토된 소스 사실 한·영 쌍 기록, Source Wiki260쪽·오류0·경고0·파생 색인 조회 확인. 실제 소비자 지식 수집과 구분.

산출물 보존: 실패 재현·공유 빌드·미커밋 시험 증거의 자동 삭제 제외. 실제 지식·설치 정리와 구분. 이번에 생성한 임시 경로의 상태와 보존 이유는 소스 세션 기록에 유지.

## R7: 실제 동작한 다른 초기화 루트

```powershell
python docs/research/project-local-global-readonly-0.14.0/reproduce.py --binary target/debug/hive.exe --work tests/work/knowledge-boundary-nested-final-20261104 --nested-user-root
```

A 내부 `isolated-root/.hive/config/user-setup.yml`에 가짜 운영 설정 준비. 그 루트를 `--user-root`로 주어 remember·현대 프로젝트 setup·retrieve 성공. [최초 실행](../../../tests/results/runs/20261104T063915-237c90d3717b.md)과 [제공 명령 재실행](../../../tests/results/runs/20261104T064902-4b22d276c212.md)에서 확인.

이 경로의 저장소는 별도 사용자 범위의 읽기·쓰기 저장소. 기존 G와 동시에 조회하는 인자·중앙 쓰기 금지 정책 부재. G를 별도 조회하면 R2의 잠금 쓰기 발생. 별도 루트 지원 확인을 요청 기능의 완성이나 일반 전역 설정·설치 전체 수용으로 바꾸지 않는 경계.
