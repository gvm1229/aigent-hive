# 과거 별도 Rust 빌드 캐시 조사

- 조사일: 2026-08-29, 실제 Windows 로컬 파일 목록 검사
- 대상: `tests/work` 아래 별도 GNU 빌드 경로 23개
- 파일 크기 합계: 78,420,543,084바이트, 약 73.03GiB
- 용도: Cargo 실행 파일·라이브러리·증분 컴파일·빌드 중간 파일
- 시험 결과와 구분: 빌드 파일 존재만으로 시험 성공·실패 판정 불가
- 원래 개별 빌드 명령·실행 시각·시험 결과: 확인 가능한 완전 기록 부재, 추정 없음
- 현재 `scripts`·`crates`·CI·`docs`·conformance에서 해당 별도 경로 참조 0건
- 재생성: 필요한 정확한 소스·Rust 도구·타깃·Cargo.lock으로 새 빌드, 기존 캐시 바이트 복구 보장 없음
- 조사 당시 공개 수용 작업의 `tests/work` 전체 보호 유지, 삭제 미실행
- 이후 삭제 조건: 보호 해제·현재 프로세스·72시간 내 재사용 계획 재확인, 이 기록의 Git 보존

## 경로별 목록

| 경로 | 바이트 | 파일 수 |
| --- | ---: | ---: |
| `tests/work/gnu-target` | 11122390913 | 15510 |
| `tests/work/gnu-target-010-full` | 4547660261 | 5224 |
| `tests/work/gnu-target-010-version` | 782456323 | 1812 |
| `tests/work/gnu-target-continuation` | 3208075967 | 4054 |
| `tests/work/gnu-target-final` | 4125276307 | 4702 |
| `tests/work/gnu-target-full` | 4130111425 | 4702 |
| `tests/work/gnu-target-graph-cli` | 3942657714 | 5390 |
| `tests/work/gnu-target-graph-consent` | 3168657677 | 3986 |
| `tests/work/gnu-target-graph-export` | 3414193108 | 3910 |
| `tests/work/gnu-target-graph-incremental` | 1705893044 | 2295 |
| `tests/work/gnu-target-graph-lifecycle` | 3414617500 | 3910 |
| `tests/work/gnu-target-graph-lock` | 3176865215 | 3986 |
| `tests/work/gnu-target-graph-planner` | 3165280143 | 3986 |
| `tests/work/gnu-target-graph-pointer` | 1700957246 | 2295 |
| `tests/work/gnu-target-graph-scope` | 2009109933 | 2976 |
| `tests/work/gnu-target-graph-upgrade` | 2787418840 | 3466 |
| `tests/work/gnu-target-graphify-adapter` | 1923133652 | 3094 |
| `tests/work/gnu-target-judge` | 3474756215 | 5090 |
| `tests/work/gnu-target-korean` | 8647618967 | 12373 |
| `tests/work/gnu-target-skill-migration` | 3609425854 | 5334 |
| `tests/work/gnu-target-skill-migration-projection` | 815493871 | 1424 |
| `tests/work/gnu-target-update-retry` | 1425552747 | 2430 |
| `tests/work/v095-gnu-target` | 2122940162 | 3023 |

## 조사 지문

파일 경로·크기·수정 시각·파일 식별자의 목록 지문. 개별 파일 내용의 암호학적 동일성 증명과 구분.

