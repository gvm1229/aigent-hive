# 0.11.1 공개 시험판의 자동 정리 수용

## 최종 test.5 수용

- 공개 시험: [`0.11.1-test.5`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.11.1-test.5), 제품 소스 `267b4a4e93df26d965acc96665588f21922bfbc7`
- 제품 지문: `sha256:e589e5162fd486fe68e394737381f0cc6ef9f0296d6f66dee711cb849f6e5fd3`
- [후보 36940738218](https://github.com/gvm1229/aigent-hive/actions/runs/36940738218)·[복구 게시 36942587351](https://github.com/gvm1229/aigent-hive/actions/runs/36942587351)·[세 OS 수용 36943131885](https://github.com/gvm1229/aigent-hive/actions/runs/36943131885) 성공; npm 여섯 test.5·latest 0.11.0·후보 압축 파일 지문 일치 독립 확인
- 수용 소스 `71dca1621294b63b02662d01e22c4e420468172c`; 시험 자료의 잘못된 0.11.0 표시만 정정, 제품 파일 차이 0건
- 실제 실행: Windows x64·Linux musl x64·macOS arm64 공개 실행 파일의 네 자동 정리 조건·지침 복구·협업 지침·한국어 규칙·검색 설정·선택형 벡터 수용 통과; Codex 등록은 모의, 모델 호출 증명 제외
- 실제 Windows Claude 2.1.163의 세 경로 설치·재설치·갱신과 외부 자료 보존: [공개 CLI 근거](claude-user-install-0.11.1.md#공개-test5-추가-수용)
- [승인된 현재 사용자 설치](../../tests/results/runs/20261001T235956-d1ca9e889516.md): Codex·Windows, test.5 설치·검증 통과, 지식 일반 파일 489개·기존 설정 2개·Hive 표시 블록 밖 지침 보존, 소비자 프로젝트 변경 0건
- [현재 사용자 새 서버](../../tests/results/runs/20261002T000208-eae715a45f37.md): 실제 Codex 0.159.0, Hive 스킬 28개·중복 0개·발견 오류 0개; 모델 호출 0건, GUI 선택·명시 호출·자연어 호출 미증명
- 현재 사용자 설정 검증 `hive.user-setup-valid`, 기본 사용량 감지 `hive.usage-native-available` 통과; Codex 완전 재시작과 실제 대화 호출의 사용자 실행 필요

| 공개 실행 운영체제 | SHA-256 |
| --- | --- |
| Windows x64 | `2423ae6a1a8958ec429713989f355fadbbf2facde8e830744e788e2cda438799` |
| Linux musl x64 | `b8c356ff2e28ab6b98a08f37d2e5e29c6eb922444b7435301f8980a463eb4e5c` |
| macOS arm64 | `f368ecb57a081cc46c69b61493248beae40ef59caf81727a62ba73d7c3852f4a` |

## 최초 test.1 수용 근거

### 수용 결과

- 공개 시험판: [`0.11.1-test.1`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.11.1-test.1)
- 제품 소스: `55fba2d67e1da8b53b077390d3d639c4cecd4d40`
- 제품 지문: `sha256:3a6d045a067390e82a2a0053b02c8ec9ef44153022242d765118e72e8c65e11b`
- 수용 절차 소스: `7c2d5e5266e94ddc6816efd0a8bae76ce9d65221`; 제품 소스와 배포 파일 차이 0건
- npm 패키지 6개 모두 `test=0.11.1-test.1`, `latest=0.11.0`; 정확한 버전·GitHub 시험판 태그의 독립 조회 완료
- [수용 제품 등록부](../public-test-product.json), [로컬 전체 검증](skill-delivery-0.11.1.md)

### 공개 실행 근거

| 상태 | 대상·이유 | 실제 실행과 증명 범위 | 미증명 범위 |
| --- | --- | --- | --- |
| 통과 | [후보 36636758756](https://github.com/gvm1229/aigent-hive/actions/runs/36636758756) | 원본 소스의 Windows x64·Linux x64/arm64·macOS x64/arm64 빌드와 패키지 생성 | 실제 Codex 발견·모델 호출 |
| 통과 | [게시 36640613644](https://github.com/gvm1229/aigent-hive/actions/runs/36640613644) | 같은 후보의 npm 6개·GitHub 시험판 게시, 독립 버전·채널 조회 | 안정판 게시·현재 사용자 설치 |
| 통과 | [수용 36641879985](https://github.com/gvm1229/aigent-hive/actions/runs/36641879985) | Windows x64·Linux musl x64·macOS arm64의 실제 공개 실행 파일 설치·갱신·복구 시험; 아래 네 조건 모두 통과 | Linux arm64·macOS x64의 실행 수용, 실제 Codex 서버·선택 목록·모델 호출 |
| 통과 | [CI 36641880016](https://github.com/gvm1229/aigent-hive/actions/runs/36641880016) | 수용 절차 소스의 필수 검사 완료 | 현재 사용자 환경의 설치·새 Codex 세션 |
| 미검증 | 현재 Windows Codex의 스킬 목록·호출 | 실제 사용자 설치의 별도 버전별 승인과 Codex 재시작 필요; 미실행 | 실제 중복 표시 제거·명시 호출·자연어 호출 |

수용의 등록 응답: 모의 Codex 명령 프로그램. 실제 공개 Hive 실행 파일의 등록 확인·파일 변경·복구는 실행, 모델 호출은 제외. 결과의 `registration=simulated-native`, `actual_ai_verified=false` 보존.

### 자동 정리의 네 조건

1. 신규 사용자 설정: 검증된 플러그인으로 기본 스킬 제공, 사용자 중복 사본 생성 제외
2. 사용자 갱신: 기록과 원본이 같은 기존 사본 자동 제거, 재갱신의 불필요한 변경 0건
3. 프로젝트 전용 선택: 플러그인에 없는 선택의 로컬 스킬 유지
4. 프로젝트 갱신: 플러그인 비활성 시 로컬 복원, 재활성 시 중복 정리, 외부 파일 보존

### 공개 실행 파일 지문

| 실행 운영체제 | SHA-256 |
| --- | --- |
| Windows x64 | `0b8111ea7257f5bda51b43b908b14ecdab4843e116271e997e5842f9c4207a70` |
| Linux musl x64 | `7291cbc3f40135c3285d8c9e50a6acaa0f60e40646a4e42d0d0a1dd5507fc69f` |
| macOS arm64 | `53e13749785c28fbb1335745dbe398a7e8f27a350902fb3122c1633d5f071025` |

### 실패와 해결

- 최초 후보 `36636321540`: 이전 버전의 시험 의도 기록으로 중단, 게시 전 거부; 현재 제품 지문 연결 후 후보 통과
- 게시 `36638951978`, `36639745895`: npm 게시 뒤 채널·저장소 전파 지연과 이미 올린 버전 충돌; 같은 후보의 재개 `36640613644` 완료, 버전·채널 독립 재확인
- 최초 수용 `36641128291`: 실제 버전 표시 `0.11.1-test #1`과 절차의 `0.11.1-test.1` 비교 차이; 절차만 수정, 제품 파일 무변경 확인 후 수용 `36641879985` 통과
- [로컬 예행 기록](../../tests/results/runs/20260929T224954-8401014156c1.md): Windows 네 조건 통과, 실행 중 소스 커밋 변경으로 단독 완료 근거에서 제외; 공개 수용의 확정 소스 근거로 대체

### 남은 승인과 확인

- 담당: 유지관리자; `SDP-005`의 실제 사용자 설치·발견 수용
- 필요 권한: 현재 사용자 환경에 정확한 `0.11.1-test.1` 설치와 전역 갱신 적용; AGENTS.md·승인된 계획의 버전별 설치 승인 경계
- 승인 후 절차: 정확한 시험판 설치 → 사용자 갱신과 자동 정리·보존 사유 확인 → 대상 소비자 프로젝트 갱신 → Codex 완전 재시작 → 스킬 선택 목록·명시 호출·자연어 호출 확인
- 실제 목록의 기대 결과: 같은 기본 스킬의 플러그인 항목 하나, 수정본·외부·프로젝트 전용 선택의 보존 항목과 이유
- 전역 갱신의 소비자 프로젝트 일괄 변경 제외; 프로젝트 중복은 해당 프로젝트 갱신에서 자동 정리
- 안정판 게시·보호 브랜치 통합의 별도 승인 필요
