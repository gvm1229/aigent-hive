# 단계 4 구현 검증과 인계

- 범위: 현재 요청의 OV151-001–007·011 구현 8/8. 전체 계획 8/11, Fable 검토·develop 통합·공개 test.1·정식·설치의 현재 범위 밖 처리
- 실행: Codex·Windows 11 10.0.26300, Rust/cargo 1.97.1·Python 3.13 격리 환경. 새 버전 Linux/macOS 실행·실제 앱의 Skill 발견·공개 설치 수용·모델 준수 증명 제외
- 브랜치: feature/overhaul-4-0.15.1-detection. main·develop의 시작점 498d674d 유지, 원격 전체 CI는 main/develop push 또는 PR 대상. 이번 단계 브랜치의 원격 검사는 브랜치 정책 범위, Fable 검토 뒤의 통합·전체 CI와 구분

## 항목과 커밋

| 항목 | 상태 | 구현 커밋 | 근거 |
| --- | --- | --- | --- |
| OV151-001 버전 시작 | 완료 | e5a7a8be | [승인·전환](OV151-001.md) |
| OV151-002 감지기 | 완료 | 392c4dc0 | [고정 코퍼스](OV151-002.md) |
| OV151-003 네 종류·조합 사실 | 완료 | d1a4f3fe | [호환 읽기](OV151-003.md) |
| OV151-004 2층 입구 | 완료 | 312e9928 | [생성·원문 보존](OV151-004.md) |
| OV151-005 확인 질문 | 완료 | dd64e71e | [수락·거부·취소](OV151-005.md) |
| OV151-006 스킬 복사 축소 | 완료 | 68fa4960 | [최소 구성](OV151-006.md) |
| OV151-007 이전 | 완료 | e750efed | [실제 이전판 18개](OV151-007.md) |
| OV151-011 등록부 빌드 | 완료 | 0d10460d | [P4·20개 버전](OV151-011.md) |