```json
[
  {
    "path": "tests/work/gnu-target",
    "bytes": 11122390913,
    "files": 15510,
    "fingerprint": "b2425811dbaee71c7bfc479eb0ac3ed91c7a329fa78f29412621641b79276760"
  },
  {
    "path": "tests/work/gnu-target-010-full",
    "bytes": 4547660261,
    "files": 5224,
    "fingerprint": "10a2f67f2a21b56ac22b298093ca892a2d4768711c490f97a1efdb469148503e"
  },
  {
    "path": "tests/work/gnu-target-010-version",
    "bytes": 782456323,
    "files": 1812,
    "fingerprint": "b4e7f83b0af8e27897d8f8e6a293135b9ddacd3aee7b438d3123f41dc7c51e1a"
  },
  {
    "path": "tests/work/gnu-target-continuation",
    "bytes": 3208075967,
    "files": 4054,
    "fingerprint": "f050f48a1f7740f1c9757342b9a8924cd5eeaa8354c257621bc8e0da7c64bf9f"
  },
  {
    "path": "tests/work/gnu-target-final",
    "bytes": 4125276307,
    "files": 4702,
    "fingerprint": "36ffbf4e7afddfca9568a4f1d526be6a9dd04c36714196b180c81d70d7e3c945"
  },
  {
    "path": "tests/work/gnu-target-full",
    "bytes": 4130111425,
    "files": 4702,
    "fingerprint": "dbd74cea1e59109072be07b186a2055adde6792b8234336304209cc5fe4332b0"
  },
  {
    "path": "tests/work/gnu-target-graph-cli",
    "bytes": 3942657714,
    "files": 5390,
    "fingerprint": "fb5ee872b185c615715d30b367fe41a4ca84708fa224f20a3cc09ba9f2d14967"
  },
  {
    "path": "tests/work/gnu-target-graph-consent",
    "bytes": 3168657677,
    "files": 3986,
    "fingerprint": "af7cc26dfbc734a1de2fffe664d66f1e78bcb58038e2e9d898bad00e35934ddf"
  },
  {
    "path": "tests/work/gnu-target-graph-export",
    "bytes": 3414193108,
    "files": 3910,
    "fingerprint": "656efbafcd5c02b943c25c3e6e4f79b380c8fb73f97f08a3be7478bba8ff6e1c"
  },
  {
    "path": "tests/work/gnu-target-graph-incremental",
    "bytes": 1705893044,
    "files": 2295,
    "fingerprint": "4c06605aa8bd929af519e9ed9ee65698330cc583911da2ecc98ac92bb599ef02"
  },
  {
    "path": "tests/work/gnu-target-graph-lifecycle",
    "bytes": 3414617500,
    "files": 3910,
    "fingerprint": "7bcaa56a13556245a18db640ccef90bdd1ed8d184f1de0eafcaa59eb1a309762"
  },
  {
    "path": "tests/work/gnu-target-graph-lock",
    "bytes": 3176865215,
    "files": 3986,
    "fingerprint": "3b773c5d2de7a3f030596b3ab82ec7ac19672ed4791af595ee69ee07fc84dff0"
  },
  {
    "path": "tests/work/gnu-target-graph-planner",
    "bytes": 3165280143,
    "files": 3986,
    "fingerprint": "e139ddb8523bd970c287dff3e9f58a477b185bff651d627c0198a6a91676970e"
  },
  {
    "path": "tests/work/gnu-target-graph-pointer",
    "bytes": 1700957246,
    "files": 2295,
    "fingerprint": "841803475d991a123ef44bda7ce4640215f0fc934787760604cc80346446ff66"
  },
  {
    "path": "tests/work/gnu-target-graph-scope",
    "bytes": 2009109933,
    "files": 2976,
    "fingerprint": "bcdc88c886f45c6c6ebf02b227ec9b6bd33025209e8b81ddb19bb267925d0c76"
  },
  {
    "path": "tests/work/gnu-target-graph-upgrade",
    "bytes": 2787418840,
    "files": 3466,
    "fingerprint": "ae0d4fac8246c6e3af7abd4aad393d9bedccd6b2bf2bd58e2dc68f4cfebcb2ed"
  },
  {
    "path": "tests/work/gnu-target-graphify-adapter",
    "bytes": 1923133652,
    "files": 3094,
    "fingerprint": "1a2b53e3b25cca3150070042a29147cfb8465a97d3f3ccedccfb7571db710f46"
  },
  {
    "path": "tests/work/gnu-target-judge",
    "bytes": 3474756215,
    "files": 5090,
    "fingerprint": "c40201e77535f0191fac8433e40c6d525ee83f320597baaf12b75821b4261bbc"
  },
  {
    "path": "tests/work/gnu-target-korean",
    "bytes": 8647618967,
    "files": 12373,
    "fingerprint": "98b90467f54f1cae304d1906cc127a194b733ac5d849a5448833b2e6c59f7010"
  },
  {
    "path": "tests/work/gnu-target-skill-migration",
    "bytes": 3609425854,
    "files": 5334,
    "fingerprint": "abfff6a0ee7602257370847d4aa7b84587b1f00b676a5a355b2197e0bf3b4390"
  },
  {
    "path": "tests/work/gnu-target-skill-migration-projection",
    "bytes": 815493871,
    "files": 1424,
    "fingerprint": "6f3db456f3d662d080d55456ee4d5f47f917098360d7c534336ab68bf4111372"
  },
  {
    "path": "tests/work/gnu-target-update-retry",
    "bytes": 1425552747,
    "files": 2430,
    "fingerprint": "5a3a578967f82b89ad9134384de2ffd95aaacae048a9615a656cfc853905b9d1"
  },
  {
    "path": "tests/work/v095-gnu-target",
    "bytes": 2122940162,
    "files": 3023,
    "fingerprint": "d5af5c597dfdd52073be3ff26f9bc63856ebce6a755fb7971541c04170f5352e"
  }
]
```
