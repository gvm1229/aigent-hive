# 공개 한국어 HTML 안내 계획

## 목표

- Hive 핵심 기능과 설치 과정을 각각 독립 실행형 한국어 HTML로 제공
- 사용자 제공 참조 HTML의 정보 구조·여백·타이포그래피 계승
- 강조색을 벌집 금색으로 전환하고 기존 Hive 로고 자산만 사용
- 최신 README branding commit을 기존 완료 증거로 확인

## 범위

- 출력: `docs/hive-core-features.ko.html`, `docs/hive-install-guide.ko.html`
- 로고: `docs/assets/branding/hive-logo-mark.png`
- 설치 기준: npm `latest`의 stable `0.8.0`
- 배포·호스팅·새 이미지 생성 제외

## Checklist

- [x] [PHG-001] 참조 HTML의 핵심 visual contract와 벌집 금색 token 확정
- [x] [PHG-002] provider-neutral 경계·설정·지식·Skill·상태·안전 기능을 담은 핵심 기능 HTML 작성
- [x] [PHG-003] 설치·host 활성화·전역 설정·project 설정·update 순서의 간단 설치 HTML 작성
- [x] [PHG-004] 지정 로고의 두 HTML 반영, 기존 README branding 확인과 HTML·link·명령·문서 말투 검증
- [x] [PHG-005] 공개 HTML 디자인 원칙 문서화와 설치 안내 3단계의 복수 호스트 CLI 계약·예시 보강
- [x] [PHG-006] 두 HTML의 원본 logo 내장·system font 전환·file-relative link 제거와 독립 파일 offline·responsive 검증

## 완료 기준

- UTF-8 한국어 독립 HTML 2개
- desktop·mobile 반응형 layout
- 빨강 accent 0건, 벌집 금색 accent 사용
- 외부 resource dependency 0건과 지정 repository logo 원본 byte의 HTML 내부 내장
- 현재 stable 설치 명령과 host별 대체값 정확성
- README의 기존 영문·한국어 branding 표시 확인

## 완료 증거

- 계획 commit: `9d32b24`
- HTML commit: `a9224cc`
- 두 HTML LXML parse·anchor·local asset·상호 link PASS
- 벌집 금색 `#F5A623`·responsive CSS·지정 logo 경로 확인
- Stable install·version·host 활성화·update 명령 확인
- README branding commit `245ae80`의 영문·한국어 banner 확인
- 디자인 원칙: `docs/guides/public-html-design-principles.md`
- `hive install --host` 단일 값 parser·복수 `selected_hosts` 추가 단위 test와 host별 명령 예시 확인
- HTML별 정본 PNG SHA-256 일치, network·file-relative resource 0건, 내부 anchor 전체 유효
- 프로젝트 밖 임시 폴더의 Edge offline desktop `1440×1200`·mobile `390×844` render PASS
- 독립 공유 commit `6f861b1`, Source Wiki fact commit `0b3bbbb`
