# 지식 이전 공개 시험판 수용

## 배포 식별

- 제품: `0.10.0`; 공개 패키지: `0.10.0-test.7`; 배포일: `2026-08-31`
- 제품 소스: `75a803a7a85c0e136119dc4eff06143e09b9e832`
- [출시 후보](https://github.com/gvm1229/aigent-hive/actions/runs/33367751906): 다섯 실행 대상과 npm 묶음 생성 성공
- [시험판 게시](https://github.com/gvm1229/aigent-hive/actions/runs/33368655619): 검증한 후보 파일 게시 성공
- [GitHub 시험판](https://github.com/gvm1229/aigent-hive/releases/tag/v0.10.0-test.7): `isPrerelease=true`, 태그 소스 일치
- npm 독립 조회: 정확 버전 `0.10.0-test.7`, `test=0.10.0-test.7`, `latest=0.9.5`
- 안정판 태그·게시·설치·`main` 통합·Discord 전송 없음

## 공개 설치 검증

[실행 33369217961](https://github.com/gvm1229/aigent-hive/actions/runs/33369217961)의 정확한 npm 공개 파일 설치 수용 성공.
기존 `release-runtime.yml`의 공개 시험 입력으로 보조 검사 호출. 보조 파일의 직접 실행 요청은 기본 브랜치 미등록으로 거부, 제품 변경 없이 기존 실행 경로 사용.

| 실행 대상 | 공개 설치·한국어·갱신·복구 | 실제 모델·지식 이전 |
| --- | --- | --- |
| Windows x64 | 통과 | 통과 |
| macOS arm64 | 통과 | 통과 |
| Linux musl x64 | 통과 | 통과 |

- 세 운영체제 실제 결과: 원문 8개 지문 일치, 묶음에 벡터 파일 0개, 새 임베딩 8개 생성·완성 색인 재사용
- 공통: 미설치·손상 때 FTS 유지, 의미 검색·출처, 비활성화·복구, 공유·소스 지식 생성 검증
- Windows: 실제 모델 자식 취소와 기존 활성 상태 보존·재개 통과. 생성과 Job 할당 사이 원자성은 미증명
- `source_worktree_dirty=true`: 검사 실행 전 `test-artifacts.py`의 결과 Markdown 생성 포함. 깨끗한 작업 트리 검증으로 해석 금지
- 제품 식별 근거: 외부 실행 절차의 태그 소스·제품 경로 차이 0건 검사와 정확 npm 버전 설치. 하위 벡터 검사 단독의 `public package identity` 미증명 표시와 구분

### 공개 실행 파일·원본 결과 지문

| 운영체제 | 실행 파일 SHA-256 | 벡터 결과 SHA-256 |
| --- | --- | --- |
| Windows x64 | `61a75179cf38dfdf5d7255c7c5c65b8f879fa3c75e31c1b9bcac783fd4dba295` | `1638bb44f022ac33e959b7e597d620502ab84c6d983510fabd0ebf443f528314` |
| macOS arm64 | `53e2c0d7f86f5aaf9ad5d13f0bc460806fe05b38a14d4cfe31b249d4d86c2cf4` | `0ffb0e405706e94969468e83aeefcfcc4e6fbd04c9ea4686c153eeb8bdfab549` |
| Linux musl x64 | `4ee8137d9cd2822ad3a117cf653661e20f69b956846ad7841d8bb8f0baaee388` | `1e96d0dc8d65727e1403fc119d476c8a886345518cd1a958af2aaca1f31bf8a6` |

원본 산출물: 같은 실행의 `korean-public-test-win32-x64`, `korean-public-test-darwin-arm64`, `korean-public-test-linux-x64`.
별도 일반 native 빌드 작업 4개는 공개 패키지 검증 모드에서 의도적으로 제외. 위 세 공개 설치 작업은 모두 실제 실행·성공.

## 증명 범위와 한계

- 지식 이전의 [세 방향 교차 운영체제 시험](knowledge-transfer-cross-os-2026-08-31.md)과 공개 패키지 설치 검증의 구분
- [Windows 100·1,000·5,000파일 비교](knowledge-transfer-2026-08-31.md): 변경 전후 각 1회, 모든 기기의 속도 보장 제외
- 공개 검증의 합성 자료·격리 사용자 루트 사용. 실제 사용자 전역 설치·지식 변경 없음
- macOS arm64와 Linux musl x64의 CLI 실행 검증. Intel Mac·Linux arm64의 후보 생성은 벡터 설치 수용과 별개
- Linux musl 실행 파일의 호스트는 Ubuntu. Alpine musl Python의 벡터 지원 주장 제외
- 실제 모델 자식 취소는 Windows 검사 범위. Unix의 같은 취소 동작은 별도 미증명
- 5만 청크 성능과 비공개·기밀 격리의 재측정은 이번 소규모 공개 검사 범위 밖. 기존 전체 회귀·전용 수용 근거와 구분
- Skill·지침·결정적 CLI 검증과 실제 호스트 대화의 자동 질문·Judge 실행은 별개
- `VON10-*`와 전체 출시 체크리스트 완료를 이번 지식 이전 수용으로 대체 금지

## 연결 자료

- [사용자 안내](../../docs/guides/knowledge-transfer.md)
- [구독자 업데이트 검토 초안](../../docs/releases/0.10.0.subscriber.ko.md): 미전송
- [지식 이전 계획](../../docs/plans/active/knowledge-transfer-0.10.0.md)
