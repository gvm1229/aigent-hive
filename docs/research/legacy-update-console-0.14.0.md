# 0.14.0 첫 갱신의 콘솔 연결 조사

## 범위와 판정

- 조사일: 2026-10-23. 대상: 수정하지 않은 공개0.13.1의 새 실행 파일 호출, Windows 11 x64·Codex 도구의 ConPTY 터미널
- 목적: 새 질문의 입력 통로와 옛 부모 식별 방법 확인. 제품 구현과 공개 첫 갱신 수용은 별도 단계
- 판정: 콘솔 전용 입출력의 공식 API 확인. 실행 중인 프로그램의 경로만으로 교체 전 파일을 식별하는 방법은 근거 부족
- 실제 사용자 설치·설정·감시·모델 호출 변경0건. 격리된 공개 원본에서 갱신 거부, 설치 실행0건

## 저장소 관측

- `v0.13.1:crates/hive-cli/src/update_activation.rs::run`: `--confirm` 없는 갱신은 표준 입력과 안내 출력의 터미널 확인 이후에만 진행
- 같은 파일의 `run_user_projection`: 새 실행 파일에 사용자 범위 apply/validate 호출. 부모600초 제한과 JSON 결과 판독
- `v0.13.1:crates/hive-cli/src/usage.rs`: 하위 입력null·출력pipe, Windows CREATE_NO_WINDOW. 단순 입력 상속만으로 첫 갱신 질문 제공 불가
- `packaging/npm/bin/hive.cjs`: 원래 터미널을 상속한 실제 Hive 실행 파일 호출. 새 프로그램 설치 뒤 옛 부모의 실행 경로와 같은 이름에 새 파일이 생길 가능성

## 공식 규약

각 자료의 접근일은2026-10-23. Win32 규약의 유일한 관리 주체인 Microsoft 자료 사용. 다른 독립 구현의 동작 증거와 구분.

