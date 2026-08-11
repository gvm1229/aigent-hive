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

- [ ] [PHG-001] 참조 HTML의 핵심 visual contract와 벌집 금색 token 확정
- [ ] [PHG-002] provider-neutral 경계·설정·지식·Skill·상태·안전 기능을 담은 핵심 기능 HTML 작성
- [ ] [PHG-003] 설치·host 활성화·전역 설정·project 설정·update 순서의 간단 설치 HTML 작성
- [ ] [PHG-004] 지정 로고의 두 HTML 반영, 기존 README branding 확인과 HTML·link·명령·문서 말투 검증

## 완료 기준

- UTF-8 한국어 독립 HTML 2개
- desktop·mobile 반응형 layout
- 빨강 accent 0건, 벌집 금색 accent 사용
- 외부 image dependency 0건과 지정 repository logo 경로 사용
- 현재 stable 설치 명령과 host별 대체값 정확성
- README의 기존 영문·한국어 branding 표시 확인
