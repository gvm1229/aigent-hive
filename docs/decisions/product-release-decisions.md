# 제품·배포 결정

기준일: 2026-07-23

| 영역 | 결정 |
| --- | --- |
| 구현 언어 | Rust stable, Cargo workspace |
| 제품 형태 | macOS·Windows CLI-first, 별도 GUI 없음 |
| 실행 경계 | 사용자가 로그인한 정액제 subscription host 위에서만 동작 |
| API | model-provider API 호출·SDK·API key 전부 금지 |
| 소스와 출하 | Hive source, release bundle, consumer harness 분리 |
| setup template | Copier 9.17.0을 authoring·CI에 사용, 소비자 runtime dependency 금지 |
| canonical data | 지식·role·run은 Markdown, setup/config/approval은 tracked YAML/TOML, Raw는 허용된 source object |
| SQLite | 삭제 가능한 FTS·tag·link index, Git 제외, 무네트워크 재구축 |
| orchestration | Codex의 compatible OMX, Claude의 compatible OMC를 우선하고 부재 시 host native가 소유; setup 선택지 없음 |
| runtime 관찰 | active host capability metadata와 side-effect-free public `--version`만 허용, foreign state read 금지 |
| 재구현 금지 | Hive plan·Ralph·team·provider session engine 없음 |
| Skill | 이름·source·revision·content digest·권한을 보여준 뒤 개별 수동 승인; 승인 후 narrow description에 따라 자동 사용, OMX/OMC duplicate보다 external 우선 |
| prompt refine | `hive-prompt-refine`; 명시적인 prompt 작성·정제 intent에서만 자동 선택, `refine-only` 기본, hidden rewrite 금지 |
| fallback hooks | OMX/OMC가 conclusively absent이고 사용자가 capability/event/path/digest를 승인한 경우에만 project-local data-integrity hook 허용 |
| 사용량 | 기본 중지선 20% remaining, 신뢰 가능한 local sensor가 있을 때만 자동 gate |
| judge | elevated 2/3, critical 3/3 + human |
| backup | update 전 생성, 최대 7일, Git 제외 |
| 저장소 | 비기밀 canonical source와 data는 Git 추적, runtime/cache/SQLite 제외 |
| 배포 정본 | GitHub Releases |
| 현재 버전 | barebone source baseline `0.1.0`; root Cargo workspace version이 정본 |
| 버전 증가 | feature는 원칙적으로 `Y`, compatible quick bugfix는 `Z`; `X`는 exact target을 사용자가 명시하고 human confirmation한 경우에만 |
| 호환성 | major `0`을 포함해 같은 major만 non-breaking upgrade 보장 |
| cross-major | 사전 경고, 자동 migration, project/docs/preferences 보존, SQLite rebuild |
| Git | `main` 안정, `develop` 일반 개발; `develop → main` PR |
| 라이선스 | CLI/source, `harness/**`와 생성된 Hive 소유 material 모두 `Apache-2.0` |

## 미확정이지만 구현을 막지 않는 항목

- Rust release/signing library와 정확한 key custody 구현
- host별 subscription usage sensor
- Homebrew·WinGet 배포 자동화의 첫 release 범위

미확정 항목은 지원된 것으로 표시하지 않는다.
