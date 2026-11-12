# 프로젝트 판별 고정 입력

- 범위: OVX-001의 표시 파일 최소 프로젝트 30종, 제품 설치·빌드 대상 제외
- 정답표: [expected.json](expected.json)의 `expected_kind`, 표시 파일 경로·정확한 UTF-8 내용 `files`, 현재 관측값 `observed_default`·`observed_effective`
- 같은 파일명 사례 구분: React·Vue·Express·Node CLI의 `package.json` 의존성·`bin`, Python·Poetry의 TOML 절, Rust 단일 패키지·작업공간의 TOML 절
- 빈 사례: `id=empty`, `files={}`. Git의 빈 디렉터리 저장 불가에 따라 표시 파일·`.gitkeep` 추가 제외. 측정 때 `tests/work/` 아래 새 빈 디렉터리 생성 후 대상 지정
- 측정: 표의 각 `id` 디렉터리에서 `hive setup --target <case> --user-root <temporary-user> --describe --question project-kind --output json`. `empty`만 위 빈 디렉터리 사용. 제공자 CLI 실행 없음
- 결과 판독: `data.questions[0].question.default`와 `data.questions[0].effective`, 종료 코드 0 확인. 기대 구분은 향후 판별용 정답이며 현재 CLI 지원 약속과 구분
- 원본 내용 검증: 표의 모든 `files` 값과 해당 파일의 UTF-8 바이트 일치, 표 밖 표시 파일 부재 확인
- 근거: [OVX-001](../../../docs/research/overhaul/OVX-001.md)
