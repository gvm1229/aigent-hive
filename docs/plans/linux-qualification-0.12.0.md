# 0.12.0 Linux 격리 검증

- 소유 기준: QLF112-004, Windows 전체 Rust997개 통과 뒤 실행
- 환경: Windows의 Docker Desktop, Linux amd64. 현재 사용자 설치·호스트 설정과 분리
- 고정 기반: `rust@sha256:408fe88047cef61a2087653b0c5255fa51c0f2d6d94ddedd7a2562a9b91a46f6`, 실제 Rust1.97.1·Python3.11.2 확인
- 추가 도구: 격리 이미지의 `python3-venv`, 저장소의 고정 `requirements-conformance.txt`. 최종 이미지 지문·패키지 버전 보존
- 파일 경계: 소스 읽기 전용 연결, 단일 Linux 출력 `tests/work/hive-linux-0120`만 쓰기 가능. 모든 Linux 검사의 빌드·의존 캐시 재사용, 시험별 별도 빌드 생성 제외
- 실행 사용자: 컨테이너1000, 호스트 권한·방화벽·계정 설정 변경 제외

## 순서와 명령

1. 관리 도구로 정확한 출력 폴더 예약, 고정 기반의 도구 이미지 생성. Docker 파일·실행 명령·이미지 지문 기록
2. `cargo test --locked -p hive-core -p hive-cli usage`: 실제 Linux 프로세스에서 소진·창·모음·제어·초기화 검사
3. `cargo test --locked -p hive-wiki -p hive-update`: 관계·원문 보존·갱신·복구, 별도 명시 실행 시험 제외 이유 보존
4. `cargo build --locked -p hive-cli` 뒤 격리 가상 환경의 고정 Python 의존 설치
5. 사용량·의미 관계·설치/갱신의 관련 Python 계약 검사. 실제 호스트 연결·모델 실행·유료 전환과 합성 자료의 차이 기록
6. 보고·실행 기록 커밋 뒤 산출물 검토·정리. 실패 재현 자료는 정확한 이유·72시간 이내 기한으로 보존

## 증명 한계

- Linux 컨테이너의 CLI·파일·명령 동작 증명, Linux 데스크톱의 실제 Codex·Antigravity 수용과 구분
- 진행 중 호스트 중단·서명·macOS 원래 오류의 대체 증명 제외
- 실패 발생 시 같은 입력 무조건 재실행 금지, 해당 파일·명령·종료 상태에서 원인 분리
