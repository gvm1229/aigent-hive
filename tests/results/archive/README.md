# 정식 이전 실행 기록 보관

> 보관일: 2026-11-10(OVD-009)
> 대상: 정식 0.14.0 게시(2026-10-30T15:01:26+09:00) 전에 시작했고 저장소 어디에서도 참조하지 않는 `runs/` 기록
> 원본 바이트: 각 압축 파일 안 `runs/<id>.md`와 첨부 폴더 그대로, 압축 해제 뒤 Git 이전 커밋과 sha256 일치 확인

## 복원

```bash
tar -xJf tests/results/archive/runs-before-0.13.1.tar.xz -C tests/results
```

## 버전별 요약

버전 열은 그 기록 뒤에 처음 게시된 정식 버전.

| 정식 버전 | 기록 수 | 파일 수 | 기간 | 결과 | 운영체제 | 압축 파일 |
| --- | ---: | ---: | --- | --- | --- | --- |
| 0.10.0 | 212 | 214 | 2026-08-28 – 2026-09-01 | passed: 158, failed: 54 | Windows: 212 | [runs-before-0.10.0.tar.xz](runs-before-0.10.0.tar.xz) |
| 0.10.1 | 66 | 66 | 2026-09-05 – 2026-09-06 | passed: 39, failed: 27 | Windows: 66 | [runs-before-0.10.1.tar.xz](runs-before-0.10.1.tar.xz) |
| 0.10.2 | 42 | 42 | 2026-09-06 – 2026-09-07 | passed: 23, failed: 19 | Windows: 42 | [runs-before-0.10.2.tar.xz](runs-before-0.10.2.tar.xz) |
| 0.10.3 | 7 | 7 | 2026-09-09 – 2026-09-09 | failed: 4, passed: 3 | Windows: 7 | [runs-before-0.10.3.tar.xz](runs-before-0.10.3.tar.xz) |
| 0.11.0 | 329 | 330 | 2026-09-12 – 2026-09-26 | passed: 223, failed: 104, interrupted: 2 | Windows: 329 | [runs-before-0.11.0.tar.xz](runs-before-0.11.0.tar.xz) |
| 0.11.1 | 177 | 177 | 2026-09-26 – 2026-10-04 | passed: 94, failed: 82, running: 1 | Windows: 177 | [runs-before-0.11.1.tar.xz](runs-before-0.11.1.tar.xz) |
| 0.12.0 | 148 | 148 | 2026-10-04 – 2026-10-05 | passed: 106, failed: 42 | Windows: 148 | [runs-before-0.12.0.tar.xz](runs-before-0.12.0.tar.xz) |
| 0.13.0 | 129 | 129 | 2026-10-15 – 2026-10-17 | passed: 100, failed: 27, running: 2 | Windows: 129 | [runs-before-0.13.0.tar.xz](runs-before-0.13.0.tar.xz) |
| 0.13.1 | 154 | 154 | 2026-10-18 – 2026-10-21 | passed: 115, failed: 39 | Windows: 154 | [runs-before-0.13.1.tar.xz](runs-before-0.13.1.tar.xz) |
| 0.14.0 | 865 | 865 | 2026-10-22 – 2026-10-30 | passed: 666, failed: 197, running: 1, interrupted: 1 | Windows: 826, Linux: 24, macOS: 15 | [runs-before-0.14.0.tar.xz](runs-before-0.14.0.tar.xz) |

## 버전별 주요 목적

| 정식 버전 | 상위 목적(건수) |
| --- | --- |
| 0.10.0 | Python conformance: documentation (12); Python conformance: documentation, security, contract, integration, release (11); 벡터 최초 설정 핵심 단위 시험 (3); 지식 이전 논리 프로젝트 연결 회귀 (2); 배포 빌드 baseline 100파일 내보내기 측정 (2) |
| 0.10.1 | 개발 검증: cargo.EXE (6); Python conformance: release (5); Python conformance: documentation (4); 개발 검증: uv.EXE (2); Python conformance: documentation, security, contract, integration, release (2) |
| 0.10.2 | Python conformance: contract (5); 0.10.2 update activation 단위 시험 (3); 0.10.2 질문 대기 사용자 투영 시험 (2); 0.10.2 질문 대기 차단 회귀 시험 (2); 0.10.2 전역 update 프로젝트 미접근 회귀 시험 (2) |
| 0.10.3 | 0.10.3 Hive update 회귀 조사 (1); 0.10.3 update fixture 재검증 (1); 0.10.3 update baseline 재검증 (1); 0.10.3 전체 Rust 검증 (1); 0.10.3 전체 Python 적합성 검증 (1) |
| 0.11.0 | 개발 검증: cargo.EXE (114); 개발 검증: uv.EXE (73); Python conformance: documentation, security, contract, integration, release (16); Python conformance: documentation (4); Python conformance: release (2) |
| 0.11.1 | Python conformance: documentation, security, contract, integration, release (13); 개발 검증: uv.EXE (8); Python conformance: integration (2); Python conformance: release (2); 0.11.1 사용자 자동 정리 컴파일 확인 (1) |
| 0.12.0 | Python conformance: contract (3); Python conformance: security (2); Python conformance: documentation, security, contract, integration, release (2); 최종 0.11.1 안정판 후보의 공개 파일 대조 준비 (1); 공개 0.11.1 안정판 여섯 패키지와 후보 SHA512 대조 (1) |
| 0.13.0 | 초기화 감시기 Rust 행동 검사 (1); 감시기 공유 상태·간격 회귀 (1); 감시기 CLI 통합 빌드 (1); 감시기 Rust 정적 검사 (1); 최종 감시기 단위 검사와 설치 버전 경계 (1) |
| 0.13.1 | Python conformance: release (2); 정식 0.13.0 여섯 latest·시험 표시·태그·공개 파일 독립 대조 (1); 정식 채널 인수를 명시한 0.13.0 공개 파일 독립 대조 (1); 0.13.1 조회 격리·관측 보정 Rust 컴파일과 집중 시험 (1); 0.13.1 조회 오류 분리·독립 재확인·단조 시계 회귀 (1) |
| 0.14.0 | 개발 검증: cargo.EXE (525); Windows Rust 검사: 시험 자식의 개인 환경 접근 차단 (54); 개발 검증: uv.EXE (25); Windows 설정 요약·작은 답안의 보존과 재개 (21); 격리 운영체제의 실제 CLI 흐름: setup (14) |

## 한계

- 계획·근거·안내 문서가 링크한 기록과 정식 0.14.0 이후 기록은 `runs/`에 유지
- 압축 기록은 `INDEX.md` 대상 밖, 검색은 압축 해제 뒤
