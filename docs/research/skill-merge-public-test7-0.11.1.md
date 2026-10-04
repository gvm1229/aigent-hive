# 0.11.1-test.7 공개 수용과 결합 미리보기

## 공개 파일의 연결

- 제품 `0.11.1`, 패키지 `0.11.1-test.7`, 배포 날짜 `2026-10-04`
- 소스 `41dc5ff6c02dd83a17ef9f21f21623682105355f`
- 제품 트리 `sha256:0780fb57e3ffde4ccefcafdd69cc96b2f484bf51d31545f026110b228432c225`
- 필수 CI [37202887988](https://github.com/gvm1229/aigent-hive/actions/runs/37202887988) 통과, 그 소스로 후보 [37203633109](https://github.com/gvm1229/aigent-hive/actions/runs/37203633109) 생성 성공
- 최초 게시 `37204424655`: npm 채널 전파 대기 시간 초과, 게시 성공 판정 제외
- 여섯 공개 패키지의 SHA512와 후보 파일 일치 확인 뒤 같은 후보의 공식 복구 [37204903795](https://github.com/gvm1229/aigent-hive/actions/runs/37204903795) 성공; 재업로드·새 후보 생성 없음
- 독립 조회: 여섯 `test=0.11.1-test.7`, 여섯 `latest=0.11.0`, [GitHub 시험판](https://github.com/gvm1229/aigent-hive/releases/tag/v0.11.1-test.7)과 실제 태그의 소스 커밋 일치
- [후보 설명](../../tests/results/legacy/673321622c85a0ae2eb7.md), [패키지 대조](../../tests/results/legacy/d45b052e637f84111ae3.md)

## 실행 결과와 한계

공개 수용 [37205090347](https://github.com/gvm1229/aigent-hive/actions/runs/37205090347)의 세 운영체제 작업 모두 성공.
공개 실행 파일의 설치·스킬 제공·지침·한국어·선택 벡터 경로 실행. Codex 관리 응답의 모의 부분과 실제 모델 대화 호출의 구분.

| 실제 실행 환경 | 보존 결과 |
| --- | --- |
| Windows x64 | [한국어](../../tests/results/legacy/34502f13750ead9133e1.md), [초기 선택](../../tests/results/legacy/712cd122823076821291.md), [벡터](../../tests/results/legacy/e87096491f0227f6f5da.md) |
| macOS arm64 | [한국어](../../tests/results/legacy/179247c3a5f669d16b98.md), [초기 선택](../../tests/results/legacy/7e95aff7fff5e1f1da38.md), [벡터](../../tests/results/legacy/139e2f2d5ff523bcc3aa.md) |
| Linux musl x64 | [한국어](../../tests/results/legacy/e17443fdca866fcff71e.md), [초기 선택](../../tests/results/legacy/e15b790b7f0a101fe5ac.md), [벡터](../../tests/results/legacy/2d2e22b05ee47f197749.md) |

- 현재 Windows·Codex의 [실제 Claude 2.1.163 검사](../../tests/results/legacy/cd9ffca497430272c70f.md): 일반·공백·한글의 세 격리 경로에서 설치·검증·재설치·갱신·보존 통과, 모델 호출 0건
- Windows 공개 바이너리 SHA256 `49815e354826b17cbf10dc5565b614a8084e6f14d535362c8409d2f8030d7438`, 로컬 파일과 원격 Windows 수용 파일 일치; [로컬 확인](../../tests/results/legacy/24b8527d8183dae26a17.md)
- 초기 로컬 판별 도구의 npm 버전 문자열 가정 오류: 실제 CLI 표시 `test #7`에 맞춘 정확한 비교로 수정·통과, 제품 바이트 변경 없음
- 현재 사용자 설치와 실제 프로젝트 선택 스킬의 모델 호출은 위 격리 검사로 증명 불가

## WProject 결합 미리보기

- 공개 Windows 파일의 공식 입력 조회·읽기 전용 미리보기 성공; [검토 결과](../../tests/results/legacy/af41de221b52640b0bbc.md)
- 본문만 제시한 첫 요청은 기존 `agents/openai.yaml`의 인증 원본 부재로 거절, 사용자 파일 보호 확인
- 본문과 연결 설정 두 파일을 함께 검토한 요청의 미리보기 성공, 검사한 기존 파일 8개 불변
- 사용자 Unity 스크립트 검사·커밋 훅·한국어 표시·프로젝트용 호출 이름 보존, 새 변경 분리 규칙과 자연어 선택 허용 반영
- 제안된 변경 47개, 스킬의 빠뜨린 새 변경 0개, 사용자 `AGENTS.md` 보존, 원본과 동일한 `project-setup` 로컬 사본 2개 제거 제안
- 현재 미리보기 승인 지문 `sha256:a90bd362aa95684879aabda82700d274114124fdfcc02d91a4293f7affa0a107`
- 현재 사용자 test.7 설치와 실제 프로젝트 반영은 승인 전 미실행; 설치 뒤 두 사용자 파일의 입력·결합 지문과 제안 경로 범위를 다시 확인
- `SGM-004` 공개 수용·정확한 결합 미리보기 완료, `DPS-003` 실제 선택 스킬 호출은 설치·적용 승인과 새 대화 검증 필요
