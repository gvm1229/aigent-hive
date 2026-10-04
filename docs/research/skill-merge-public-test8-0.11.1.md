# test.8 공개 수용과 실제 프로젝트 반영

## 배포 연결

- 제품 `0.11.1`, 패키지 `0.11.1-test.8`, 배포 날짜 `2026-10-05`
- 소스 `06d56360748fd4202fec894bfa92c7ddbfe703df`
- 제품 트리 `sha256:c4f357f2abdbc6b2231d9e50a025d1e8a35b22dd17060c0c01f3f0454873c074`
- [필수 CI 37217641485](https://github.com/gvm1229/aigent-hive/actions/runs/37217641485)·[후보 37218215348](https://github.com/gvm1229/aigent-hive/actions/runs/37218215348) 성공
- 최초 게시 `37219209309`: Linux arm64 npm 표시 전파 시간 초과. 여섯 후보 SHA512 일치·시험판 표시 8·안정판 표시 0.11.0 확인 뒤 [공식 복구 37219894233](https://github.com/gvm1229/aigent-hive/actions/runs/37219894233) 성공
- [게시 기록](../../tests/results/legacy/27474ac554f0d40ad70d.md)·[패키지 대조](../../tests/results/legacy/48a5f5c00dad6659ee62.md), 재업로드·새 후보 없음
- [GitHub 시험판](https://github.com/gvm1229/aigent-hive/releases/tag/v0.11.1-test.8)과 주석 태그의 실제 소스 커밋 일치
- Windows 실행 파일 SHA256 `777d76625056f6d86c3eb517c612d1a07cc12550bca2be5281465300bf592556`

## 공개 실행 결과

[공개 수용 37220067876](https://github.com/gvm1229/aigent-hive/actions/runs/37220067876)의 Windows x64·Linux musl x64 최초 실행 성공, macOS arm64 두 번째 실행 성공.
설치·스킬 제공·부속 자료 복원/정리·협업 지침·한국어·선택 검색 기능의 실제 공개 바이너리 검증.
Codex 등록 응답의 모의 부분과 실제 모델 대화의 구분, 모델 호출 증명 제외.

| 환경 | 한국어 | 초기 선택 | 검색 |
| --- | --- | --- | --- |
| Windows x64 | [결과](../../tests/results/legacy/50e46fddc60c2c2b3db4.md) | [결과](../../tests/results/legacy/9561ec46e8a6eaa0b4f5.md) | [결과](../../tests/results/legacy/2be163022cf2a9fc5c8d.md) |
| Linux musl x64 | [결과](../../tests/results/legacy/b66dd94fd1c08258f5a1.md) | [결과](../../tests/results/legacy/b9eeb17776cc7b6c88c5.md) | [결과](../../tests/results/legacy/aed6dcc60fca988bdf4a.md) |
| macOS arm64 | [결과](../../tests/results/legacy/c016c038aa445ace1efd.md) | [결과](../../tests/results/legacy/8a688a81e63e4fe2c88a.md) | [결과](../../tests/results/legacy/729d8f65c8d02896fe39.md) |

- 최초 macOS: Skill 정리 통과 뒤 소스 검색 재구성의 도우미 `unknown` 오류, [실패 보존](../../tests/results/legacy/bcb66f25fd4cb5ead63a.md). 별도 진단의 종료 0·성공 JSON·표준 오류 0바이트는 원래 실패의 대체 증명에서 제외
- [기존 동일 유형](all-public-project-refresh-0.11.0.md#최종-수용과-이력-보존)과 대조 후 실패한 macOS만 새 호스팅 환경·Python 및 의존 파일의 새 설치로 1회 재검증. 제품·검사 바이트 변경 없음, 통과를 원인 규명·수정 근거로 해석하는 주장 제외
- 같은 이름의 첫 산출물 선택 방지: [두 번째 산출물 11309369485](../../tests/results/legacy/ba6d688f7791f59c4be7.md)의 정확한 식별자로 재수집·통과 확인
- 현재 Windows·Codex에서 [실제 Claude 2.1.163 세 경로](../../tests/results/legacy/fa3bf530d1827a49c6b9.md)의 설치·검증·재설치·갱신·보존 통과, 일반·공백·한글 경로·모델 호출 0건
- [수정·전체 로컬 검사](skill-resource-cleanup-0.11.1.md): Windows Rust 976개 통과·4개 수동 조건 제외, Python 925개 통과·44개 운영체제/권한 조건 제외. 제외 항목의 실행 성공 주장 없음

## 승인된 실제 반영

- 2026-10-05 사용자 승인: 제시된 현재 사용자 설치·WProject 결합과 0.11.1 구현 진행. 새 시험판은 같은 두 사용자 파일의 입력·결합 내용 불변, 새 의미 변경 없음
- [설치 보존](../../tests/results/legacy/04f26f4addf44dbf3c06.md): Windows의 인증된 `hive update --channel test --confirm`, 공개 실행 파일 지문 일치·`hive.user-install-valid`, 지식 502개·설정 2개·Hive 표시 밖 지침 보존
- [새 미리보기](../../tests/results/legacy/67aeb0b6785f57b89241.md): 검사 파일 8개 불변, 34개 경로 제안, 승인 지문 `sha256:99cfd024bd177b74259cb71838587b7e614d6d42035ab5e1f640a97aad352287`
- [실제 프로젝트 적용](../../tests/results/legacy/b03096ff7734e7d2e9cc.md): 34개 경로 반영, 두 사용자 파일의 승인 내용 일치, 기존 원본과 같은 플러그인 중복 14개 추가 제거. 사용자 AGENTS·기존 별도 문서 3개 보존, `hive.project-upgrade-current`, 재갱신 변경 0건
- Unity 스크립트 검사·커밋 훅·한국어 표시·프로젝트용 이름 보존, 새 변경 분리 규칙과 자연어 선택 허용 반영. 소비자 Git 커밋·Unity 실행 없음
- [새 실제 Codex 관리 서버](../../tests/results/legacy/bdf3e0ea3f4decc6732f.md): 프로젝트 `ship` 한 개·활성 상태 발견, `initialize`·`skills/list`만 실행, 모델 호출 0건

## 남은 사용자 확인

- `SGM-006` 완료, `DPS-003`은 `awaiting-external-evidence`
- WProject 새 Codex 대화 두 개에서 프로젝트 `ship`의 명시 호출과 자연어 선택 확인. 코드·설정·Git 변경 없는 검사로 제한, 선택한 파일 경로·보존한 Unity 검사·변경 분리 규칙의 실제 응답 필요
- 오래된 대화의 스킬 목록, CLI 파일 검증, 관리 서버 발견을 실제 모델 선택의 대체 근거로 사용 금지
- 0.11.1 안정판·main·Discord의 기존 승인 유지, 실제 프로젝트 호출 증거 전 실행 보류
