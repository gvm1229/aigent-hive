# `0.10.0` 시험·정식 출시

> Checklist owner: `REL10-*`
> 완료 선행: `SCP10-001–003`, `CON10-*`, `VWF10-*`, `JDG10-*`, `SKM10-*`, `KRG10-001–016`, `VEC10-*`, `VQR10-*`, `KOR10-*`

## Checklist

- [x] [REL10-001] 쉬운 설명 강화 포함 다음 공개 시험 입력·변경 기록 정합화
- [x] [REL10-002] 설치 사용자 설명 지침 구현 뒤 Rust·Python·문서·보안·갱신·복구 전체 검사
- [x] [REL10-003] `0.10.0` stable Skill snapshot 뒤 `test.13` 공개 시험 게시와 npm `latest=0.9.5` 불변 확인
- [x] [REL10-004] `0.10.0` stable Skill snapshot `test.13`의 Windows x64·macOS arm64·Linux musl 수용
- [x] [REL10-005] accepted test exact source의 protected `main` 통합과 stable candidate, 승인 구독자 안내문·버전별 SHA-256 sidecar 대조
- [x] [REL10-006] 같은 product bytes의 stable publication·설치·의존 검사
- [x] [REL10-007] 공개 시험·수용 증거가 갖춰진 뒤 유지보수자의 명시적 `0.10.0` 안정판 승인 수령

## 출시 차단

- 해소: `REL10-001–007` 완료
- 안정판 구독자 안내: `docs/releases/<version>.subscriber.ko.sha256` 승인 지문·보호 환경 `AIGENT_HIVE_SUBSCRIBER_SUMMARY_DIGEST` 동시 대조, 불일치 시 배너·Discord 요약 전송 전 차단

## 구독자 안내 승인 자동화

- 목표: 대화의 문구 승인 뒤 에이전트가 외부 지문 등록, 출시마다 GitHub 수동 설정 제거
- 구현: `register-stable-summary-approval.py`에서 승인 지문·버전·원문·sidecar 검증 뒤 기존 `gh` 인증으로 환경 값 등록
- 검증: 원문과 sidecar 동시 변경 차단, 승인 지문 누락·버전 오류·등록 실패 처리, 실제 승인 지문 등록 확인
- 발송 경계: 지문 갱신·문구 재작성 없음. 문구 변경은 새 승인 필요, 등록 실패는 같은 승인으로 재시도
- 출시 권한·제품 바이트·현재 공개 시험 수용 불변
- 완료 근거: Windows 등록·발송 회귀 21개 통과, 기존 승인 `0.10.0` 지문 실제 등록 성공·항목 존재 확인. 비밀 값 재조회·실제 Discord 전송 검증 제외

## 정식 출시 완료 근거

- 유지보수자 현재 요청의 `0.10.0` 안정판 출시 명시 승인
- `develop → main` PR #47 병합, `main` source `301147fab8252954b29b7393327dfcff18eb8b1`
- stable candidate `33549229092`: 다섯 native target·npm 묶음 통과
- stable publication `33550812035`: 같은 후보 산출물 게시
- npm `latest=0.10.0`, `test=0.10.0-test.13`; GitHub Release `v0.10.0` prerelease 아님
- Windows 별도 임시 경로의 `aigent-hive@0.10.0` 설치와 `AIgent Hive v0.10.0 (released 2026-09-02)` 확인. 증명 범위: Windows 설치. 미증명 범위: 다른 운영체제의 새 설치 재실행

## 현재 공개 시험 근거

- `0.10.0-test.10`: 설명 지침 강화 전 제품의 과거 수용 근거. 현재 stable 승격 근거로 사용 금지
- `0.10.0-test.13`, 제품 소스 `dc9491ca7c6acbab2e67b0d90dcc8cda5d972797`
- 후보 `33545448836`·게시 `33546575448`·최종 공개 수용 `33546986588` 성공
- npm `test=0.10.0-test.13`, `latest=0.9.5`, GitHub prerelease `v0.10.0-test.13`

## 안정판 이후 기준

- 현재 안정판: `0.10.0`; 이후 개발 대상: 새 유지보수자 요청의 활성 제품 버전
- 다음 안정판: 모든 변경의 `develop` 선통합, `develop → main` PR만 사용

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