| 자료·갱신일 | 적용되는 사실과 한계 |
| --- | --- |
| [AttachConsole](https://learn.microsoft.com/en-us/windows/console/attachconsole),2021-08-26 | 지정한 프로세스의 콘솔 연결. 한 프로세스의 동시 콘솔은 하나, 부모의 실행 신원 검증 기능은 별도 |
| [Console Handles](https://learn.microsoft.com/en-us/windows/console/console-handles),2021-12-30 | 표준 입출력이 다른 곳으로 연결된 경우에도 CONIN$/CONOUT$로 콘솔 전용 통로 사용 가능 |
| [GetStdHandle](https://learn.microsoft.com/en-us/windows/console/getstdhandle),2021-12-30 | 콘솔 연결 시 표준 핸들 교체 조건 존재. JSON 파이프와 질문 통로를 명시적으로 분리할 필요 |
| [SetConsoleCtrlHandler](https://learn.microsoft.com/en-us/windows/console/setconsolectrlhandler),2021-12-30 | 콘솔 연결 변경 시 제어 처리기 초기화. Ctrl-C·창 닫기에서 이전 답변 저장과 중단 처리가 별도 검증 대상 |
| [QueryFullProcessImageNameW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew),2023-02-09 | 프로세스 실행 이미지의 전체 이름 조회. 파일 교체 전 바이트의 지문 인증을 보장하는 규약은 없음 |
| [GetMappedFileNameW](https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getmappedfilenamew),2025-08-15 | 지정 주소에 연결된 파일 이름 조회. 실행 중 폴더 이름 변경 이후의 파일 신원 증명과 구분 |
| [PE Format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format),접근 시 제공본 | Windows 실행 파일의 영역·주소 보정·가져오기 표 구조. 실제 실행 주소의 차이를 보정한 비교에 사용 |
| [ReadProcessMemory](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-readprocessmemory),2022-05-14 | 권한이 있는 프로세스의 지정 영역 읽기. 이번 시험은 직접 시작한 격리 Hive의 공개 실행 파일 영역에만 한정 |

## 실제 Windows 시험

- 공개 원본: `@aigent-hive/win32-x64@0.13.1`, npm 원본 압축 파일의 SHA512와 registry integrity 일치
- 실행 파일 SHA256: `e34e5b7486feb6adc52f5b0ca59c01da303a14af2ecd397b1c8e60c9ecab7824`
- [준비 기록](../../tests/results/runs/20261023T140747-8ce7b8075fa7.md)·[터미널 시험 기록](../../tests/results/runs/20261023T140802-6e93446ab098.md)
- 격리된 사용자 루트에서 원본 `hive update --channel test` 실행. 실제0.13.2-test.2 갱신 확인에서 입력 대기
- 시험용 설치 폴더를 같은 시험 루트의 다른 이름으로 이동. 실제 실행 프로세스의 QueryFullProcessImageNameW와 GetMappedFileNameW 모두 이전 경로 반환
- 거부 답변으로 정상 종료. 격리 사용자 루트의 설치 기록 부재 확인. 새 질문 연결·설정 적용·시간 제한 복구·npm 실제 교체는 미실행

## 실행 이미지 비교의 추가 시험

- [첫 비교](../../tests/results/runs/20261023T141634-f45e51bc2313.md): 같은 공개 원본에서 주소 보정 후 .text·.pdata 일치, .rdata의1407바이트 불일치. 이 결과만으로 전체 읽기 전용 영역 일치 주장 제외
- [가져오기 표 제외 비교](../../tests/results/runs/20261023T141819-7d1d2f3ad181.md): Windows가 실행 준비 때 채우는2016바이트의 가져오기 표를 제외하고, 읽기 전용 주소 보정28742개를 반영. .text20846088바이트·.rdata7682218바이트·.pdata788532바이트 모두 공개 원본 지문과 일치
- 같은 시험의 폴더 이동 전후 일치. 변경 가능한 자료·스택·힙·계정·대화 내용은 읽기 대상에서 제외, 공개 기록에는 비교 지문과 건수만 저장
- 적용 후보: 공개 원본에서 고정한 실행 파일 머리말·읽기 전용 영역 지문·주소 보정 위치로 부모 검증. 기존 경로가 가리키는 새 파일 대신 실제 실행 이미지의 알려진 영역 비교
- 범위: 위 두 비교는 Windows 격리 시험의 비교 방법만 증명. 제품용 판독기와 위조 거부는 아래 별도 근거, 부모 종료 경합·콘솔 질문 연결·POSIX 검증은 미실행

## 제품용 판독기의 현재 근거

- `scripts/compile-legacy-update-images.py`와 `harness/setup/legacy-update-images.json`: 공개0.13.1·0.13.2-test.1/2 압축 파일의 SHA512 대조 뒤 고정한 머리말·읽기 전용 영역·주소 보정 위치. [공개 파일 생성 근거](../../tests/results/runs/20261023T143017-bda50b4d7372.md)
- `crates/hive-cli/src/update_bridge/windows-parent.cs`: 프로세스 핸들을 유지한 제한된 읽기와 생성 시각 확인. Rust unsafe 추가 없음
- [초기 실패](../../tests/results/runs/20261023T143627-c809341311c1.md): Windows가 머리말의 ImageBase를 실제 주소로 바꿔 고정 머리말 지문 불일치. 판독 실패에 따른 시험 결과 기록 부재도 보존
- [수정 후 실제 Windows PowerShell5.1 시험](../../tests/results/runs/20261023T143914-380d656efae0.md): 알려진 주소 필드를 원래 값으로 되돌린 비교. 공개0.13.1 식별·다른 두 기준 머리말 불일치·조작한 지문 거부·잘못된 읽기 범위4개 거부, 폴더 이동 이후 같은 결과
- 시험 도구의 `ulong` 별칭은 Windows PowerShell5.1의 `uint64`로 수정. 판독기 자체의 C# `ulong`과 구분
- 한계: 이 판독기는 아직 Rust CLI 호출·직접 부모 관계·갱신 인자·설치 범위·터미널 질문에 미연결. 다른 공개 두 버전의 실제 실행 판독과 POSIX도 미실행. U140-001 전체 완료 근거에서 제외

## 구현에 넘길 조건

1. 사용자 질문과 부모 JSON 결과의 별도 통로 유지. 새 창 생성이나 임의 콘솔 탐색 제외
2. 부모의 프로세스 식별자·생성 시각·갱신 인자·설치 범위를 함께 검증. 이름이 같은 현재 디스크 파일의 버전만으로 옛 부모 판정 금지
3. 위 실행 이미지 비교를 제품용 제한된 검사로 구현하고 위조·읽기 실패·부모 종료 시험 추가. 다른 운영체제는 해당 프로세스의 실행 파일 신원과 터미널 결합을 별도 검증
4. 부모 확인 불가 또는 구버전 `--confirm`의 원래 입력 모드 불명은 설정 보존·질문 보류
5. Linux/macOS의 실제 경로·터미널 동작은 미실행. Windows 결과의 다른 OS 전용 금지
6. U140-001은 미완료 유지. 공개 원본에서 새 질문·답변·적용·기한 복구를 마칠 때까지 출시 관문 통과 표시 금지
