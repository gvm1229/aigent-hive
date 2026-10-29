# 0.14.0 공개 시험 근거

기준일: 2026-10-30. 원래 완료9/15. 실제 npm 반례로 U140-001과 의존 조건U140-002·003·D140-001 재개, 이전 근거 보존.

## test.1 게시와 식별

| 항목 | 확인값 |
| --- | --- |
| 소스·태그 대상 | `b3db82adea8536cfb3cdd22b66ded31bc3ac31bd` |
| 제품·패키지·날짜 | `0.14.0` · `0.14.0-test.1` · `2026-10-30` |
| 정확 소스 CI | [40907960914](https://github.com/gvm1229/aigent-hive/actions/runs/40907960914) 성공 |
| 비공개 후보 | [40917826896](https://github.com/gvm1229/aigent-hive/actions/runs/40917826896) 성공 |
| 최초 게시 | [40921285081](https://github.com/gvm1229/aigent-hive/actions/runs/40921285081) 태그 반영 확인 실패 |
| 동일 후보 복구 | [40922626507](https://github.com/gvm1229/aigent-hive/actions/runs/40922626507) 성공 |
| GitHub 시험판 | [v0.14.0-test.1](https://github.com/gvm1229/aigent-hive/releases/tag/v0.14.0-test.1), prerelease·비공개 초안 아님 |
| npm | 여섯 패키지의 정확 버전·test 태그 일치, latest 모두0.13.1 |

- 후보: macOS arm64/x86_64·Linux arm64/x86_64 musl·Windows x64의 빌드·이전 버전 복구와 직접 설치 파일 구성 통과. 공개 파일의 최종 CLI 수용과 구분
- CI 최초 시도의 Windows→macOS 이전 작업은 실행기 미배정으로 실제 실행0건. 제품 검사는 모두 통과, 해당 작업과 연결 관문의 한정 재실행에서 실제 성공. GitHub의 macOS arm64 공급 부족 안내 보존
- 최초 게시: 여섯 업로드 접수 후 macOS arm64의 test 태그가60초 안 반영되지 않은 실패. 후속 독립 조회에서 Windows 버전404도 관측, 이후 정확 버전·태그 반영 확인. 새 후보나 패키지 덮어쓰기 없이 기존 복구 모드의 전체 지문 대조 후 나머지 게시 완료
- 독립 확인: npm 정확 버전·채널과 GitHub 주석 태그의 실제 커밋 대조. npm 대표 묶음의 SRI와 내려받은 후보 파일 일치. 실제 사용자 설치·정식 main·정식 태그·구독자 전송 미실행

## 공개 npm 파일의 고정 지문

| 패키지 | SRI |
| --- | --- |
| `@aigent-hive/darwin-arm64` | `sha512-eAZP5IGesrRJj1TgEZwyNnPsG1ZSZa9Xrhg9br4M89t3/+QN92qASuYg6MBJ9kyF31SDY9mOkuBsgOYa0iN4Cw==` |
| `@aigent-hive/darwin-x64` | `sha512-RWxF6eQDEei0j+SeO04pmzl9PEOW+r+Ph3k0ly4eUdfQKDuhrvTP8nglLuoQIyRgjoWMiYa7lfxX6SOUCUpKIQ==` |
| `@aigent-hive/linux-arm64` | `sha512-KmWAub7eZp1vFStbQi4hYDlMt4uP8MzxH3b8N10CKAwgbb0MkRqhKxopNnv5b/H5DoA+TErxH39hmsPrps4Fog==` |
| `@aigent-hive/linux-x64` | `sha512-w74gufGbOpw2o8+PAvl1uSdyrMRVWbFJV6jal2yBGvOAeB943z0exVfh0nwbrLiao2ayggQOX4s12xv193LKLg==` |
| `@aigent-hive/win32-x64` | `sha512-ewgxbIUeRvH7wuBw8eOIijz63S1c4FNBA6X7QDneLMsVnGMBr4lpLAhyr5vR2UHfJkUpd4UbKGC43VQ+4pWjiQ==` |
| `aigent-hive` | `sha512-CeB+k+Eq4NkopzCFDMj4R173ehl8xov5brC76/Hk3Nvmn/6zS4uHtdME7C9NpiH7/mjq6lVRMRng2LRmj09oFA==` |

## 첫 공개 파일 수용의 실패와 다음 범위

- [40923321013](https://github.com/gvm1229/aigent-hive/actions/runs/40923321013): Windows x64·Linux x64·macOS arm64 모두 최초 원본 설치 전에 실패. 새 CI checkout에 없는 `tests/work` 부모 폴더를 준비하지 않은 검증기 오류, npm 설치·Hive 명령 실행 전
- 원시 근거: [Windows](../../tests/results/runs/20261029T173940-502c00ca64aa.md)·[Linux](../../tests/results/runs/20261029T173915-f770eb2c2837.md)·[macOS](../../tests/results/runs/20261029T173923-c8b36bc289ba.md). 소스b3db82ad·실행 중 소스 변경 없음. 제품 실패나 사용자 자료 손상으로 확대 해석 금지
- 최소 변경: 기존 `public-update-ci.py`의 검증된 시험 경로에 부모 폴더 포함 생성. 격리 OS 조건·기존 파일 거부·명시 선택·공개 지문 검증 유지
- 수정한 검증기로 기존 공개test.1의 진단 수행, 최종 수용 근거로 전용 금지. 검증 코드 보정에 따른 다음 공개 번호test.2에서 원래 Q140-002·Q140-003 판정
- 남은 범위: 세 OS의 원본 갱신·보존 제거/재설치·현재 CLI 터미널, Windows 실제 Codex·대기 감시 수용. 실제 데스크톱 대화 중단·구독 관측은 해당 격리 시험의 증명 범위 밖

## 구버전 입력 모드의 진단

- 진단40924970047·소스618c1605: 원본 npm 설치·전역/프로젝트 설정 후 실제 갱신 진입. Windows는 종료0·질문0, POSIX는 본체 교체 뒤 사용자 적용의 실패 반환. [Windows](../../tests/results/runs/20261029T175009-7969b9ce0334.md)·[Linux](../../tests/results/runs/20261029T174942-c448d9042db0.md)·[macOS](../../tests/results/runs/20261029T174947-595e7c054d5f.md) 보존
- 기존 U140 계약: 구버전 `--confirm`은 비대화형 보존, 새0.14.0의 같은 옵션과 구분. 대화형 시험에서 구형 옵션 제거·소프트웨어 갱신 동의 답안 명시, 리다이렉션 시험의 옵션 유지
- POSIX 적용 실패는 원인 미확정. 정상 대화형 입력과 함께 해당 임시 프로세스의 Linux 실행 경로·시험 npm 이동 기록·실패 뒤 읽기 전용 상태 수집. 설치 범위·부모 인증·시간 제한의 제품 조건 변경 전 실제 근거 확인

## 실제 npm 이동 오류의 최소 수정

- 정상 입력 진단40929599444·소스e341385d: [Linux](../../tests/results/runs/20261029T181907-19ee38a642a3.md)의 실행 중 원본이 같은 node_modules 아래 `.aigent-hive-WBINaHJ1`로 이동, 실행 인자의 원래 경로는 유지. npm 이동 기록과 일치. 새 공개 파일 지문은 일치하나 사용자 설정 적용 전 차단. [macOS](../../tests/results/runs/20261029T181921-d2de718ce3f8.md)도 같은 이동과 적용 실패
- 원인: POSIX 연결층의 실제 실행 파일 경로 동등 비교가 npm의 옛 패키지 이동을 별도 설치로 판정. Windows의 정상 npm 질문·답변·적용은 [같은 진단](../../tests/results/runs/20261029T181955-60a635e7948a.md)에서 통과
- 최소 변경: 기존 원본 지문/서명·부모 식별·동일 사용자 루트·터미널·기한 검사 뒤, 현재 npm 설치의 소유 검증·원래 실행 경로·같은 node_modules·고정된 하위 경로·npm 이동 이름을 함께 확인. 다른 prefix·다른 실행 인자·알 수 없는 패키지·링크 탈출은 거부. 직접 설치의 기존 경로 비교 유지
- 가까운 검증: 실제 경로·이동/삭제된 옛 경로·다른 설치·변조된 메타데이터·상충 실행 인자·링크의 제한 검사, 관련 Rust 검증. 최종 실제 npm 갱신은 수정한 공개test.2로 재수용. 생산 코드가 다른 test.1 진단을 수정본의 성공 근거로 전용 금지
- Windows 제거 뒤의 누락 표본은 제거 대상인 runtime 관측 경로에 배치한 시험 오류. 승인된 제거 계약은 실행/관측 자료 제거와 `.hive/history` 보존. 갱신 중 runtime 표본 보존 검사는 유지하고, 제거 뒤 자료 보존 표본은 정본 history에 추가. 제품 제거 조건 변경 없음
- 의존 조건은 원래 계획의 완료 전제에 따른 대기. 기존 새 갱신·출력·제거 근거의 삭제나 독립 결함 판정과 구분
