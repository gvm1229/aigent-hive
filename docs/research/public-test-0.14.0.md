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
- 수정 `329da0a0`: 기존 설치 소유 검증을 재사용한 POSIX 경로 결합. [Windows 경로 검사5개](../../tests/results/runs/20261029T184837-977e1eeef0a3.md)와 [코드 검사](../../tests/results/runs/20261029T185241-6f2171e2be2a.md) 통과, 실제 수정본의 POSIX npm 흐름은 아직 미검증
- 검증기 `61e9b151`: 관측 표본의 승인 전 보존·승인 후 정리와 정본 이력 보존 구분. Windows 직접 설치는 기존 구형 보조 프로세스의 완료 확인 재사용. [읽기 전용 미리보기·컴파일](../../tests/results/runs/20261029T191257-22829749dba6.md) 통과, 실제 공개 제거/재설치는 test.2에서 확인
- 수정본 CI40940378673의 POSIX 구성 시험은 실제 test 태그가0.14.0-test.1로 바뀐 뒤, 대역 설치기의 고정0.13.2-test.2 입력 검사에서 실패. 제품의 경로 판정 진입 전 실패이며 새 제품 결함으로 확대 해석 금지. 대역은 기존13.2-test.2와14.0 번호 시험만 허용하고 요청 버전·제품·prefix를 대조, 실제 요청을 결과에 기록. 대역 사용 범위와 공개 파일 수용의 구분 유지
- test.2 게시 전 검증기 점검: 원본0.13.1의 직접 갱신은 PATH의 PowerShell 5.1 사용. 초기 설치의 절대 경로와 달리 후속 갱신에 빠져 있던 해당 시스템 폴더만 시험 PATH에 추가. 격리 OS·별도 사용자/설치/캐시 경계 유지, 제품 파일 변경 없음

## test.2 공개와 남은 진단

- 소스6139577b, 후보40950931618, 게시40956196462 성공. 여섯 npm 버전·test 태그와 GitHub 주석 태그의 소스 일치, latest 모두13.1. 초기 게시40954067074의 투명성 기록 중복 오류와 복구40955313809의 태그 반영 지연 보존, 서명 검사 유지·동일 후보 복구
- 공개 전 경로 보정789712e8 포함 소스6a8133e3의 수용40956849777: [Linux](../../tests/results/runs/20261029T212007-9754eebe61a2.md)·[Windows](../../tests/results/runs/20261029T212051-3cf0ee013450.md)는 원본13.1 npm의 질문·적용·자료 보존·제거까지 통과. 재설치의 사용자 설치는 다른 등록 프로젝트가 있는 누락 색인의 명시 복구를 요구하며 중단
- 색인 거부는 기존 `ensure_user_setup_index`의 범위 보호. 시험의 합성 사용자/프로젝트만 대상으로 명시 `hive index rebuild --user-root` 후 사용자 설치를 마무리하고 원문·설정 보존 확인. 거부를 성공으로 처리하거나 자동 외부 모음 읽기를 허용하는 제품 변경 제외
- [macOS](../../tests/results/runs/20261029T212014-2bbdc98689ff.md)는 첫 사용자 갱신 연결 실패 유지. 사설 PTY 자식의 원래 인자·커널 실행 경로·실행 중 서명 판정을 읽기 전용으로 수집하여 추가 원인 확인. 환경 값 수집·앱 제어·부모 인증 완화 없음
- test.2는 최종 수용 전. 수정한 진단 결과는 다음 test.3 수용을 대체하지 않는 경계, 원래9/15 유지

