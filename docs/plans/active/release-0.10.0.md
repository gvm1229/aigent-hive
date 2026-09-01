# `0.10.0` 시험·정식 출시

> Checklist owner: `REL10-*`
> 완료 선행: `SCP10-001–003`, `CON10-*`, `VWF10-*`, `JDG10-*`, `SKM10-*`, `KRG10-001–016`, `VEC10-*`, `VQR10-*`, `KOR10-*`

## Checklist

- [x] [REL10-001] 쉬운 설명 강화 포함 다음 공개 시험 입력·변경 기록 정합화
- [x] [REL10-002] 설치 사용자 설명 지침 구현 뒤 Rust·Python·문서·보안·갱신·복구 전체 검사
- [ ] [REL10-003] `0.10.0` stable Skill snapshot 뒤 `test.13` 공개 시험 게시와 npm `latest=0.9.5` 불변 확인
- [ ] [REL10-004] `0.10.0` stable Skill snapshot `test.13`의 Windows x64·macOS arm64·Linux musl 수용
- [ ] [REL10-005] accepted test exact source의 protected `main` 통합과 stable candidate, 승인 구독자 안내문·버전별 SHA-256 sidecar 대조
- [ ] [REL10-006] 같은 product bytes의 stable publication·설치·의존 검사
- [ ] [REL10-007] 공개 시험·수용 증거가 갖춰진 뒤 유지보수자의 명시적 `0.10.0` 안정판 승인 수령

## 출시 차단

- `VON10-007`, `REL10-001–004`: 벡터 최초 설정 product bytes의 전체 검사·다음 번호 공개 시험·세 운영체제 공개 수용
- `REL10-005–007`: 유지보수자의 현재 요청 안 `0.10.0` 안정판 명시 승인 전 protected `main`·tag·게시·설치 금지
- 안정판 구독자 안내: `docs/releases/<version>.subscriber.ko.sha256`의 승인 지문과 보호 환경 `AIGENT_HIVE_SUBSCRIBER_SUMMARY_DIGEST` 모두 원문과 일치해야 함. 불일치면 배너·Discord 요약 전송 전 차단

## 구독자 안내 승인 자동화

- 목표: 대화의 문구 승인 뒤 에이전트가 외부 지문 등록, 출시마다 GitHub 수동 설정 제거
- 구현: `register-stable-summary-approval.py`에서 승인 지문·버전·원문·sidecar 검증 뒤 기존 `gh` 인증으로 환경 값 등록
- 검증: 원문과 sidecar 동시 변경 차단, 승인 지문 누락·버전 오류·등록 실패 처리, 실제 승인 지문 등록 확인
- 발송 경계: 지문 갱신·문구 재작성 없음. 문구 변경은 새 승인 필요, 등록 실패는 같은 승인으로 재시도
- 출시 권한·제품 바이트·현재 공개 시험 수용 불변
- 완료 근거: Windows 등록·발송 회귀 21개 통과, 기존 승인 `0.10.0` 지문 실제 등록 성공·항목 존재 확인. 비밀 값 재조회·실제 Discord 전송 검증 제외

## 현재 실행 제외

- `REL10-005–007`: 유지보수자 명시 제외. `0.10.0` 안정판 protected `main` 후보·게시·설치·승인 작업 시작 금지
- `REL10-001–004`: 벡터 최초 설정 안내의 현재 미완료 항목. 과거 공개 시험 근거는 당시 product bytes 한정
- `KTX10-*`: `0.10.0-test.8` 수용 완료. 벡터 최초 설정 안내의 전체 완료 근거로 대체 금지

## 현재 공개 시험 근거

- `0.10.0-test.10`: 설명 지침 강화 전 제품의 과거 수용 근거. 현재 stable 승격 근거로 사용 금지
- `0.10.0-test.11`, 제품 소스 `86f05fd0da3c016d738c7bab6f060ff820948325`
- 후보 `33512899857`·게시 `33514717158`·최종 공개 수용 `33517244245` 성공
- npm `test=0.10.0-test.11`, `latest=0.9.5`, GitHub prerelease `v0.10.0-test.11`

- `0.10.0-test.10`, 소스 `a0cc0a1c0b45a22e70bb93ba92fee744da40c26c`
- 후보 `33408454546`·게시 `33409550563`·공개 설치 수용 `33409940218` 성공
- npm `test=0.10.0-test.10`, `latest=0.9.5` 유지, GitHub prerelease `v0.10.0-test.10`
- Windows x64·macOS arm64·Linux musl x64: 정확한 공개 npm binary의 한국어 core·벡터 실행 환경·벡터 최초 설정 yes·no·고정 범위 안내문 수용

## 비벡터 수정 수용

- `test.5` 후보 `33188952482`: 게시 전 취소. 새 복제본의 시험 경로·Unix 코드 검사·문서 검사 수정 뒤 `test.6`으로 재검증

- `0.10.0-test.4`, 소스 `5cedbed4`: 후보 `33101040467`, 게시 `33102125677`, 공개 설치 수용 `33102491627` 통과
- 실제 실행: Windows x64·macOS arm64·Linux musl x64. 수정 한국어 기능·팩·직접 갱신·복구 검사
- npm 시험 채널 test.4, 안정 채널 `latest=0.9.5` 유지. 벡터 포함 제품의 수용 근거로 재사용 금지
- Windows 전체 Rust·Clippy와 Python 710개 목록 대조: 671 통과·39 운영체제별 건너뜀. 단일 Python 전체 실행 성공이 아닌 수정 재실행 합산 근거

## 이전 공개 시험 근거

- Candidate `32633724977`, publication `32634206001`, `0.10.0-test.1`, Windows 실제 설치·세 운영체제 수용: 당시 승인 범위의 유효 기록
- `0.10.0-test.1`: `KOR10-*` 제외. stable `0.10.0`의 현재 product acceptance 승격 불가

## 2026-08-28 재개방

- 이전 시험 통과 기록은 당시 실행 근거로만 보존
- 활성 팩 미반영·규칙 구조 검사 결손·부정 의미 반전 반례로 현재 완료 판정 취소
- 수정된 소스·새 반례 시험·다음 번호 공개 시험의 근거로 재수용