- 시작 main CI: [43330629170](https://github.com/gvm1229/aigent-hive/actions/runs/43330629170) 실패. CURRENT 제품 버전 표기의 릴리스 검사 2건 실패, 단계 브랜치 3bc6c884에서 먼저 수정. [원인·범위](stage-4-main-ci.md), 원래 커밋의 CI 성공 주장 없음
- 후속 보완: 책임별 함수 분리 bce01c54, 0.15.0 호환 기대값 b4bcb83c, Claude 스킬 복원 5bc4a6ae, 배포 분류 319ac812, 현재 지문 abcbb0fe·Copier Claude 목록 37c23a82. 기존 명령·스킬·기능 식별자 불변의 0.15.1 수정판 분류, 버전 검사 완화 없음
- 빈 감지 정보의 불필요한 이전 쓰기 제거 4ab72d93: 근거 없는 옛 답안의 호환 검증 유지, 실제 근거의 coding 전환 유지

## 전후 측정

| 고정 입력 | 이전 | 최종 |
| --- | --- | --- |
| OVX-001 기본 general | 30/30 | 1/30, coding 28·docs 1 |
| OVX-001 세부 구분 | 자동 결과 없음 | 30/30 일치, 미승인 effective=null 30/30 |
| OVX-002 상위 5개 적중 | 22/40·55% | 22/40·55% |
| OVX-002 보류 적중 | 3/10·30% | 3/10·30% |
| OVX-004 Codex 신규 파일 | 0.14.2 기준 81개 | 검증된 플러그인의 기본 구성 14개, Claude 15개 |

- 마지막 명령: `python scripts/test-artifacts.py run --purpose '최종 단계 4 고정 입력 재측정' --path tests/work/ov151-measurements-final --command python scripts/qualify-project-detection.py --hive target/debug/hive.exe --work tests/work/ov151-measurements-final`
- [사례별 측정](OV151-measurements.json)·[18개 공개 이전 입력 결과](OV151-predecessors.json). 고정 입력·질의·개발/보류 분리 불변, 검색 조정 없음. 검색 개선은 단계 5 소속
- 파일 수의 조건: 플러그인이 선택한 내장 스킬 전체 제공·추가 역할/선택 승인 없음. 선택 기능의 추가 파일과 다른 PC의 실제 플러그인 발견 증명 제외. 공개 0.15.0 합성 생성은 99개 파일, 개인 설치 변경 없음

## Rust

- fmt·전체 workspace/all-targets/all-features/locked clippy의 경고 오류화 검사 통과. 마지막 clippy도 같은 범위 재실행 통과
- `python scripts/dev-check.py rust`: CLI 867·통합 12·core 163·projection 45·render 87 통과, CLI 수동 제외 2개. 이후 hive-update 실패 원인은 0.15.0 feature 분류의 복제, 수정판 분류로 보완
- `python scripts/dev-check.py rust -- test -p hive-update --locked`: 60개 통과. `python scripts/dev-check.py rust -- test -p hive-wiki --all-targets --all-features --locked`: 219개 통과·수동 제외 3개
- 합계 1,453개 통과·5개 수동 제외, 최종 관련 실패 0. 한 번의 전체 명령 성공 주장과 구분, 중복 재실행 미가산
- 마지막 빈 감지 보완: 옛 답안의 실제 CLI 검증 1개와 과거 0.9.5 전환 Rust 시험 1개 추가 통과. 위 합계와 중복 가산 없음
- 수동 제외: 동의된 벡터 환경 1·실제 센서 15초 간격 네 관측 1·기밀 조각 50,000개 초과 1·release/SSD 성능 2. 해당 본문 미실행, 실제 센서·규모·SSD 성능 증명 제외
- Unix 전용 외부 연결 경로 시험은 이 Windows 대상에서 컴파일 대상 제외. 문자열 경로 이탈·Windows 경로 검사와 다른 운영체제의 실제 연결 경로 동작 증명 구분

## Python

| 묶음 | 실행 시험 | 최종 통과 | 조건부 제외 |
| --- | ---: | ---: | ---: |
| 문서 | 164 | 162 | 2 |
| 보안 | 103 | 98 | 5 |
| 기능 계약 | 673 | 658 | 15 |
| 통합·릴리스 | 256 | 233 | 23 |
| 합계 | 1,196 | 1,151 | 45 |

- 명령: `python scripts/dev-check.py python`. 문서·보안 통과 뒤 기능 계약에서 종료, 미실행 통합·릴리스의 22개 모듈은 같은 격리 개발 검사 경로에서 별도 실행
- 기능 계약 최초 하위 사례 실패 8건의 소유 시험 6개 재실행 통과. 원문 지문 목록·Copier Claude 목록·이동된 상세 지침 검사 경로·coding 기대값 보완
- 통합·릴리스 최초 실패 15건·오류 2건. 연결 설정·과거 표시 블록·공개 버전 표본·옛 답안 검증의 소유 시험을 재실행, 남은 사용자 메모 위치 기대값 2개도 정확한 바이트·개수·정본 전체 일치로 재검증 통과
- English 전체 검토의 최초 종료 3은 새 실행에서 재현 없음, 원인 수정 주장 제외. 콘솔 진단을 실패 메시지에 보존, 재실행의 양 언어 원문·설정 보존 통과와 최초 원인 미규명 구분
- [제외 식별자·정확한 사유](OV151-skips.json): Windows 연결 생성 권한 부재·POSIX 전용 수단·macOS/Antigravity 조건·미선택 고정 공개 실행 파일 등. 각 본문 미실행, 해당 플랫폼·권한·공개 실행 파일의 수용 증명 제외

## 저장소와 종료 경계

- 기존 과거 원본 수정 0개, v0.15.0 프로젝트/사용자 두 계열 190개 신규 동결. 실제 포함 생성 바이트 교체 전후 동일 SHA-256 0dc1cd9cb5862a7fa77c9fc2d04e3c98f3adcb2de78bc8867e2b7f88dfde2f36
- source Wiki: current-project의 기존 프로젝트 설정 사실 한 쌍 갱신, 나머지 28쌍은 본문 보존·출처 지문만 정합화. 정본과 파생 색인 274쪽의 오류·경고 0 확인
- 보호 요청한 비추적 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외. AI 공동 작성 표기·세션 링크 없음, LF
- 현재 요청의 남은 제품 구현 0개. 외부 근거 소유자: Fable 5.2 단계 전체 검토, 승인·구조 제안 처리 기록 뒤 develop 통합. 공개 test.1·정식·설치는 현재 요청 제외