- 진단40962692200·소스10c62613: Windows [0.13.1 npm](../../tests/results/runs/20261029T220338-9302a16cf38a.md)·[0.13.2-test.2 npm](../../tests/results/runs/20261029T220411-edf2a849ed70.md)의 실제 갱신·보존 제거·명시적 색인 복구 후 재설치 통과. 다음 직접 설치의 보조 프로세스는 종료2, 본체 미교체. 예외 문구가 먼저 수집한 화면을 덮는 검증기 오류도 확인
- [Linux 원본0.13.1 흐름](../../tests/results/runs/20261029T220239-72477125bb99.md)은 통과 기록 뒤 npm 연결 파일의 정리 목록 거부로 작업 실패. `89c45512`에서 시험 결과는 유지하고 산출물은 검토 상태로 보존. 로컬 Windows의 기록기46검사 중45통과·Unix 전용1건 미실행, 자동 삭제·입출력 오류 은폐 없음. 전체 공개 수용 성공과 구분
- [macOS 서명 진단](../../tests/results/runs/20261029T220247-3e2056e2e0fa.md): npm 이동 전후 모두 원래 실행 인자·커널 실행 경로 유지, 실행 중 서명 검사0·터미널 그룹 유지. 이동된 실제 파일 경로까지만 관측, 삭제 후 경로와 연결 실패 원인은 미확인. 다음 진단은 같은 사설 자식의 경로 변화를 최대3초 읽기 전용 관측, 제품 인증 변경 전 근거 확보
- [Windows 직접 설치 실패](../../tests/results/runs/20261029T220438-2fb4870205c5.md)는 실행 중 미추적 소스 지문 변경도 기록. 커밋·추적 파일 지문은 동일, 완료 근거 사용 제외. 나머지 네 기록은 실행 중 소스 변경 없음

## 운영체제별 후속 수용과 수정

- 진단40965341233·소스58c6ca36: Linux x64 공개 수용 전체 성공. [원본0.13.1 npm](../../tests/results/runs/20261029T222236-d2b866201d3a.md), [0.13.2-test.2 npm](../../tests/results/runs/20261029T222253-ca5b3cf6753f.md), [직접 설치](../../tests/results/runs/20261029T222309-eaa4c4334d8b.md), [비대화형](../../tests/results/runs/20261029T222328-563b6b61a3f6.md)의 실제 갱신·보존 제거·재설치, 한국어/영어 설정·프로젝트·취소재개·지침·벡터 검증 통과. 격리 Ubuntu24.04·앱 대역, 모델·실제 구독 중단 증명과 구분
- Windows 두 원본 npm 흐름 재통과, [직접 설치](../../tests/results/runs/20261029T222423-9288e737675a.md)는 옛 부모가 실제 설치기의 거부만 반환. 수정한 검증기는 화면 보존·임시 작업 폴더 사용, 세 기록 모두 실행 중 소스 변경 없음
- 공개 Windows 바이너리4dd3b31d의 읽기 전용 [문자 해석 재현](../../tests/results/runs/20261029T223136-9775bcc685b1.md): PowerShell5.1에서437번 문자 방식은 가운데 점을 두 글자로 읽어 버전 거부, UTF-8은 정상. 앞선223043은 시험 환경의 SystemRoot 누락으로 PowerShell 시작 실패, 제품 실행 증거에서 제외. 기존 직접 교체 대역의 선행 UTF-8 설정과 제품 설치기의 누락 차이 확인
- 수정a03c6a2e: 실제 버전 읽기 동안만 UTF-8 사용 후 원래 설정 복원. [공개 바이너리의 수정된 검사](../../tests/results/runs/20261029T223512-967275e97cca.md)와 PowerShell5.1/7의 정상·다른 버전·종료 실패·설정 복원 검사 통과. 전체 공개 직접 갱신은 다음 시험판 수용 대상
- [macOS 삭제 경로 관측](../../tests/results/runs/20261029T222243-9ac48a3d9c25.md): 이동된 경로 다음에 경로 조회0·서명 검사-67065 관측. -67065는 Apple의 `errSecCSNoSuchCode`, 단순 프로세스 존재 확인만으로 종료 전 실행 상태 증명 불가. 기존 격리 구성 시험의 설치 대역에서 옛 폴더 이동·삭제 후 부모가 자식을 기다리는 동안 관측하여 두 상태 분리. 인증 우회나 정식 수용 대체 제외

