# 프로젝트 갱신 스킬 접근성과 실제 적용

> Plan version: 0.11.0
> Scope: product
> 결정: [ADR-0023](../../decisions/ADR-0023-foundation-refactor.md)

## 목적과 경계

- 사용자의 자연어 프로젝트 갱신 요청을 설치된 `project-refresh`로 연결
- 현재 설치 파일 존재 확인, Codex 자동 선택 설정 `false` 확인. 미설치와 구분
- 검증 후 기존 [6단계 계획](../../research/example-project-hive-repair-0.11.0.md)에 따라 예시 프로젝트 적용 승인
- `test.3` 공개·Windows 설치 완료. 기준본·예약·Windows 줄끝 병합 오류 보완, 미공개 test.4·5 후보 취소. 수정은 `0.11.0-test.6` 검증, 안정판·main 변경 제외
- 소스 계획 경로로 기존 검증형 작업 방식 유지. 실행 그래프 활성화 주장 제외

## 수용 기준

- [x] [PRF-001] 갱신 스킬 자연어 선택·설치 투영·승인 경계 개선과 회귀 검증
  - state: complete; evidence: repo:tests/results/runs/20260923T004113-af7672586d6f.md#sha256:2b9571b47e2da03492a51e1f602c5afb258ccc7e69d018c30879de42610e12b2
  - 전역 선택 스킬의 자연어 호출 허용, 프로젝트 사본의 기존 명시 호출 정책 보존
  - 프로젝트 갱신 요청·미리보기 요청과 일반 질문·일반 개발·전역 설치 요청 구분
  - 사용 가능 여부 검사, 사용자 명령 입력 불필요, 미리보기 전용 요청의 무변경 보장
- [x] [PRF-002] 공개 시험판과 설치된 스킬의 실제 호스트 발견·자연어 요청 검증
  - state: complete; depends: PRF-001,PRF-004,PRF-005,PRF-006; evidence: repo:tests/results/runs/20260923T025703-d217e6ef6293.md#sha256:ec72ffdeb09ad84862ffadb0533eeca8d16f235789210e69777d892be0075a73
  - 선택 설치·내용 지문·언어별 설명과 자연어 선택 증거 확인
  - Windows Codex 실제 작업과 결정적 배포 검사를 구분, 재시작 필요 시 정확한 경계 기록
- [x] [PRF-003] 갱신 스킬과 6단계 계획을 통한 실제 예시 프로젝트 수정
  - state: complete; depends: PRF-002; evidence: repo:tests/results/runs/20260923T032437-67076a40fbcb.md#sha256:0056ff0d514d10e2ec670db8d4827ef8996bffc5e8e8835fb7710f4903ed7181
  - 현재 사용자 변경 재확인, 인증된 기준본·미리보기·백업·적용·검사
  - 사용자 지침·외부 블록·앱 작업 보존, 프로젝트 Git 절차와 원래 작업 폴더 반영 확인
- [x] [PRF-004] 정식 0.10.3 생성 결과의 과거 기준본 인증과 갱신 복구
  - state: complete; depends: PRF-001; evidence: repo:tests/results/runs/20260923T012740-52f5a750001e.md#sha256:1b4a5bcf8ae7f7425f9840ba14c0b6b74c814c79de7505ab7f02776cf8320b79
  - 실제 0.10.3 CLI로 생성한 합성 자료의 `available`·`host-native` 조합 재현
  - 버전별 실행 주체 규칙 구분, 이전 기준본 바이트 보존, 위조·잘못된 조합의 무변경 거부 유지
  - 0.10.1 이후 실제 생성된 `.prettierignore`를 과거 기준본 목록에 포함, 불완전한 목록 허용으로 우회 금지
  - 과거 사용량 검사 비활성 문구의 두 실제 생성 형식 재현, 원본 지침 변경 제외
  - 해당 자료의 조사·미리보기·적용·검사와 기존 설치 회귀, 수정된 공개 시험판 검증
- [x] [PRF-005] Hive 지침의 정확한 파일 경로 예약과 전체 스킬 흐름 연결
  - state: complete; depends: PRF-004; evidence: repo:tests/results/runs/20260923T014832-909b7a34c38d.md#sha256:70852f5d1c1f67e0b901a3ca6114c6f5a51d65e7cc081edbbbc1f2ec13251884
  - 기존 경로 검사기로 `.agents/directives/<안전한 이름>.md` 예약 허용, 파일 소유권·쓰기 권한 부여 제외
  - 상위 폴더·외부 설정·잘못된 경로 거부, 기존 예약 충돌·호스트별 스킬 경계 유지
  - 정식 구버전 자료에서 미리보기·정확한 예약·적용·검사·종료, 실제 Codex 재검증
- [x] [PRF-006] Windows 줄끝 차이로 누락되는 Hive 신규 지침 병합 복구
  - state: complete; depends: PRF-005; evidence: repo:tests/results/runs/20260923T021945-a49d732d0555.md#sha256:1516a07000fc4192486194b914b6ccd9b5a4f68b395899c8b3047bedafad40eb
  - 실제 예시처럼 AGENTS만 CRLF인 합성 자료에서 신규 설치 선택 문구 누락 재현
  - Hive 지침에 한정한 LF·CRLF 정렬, 실제 사용자 수정·혼합 줄끝·마지막 개행·표시 블록 밖 바이트 보존
  - 일반 설정·스킬 병합·기준본 정본 검사·병합 한도 유지, 실제 문구와 전체 갱신 흐름 검사

## 순서와 근거

`PRF-001` → `PRF-004` → `PRF-005` → `PRF-006` → `PRF-002` → `PRF-003`. 제품 검증 전 실제 예시 갱신 금지.