- CI40968179592의 [npm 폴더 삭제 재현](../../tests/results/runs/20261029T224736-38f4cf1eea89.md): 원본 부모가 설치 자식을 기다리는 S 상태에서도 경로0·서명-67065, 종료 경합과 무관한 실패 확인. 실제 소스a060e3d1·실행 중 변경 없음, 설치/앱 대역의 구성 시험 범위
- [Apple 커널의 csops](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_proc.c)는 보존 실행 파일의 코드 지문과 유효 상태 조회 제공. [Apple의 JavaScript 함수 연결](https://developer.apple.com/library/archive/releasenotes/InterapplicationCommunication/RN-JavaScriptForAutomation/Articles/OSX10-10.html)로 이 읽기만 호출하고 기존 원본 목록과 대조하는 최소 보정. 파일 경로 부재 때의 현재 npm 위치·원래 실행 경로 결합과 기존 수명/소유/터미널 검사 유지, 새 파일 지문으로 옛 부모 인증 대체 금지

- 수정5fe92e90·CI40970272866: [원본0.13.1 npm 폴더 삭제·직접 설치](../../tests/results/runs/20261029T230626-6d749bd562c8.md)와 [0.13.2-test.1 직접 설치](../../tests/results/runs/20261029T230658-8af3f5ffd77a.md)의 실제 질문·답변·설정 완료 통과. 삭제 후 S 상태의 부모에서 커널 조회0·유효 상태와 원본 코드 지문4eded7b8… 일치. 기존 동적 조회 실패를 성공으로 바꾸는 예외 처리 없음
- 같은 격리 macOS arm64의 [운영체제 조회 검사](../../tests/results/runs/20261029T230337-dbee9240abd8.md)·[플랫폼 Rust 검사](../../tests/results/runs/20261029T230752-8411a09c7863.md) 통과. 실제 검사 소스d9c178db·트리0347535f30c9f64b5debe6160ca550ae0484de39는6c16b74a와 동일, 실행 중 소스 변경 없음. 설치·앱 대역의 개발 바이너리 범위이며 공개test.3 수용은 별도
- 같은 소스의 격리 Windows x64: [Windows Terminal](../../tests/results/runs/20261029T230552-e685f79cd966.md), [직접 설치 제거](../../tests/results/runs/20261029T230631-71d499ee883d.md)·[npm 제거](../../tests/results/runs/20261029T230708-eb4df41c196b.md), [공개 Codex CLI와 대기 감시의 보존 제거](../../tests/results/runs/20261029T230744-2ec97bff0a9e.md), [플랫폼 Rust 검사](../../tests/results/runs/20261029T231019-701c20f9cd75.md) 통과. 새 임시 프로필·캐시와 개발 Hive 사용, 실행 중 소스 변경 없음. 모델 호출·실제 구독 관측·대화 중단·공개0.14.0 파일 수용과 구분
- [CI40970272866](https://github.com/gvm1229/aigent-hive/actions/runs/40970272866) 전체 성공, 소스6c16b74a의 PR83 develop 통합 완료. 후속 문서·사실·검증 기록만 추가하며 제품 지문170db926… 유지. 다음test.3 후보는 문서 확인까지 끝난 정확 소스에서 생성, 공개 수용 전 원래9/15 유지

## test.3 공개와 첫 갱신 수용

- 소스1ab3c773·제품 지문170db9262d8e7617f253f36fde097f091d5a2a6c1bd0a60f5652e531695ddc58. 제품 CI40970272866·문서 CI40976504251, 후보40977030789의 다섯 배포 대상과 기존 안정판 복구 통과
- 첫 게시40979719398은 여섯 npm 업로드 후60초 태그 반영 확인 실패. 같은 후보 복구40980557213 성공, 기존 공개 파일 지문 대조 후 재업로드 없이 GitHub 게시. 여섯 정확 버전·test 태그와 주석 태그의 소스1ab 일치, npm/GitHub latest 모두13.1 독립 확인
- [공개test.3](https://github.com/gvm1229/aigent-hive/releases/tag/v0.14.0-test.3)의 묶음 SHA256: `0ac7ec49d542f4196908272be9034559273a9f1772c87a5a534a35a903f75221`

| 실행 환경 | 원본0.13.1 npm | 원본0.13.2-test.2 npm | 원본0.13.1 직접 설치 | 원본0.13.1 비대화형 |
| --- | --- | --- | --- | --- |
| Windows x64 | [004109](../../tests/results/runs/20261030T004109-6c481e61e14c.md) | [004144](../../tests/results/runs/20261030T004144-b0239fd36746.md) | [004218](../../tests/results/runs/20261030T004218-9e7a239f9f55.md) | [004411](../../tests/results/runs/20261030T004411-f06e7b6edfac.md) |
| macOS arm64 | [004047](../../tests/results/runs/20261030T004047-86326b3b4a39.md) | [004121](../../tests/results/runs/20261030T004121-6ff91b94e5b0.md) | [004152](../../tests/results/runs/20261030T004152-0f7dd9003ca2.md) | [004215](../../tests/results/runs/20261030T004215-cdfec9e33de5.md) |
| Linux x64 | [004035](../../tests/results/runs/20261030T004035-3300260b09a2.md) | [004047](../../tests/results/runs/20261030T004047-72f0618ed6a9.md) | [004058](../../tests/results/runs/20261030T004058-903bb3284c43.md) | [004111](../../tests/results/runs/20261030T004111-63cbeeb15858.md) |

- 모두 실제 공개 실행 파일·설치 관리자와 격리 GitHub OS에서 갱신→설정 확인→미리보기/거부/승인 제거→명시적 색인 복구→재설치 통과, 실행 중 소스 변경 없음. 앱은 제한된 대역, 역할·지식·이력은 합성 보존 자료. 모델·실제 구독 관측·개인 설치 증명과 구분
- U140-001–003·D140-001의 공개 첫 연결과 의존 조건 재수용, 원래13/15(86.7%). 전체 공개 수용 Q140-002·003은 아래 취소 검증기 보정 후 test.4에서 판정

## Windows 취소 검증기의 상속 입력 보정

- 수용40981139083의 Linux·macOS 전체 통과. Windows는 위 네 갱신 흐름과 한국어/영어 설정·프로젝트 통과 후 [Ctrl+C 대기 종료23](../../tests/results/runs/20261030T004456-23d9db508920.md). 실패한 Windows만 새 VM에서1회 재실행해 [같은 실패 재현](../../tests/results/runs/20261030T010931-deb01db9d333.md), 통과한 Linux·macOS 실행 시각 유지
- [PowerShell 시작 비교](../../tests/results/runs/20261030T005929-9078b6154c95.md)는 직접/PowerShell 경유 모두 이벤트1·실제 종료. 같은 공개 파일의 [Git Bash 시작](../../tests/results/runs/20261030T011958-fb28105b91d9.md)은 두 경로 모두 이벤트0·종료23, 사용량 질문 대기 재현. 제품 입력·기한 변화 없이 시작 환경만 변경
- [Microsoft의 제어 처리 규약](https://learn.microsoft.com/en-us/windows/console/setconsolectrlhandler): Ctrl+C 무시 여부의 자식 상속과 현재 프로세스만의 복원. 소스 CI는 Python, 공개 검증은 Git Bash에서 시작하는 차이 확인
- 수정4c10b364: 검증기의 상속 무시를 해제한 뒤 보호 콜백·제품 자식 시작, 원래10초 종료·미적용/선택 보존 조건 유지. 실패 시 이벤트 수/화면 보존. [같은 공개test.3의 Git Bash 회귀](../../tests/results/runs/20261030T012901-e32da48ccdce.md)에서 일반 Ctrl+C·강제 무시 상속·EOF와 비대화형 재개3검사 통과. 실제 앱·예약·설치 변경 없음
- 강제 무시 상속 사례를 develop 및 공개 Windows 검사에 연결. 제품 바이트는170db926…로 동일, 원래 공개 검증기 수정 규칙에 따라 test.4로 최종 수용. test.3 전체 합격 표시 제외

## test.4 최종 공개 수용

- 소스 `5ae6a5e2cfa8241f892183d695413cd447e8046c`, 제품 지문 `170db9262d8e7617f253f36fde097f091d5a2a6c1bd0a60f5652e531695ddc58`. [정확 소스 CI40989010752](https://github.com/gvm1229/aigent-hive/actions/runs/40989010752)와 [후보40992421959](https://github.com/gvm1229/aigent-hive/actions/runs/40992421959) 전체 성공. 다섯 배포 대상·기존 안정판 복구·직접 설치기 검증 포함
- [게시40995484009](https://github.com/gvm1229/aigent-hive/actions/runs/40995484009) 성공. 첫40994978732의 업로드 후60초 태그 확인 실패는 같은 후보의 기존 지문 대조·재사용으로 복구, 공개 파일 덮어쓰기 없음. 여섯 npm 정확 버전/test 태그와 GitHub 주석 태그의 소스5ae 일치, 모든 npm latest와 GitHub 최신 안정판13.1 독립 확인
- [공개test.4](https://github.com/gvm1229/aigent-hive/releases/tag/v0.14.0-test.4) 묶음 SHA256: `fbbf4083ebe5f93441f32530fc5958e6ecb7dbc560dfd02d236f1cda30e94ac6`. SHA512: `82da420e226c6380a2ba1a4a02c14eae1d15ba3cd286ee63d97e89e8f804b36429816f567e66e87b5c95bc76a2ca671f15052714e2f707b65bc90ef2dfa32c0a`
- [공개 수용40995822179](https://github.com/gvm1229/aigent-hive/actions/runs/40995822179), 첫 실행에서 세 OS 모두 성공. 아래45개 정본 실행 기록의 source5ae·공개 바이너리/입력 지문·명령·환경 결합과 실행 중 소스 변경 없음 확인. 이전 실패 기록 보존, 과거 결과로 새 공개 파일 실행 대체 없음

| 실제 실행 환경 | 공개 원본 갱신·보존 제거 | 새 프로젝트 | 취소·재개 | 고정 답안·기타 계약 |
| --- | --- | --- | --- | --- |
| Windows Server2025 x64, build26100 | [0.13.1 npm](../../tests/results/runs/20261030T031052-8a57c827436f.md)·[0.13.2-test.2 npm](../../tests/results/runs/20261030T031130-3e46508e66aa.md)·[직접 설치](../../tests/results/runs/20261030T031203-a321c69ab687.md)·[비대화형](../../tests/results/runs/20261030T031456-34829da2b47e.md) | [프로젝트](../../tests/results/runs/20261030T031552-47f4d0be4838.md) | [Ctrl+C 재개](../../tests/results/runs/20261030T031557-f60f2a350d50.md)·[무시 상속 회귀](../../tests/results/runs/20261030T031632-31d5ce546db5.md) | [고정 답안](../../tests/results/runs/20261030T031637-d5891e1f46bc.md)·[지침 복구](../../tests/results/runs/20261030T031756-132d9ecd2b80.md) |
| macOS15.7.9 arm64 | [0.13.1 npm](../../tests/results/runs/20261030T031030-981ba9233b87.md)·[0.13.2-test.2 npm](../../tests/results/runs/20261030T031106-25fe5457a290.md)·[직접 설치](../../tests/results/runs/20261030T031136-0d6a87fbaec0.md)·[비대화형](../../tests/results/runs/20261030T031202-de98e6434b9c.md) | [프로젝트](../../tests/results/runs/20261030T031230-03d112655677.md) | [취소·재개](../../tests/results/runs/20261030T031233-a83a2174a562.md) | [고정 답안](../../tests/results/runs/20261030T031238-dda3da37f8ed.md)·[지침 복구](../../tests/results/runs/20261030T031255-3a24d8f09090.md) |
| Linux x64, Ubuntu24.04·Azure kernel7.0 | [0.13.1 npm](../../tests/results/runs/20261030T031019-40d9b8766325.md)·[0.13.2-test.2 npm](../../tests/results/runs/20261030T031037-919a455f1ef3.md)·[직접 설치](../../tests/results/runs/20261030T031054-bececaeb277a.md)·[비대화형](../../tests/results/runs/20261030T031112-d0897c81d664.md) | [프로젝트](../../tests/results/runs/20261030T031135-538786fc293f.md) | [취소·재개](../../tests/results/runs/20261030T031138-620c1b13d312.md) | [고정 답안](../../tests/results/runs/20261030T031143-57f8f2a8ba38.md)·[지침 복구](../../tests/results/runs/20261030T031157-97cc75c92ac4.md) |

- 각 OS의 한국어/영어 전역 설정·비대화형 입력·협업 지침·벡터 선택 검사도 같은 실행에서 통과. Windows의 실제 [Windows Terminal](../../tests/results/runs/20261030T031605-9739ba5a0b50.md), [Codex CLI·대기 감시·예약·제거 실패 복구/재설치](../../tests/results/runs/20261030T031650-69c22f560e94.md) 포함
- 범위: 격리 GitHub OS와 임시 사용자/설치/캐시/프로젝트. 일반 설정·원본 갱신의 앱 연결은 제한된 대역, Codex 결합은 새 프로필의 실제 로컬 plugin/marketplace 명령. 실제 모델·구독 관측·데스크톱 대화 중단·개인 설치의 증명으로 확대하지 않는 한계. 나머지 Linux arm64·macOS x64는 후보 빌드/설치 검사 범위
- 모델 수용은 [별도 실제 Luna 근거](cli-model-validation-0.14.0.md): 고정7사례, 두 빠른 설정별 각판2표본. 입력 중앙값 전역60.70%·프로젝트50.009% 감소, 프로젝트 여유3바이트. 토큰·실비·처리 속도 미관측, 단위/대역 검사를 실제 모델 평가로 대체하는 처리 없음

## 원래 15개 완료 조건과 최종 근거

| 조건 | 완료 근거 | 최종 확인 |
| --- | --- | --- |
| P140-001 자료 형식·선택 출처 | [정책·자료형 기반](cli-foundations-0.14.0.md) | 완료 |
| P140-002 이전·중단 복구 | [설정 복구](cli-state-recovery-0.14.0.md) | 완료 |
| S140-001 공통 질문·입력 분리 | [공통 엔진·프로젝트](cli-project-validation-0.14.0.md), 위 공개4 터미널 | 완료 |
| S140-002 두 언어 전역 설정 | [전역 설정](cli-user-settings-0.14.0.md), 공개4 한국어/영어·재개 | 완료 |
| S140-003 프로젝트·기능·보존 | [보존 재설정](cli-project-reconfiguration-0.14.0.md), 위 공개4 프로젝트 | 완료 |
| S140-004 얇은 AI 연결·제한 출력 | [연결·출력](cli-local-validation-0.14.0.md), 공개4 고정 답안 회귀 | 완료 |
| U140-001 원본0.13.1 첫 연결 | [갱신 수용](cli-update-validation-0.14.0.md), 위 세 OS 공개4 npm/직접 명령 | 완료 |
| U140-002 조정기·보존·복구 | [기한·부분 실패·동일 버전 재개](cli-update-validation-0.14.0.md), 공개4 보존·재설치 | 완료 |
| U140-003 결과·안내·호환 출력 | [출력 계약](cli-update-validation-0.14.0.md), 공개4 원본 부모 결과 | 완료 |
| G140-001 기본값·전체 스위치 | [사용량 선택](cli-usage-settings-0.14.0.md), 공개4 새 설정·비대화형 | 완료 |
| G140-002 선택·감시 전환·실제 상태 | [감시 수용](cli-usage-settings-0.14.0.md), 공개4 실제 Windows 대기 감시 제거 | 완료 |
| D140-001 안전 제거·자료 보존 | [제거 수용](cli-uninstall-validation-0.14.0.md), 공개4 세 OS·Codex·예약 | 완료 |
| Q140-001 실제 모델·입력 비용 | [실제 Luna 측정](cli-model-validation-0.14.0.md), 지속 회귀 연결 | 완료 |
| Q140-002 통합 수용 | 위 정확한 공개4의 세 OS 첫 실행 전체 성공 | 완료 |
| Q140-003 공개 시험·출시 준비 | 후보40992421959·게시40995484009·독립 지문/태그·수용40995822179 | 완료 |

원래 승인된 구현·번호 공개 시험 범위15/15,100.0% 수용. `agent-owned`·`awaiting-external-evidence`·`blocked` 없음. 별도 `awaiting-user-authority`: 정식0.14.0 main 통합·정식 태그/게시·실사용 설치·새 공지 전송. 현재 버전별 명시 승인 전 실행 제외, 시험판 완료를 정식 출시로 표시하는 처리 없음.
