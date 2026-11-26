# 소개 사이트 시제품

Aigent Hive 소개와 업데이트 전후 대화 비교를 담은 단일 HTML 페이지.

## 구성

| 경로 | 역할 |
| --- | --- |
| `src/index.html` | 정본 페이지. 로고 자리 `{{HIVE_LOGO}}` 포함 |
| `build.py` | 로고 원본 byte를 data URI로 한 번 내장해 `dist/index.html` 생성 |
| `dist/` | 생성 결과, Git 추적 제외 |

## 만들기와 확인

```console
python3 website/build.py
```

- 결과: 로고를 내장한 단일 파일 `website/dist/index.html`, 외부 참조는 Pretendard 글꼴 링크(일본어 화면은 Pretendard JP 추가)뿐
- 장면 주소: `index.html#memory`, `#coauthor`, `#backup`, `#usage`, `#msg`, `#skills`
- 장면 작성 규칙과 정확성 기준: [업데이트 전후 대화 비교](../docs/guides/release-comparison-mockups.md)
- 디자인 기준: [공개 HTML 디자인 원칙](../docs/guides/public-html-design-principles.md)
- 기본 글꼴: Pretendard Variable 1.3.9 동적 부분 집합, jsDelivr 링크. 이후 글꼴은 Pretendard 공식 권장 순서의 시스템 글꼴
- 문서 HTML과의 차이: 웹 배포용이라 글꼴만 외부 참조 허용. 오프라인 확인 시 시스템 글꼴로 대체

## 페이지 구성

| 순서 | 구역 | 내용 |
| --- | --- | --- |
| 1 | Hero | 방문자 운영체제에 맞춘 설치 명령 한 줄과 복사 버튼 |
| 2 | 0.16.0 비교 | 장면별 업데이트 전후 대화 |
| 3 | 지원 앱 | Codex·Claude Code·Gemini Antigravity별 되는 것과 미검증·주의 사항 |
| 4 | 특징 | API 키 요청 없음, 텍스트 정본, 바꾸기 전 백업, 동의 뒤 적용 |
| 5 | 최근 업데이트 | 최근 정식 3개의 날짜·요약과 릴리스 노트 링크 |
| 6 | 설치 | 설치 방법 탭, 운영체제에 맞는 탭 미리 선택 |
| 7 | FAQ | 비용, 지원 앱, 저장 위치, 실행 주체, 설치 조건, 되돌리기, 제거 |
| 8 | 마무리 띠 | 설치 명령 한 줄 반복과 GitHub 이동 |

- 참고 사이트: [Claude Code 제품 페이지](https://claude.com/ko/product/claude-code)의 구성과 기법만 참고. 상표·문구 복사 금지
- 금지: 실제 자료가 없는 고객 로고·추천사·사용 통계
- 운영체제 판별: macOS·Linux는 curl, Windows는 PowerShell, 휴대 기기·판별 불가는 npm
- 앱별 미검증 항목과 FAQ 답은 README와 릴리스 노트 근거만 사용

## 언어

| 항목 | 기준 |
| --- | --- |
| 지원 언어 | 한국어 `ko`, 영어 `en`, 일본어 `ja` |
| 선택 순서 | 주소의 `?lang=`, 오른쪽 위 메뉴에서 고른 언어(브라우저 저장), 브라우저 언어 목록 순서 |
| 대체 언어 | 지원하지 않는 브라우저 언어는 영어 |
| 주소 예시 | `index.html?lang=ja#usage`: 일본어로 사용량 장면 열기 |
| 메뉴 선택 | 화면 즉시 전환, 주소의 `?lang=` 갱신, 다음 방문에 같은 언어 |
| 일본어 글꼴 | 일본어 선택 시에만 Pretendard JP Variable 1.3.9 추가 로드 |
| Discord 알림 카드 | 제품이 한국어로만 보내므로 모든 언어에서 실제 한국어 문구 유지, 영어·일본어는 안내 한 줄 추가 |

## Vercel 배포

설정 파일: [`vercel.json`](vercel.json)

| 항목 | 값 | 이유 |
| --- | --- | --- |
| Root Directory | `website` | 소개 사이트만 배포 |
| 바깥 파일 포함 | 켬(기본값) | `build.py`가 `docs/assets/branding`의 로고 읽기 |
| Framework | 없음 | 정적 파일 한 개 |
| 설치 명령 | 없음 | 패키지 설치 불필요 |
| 만들기 명령 | `python3 build.py` | `dist/index.html` 생성 |
| 결과 폴더 | `dist` | 생성 결과 위치 |
| 배포 브랜치 | `main`만 | 다른 브랜치는 배포 기록 생성 없음 |
| 건너뛰기 | 사이트 파일·로고 변경 없음 | 불필요한 빌드와 취소 기록 감소 |

무료 요금제 보호를 위한 대시보드 설정(파일로 지정 불가):

1. Project Settings → Build and Deployment → Deployment Retention Policy
2. Canceled·Errored·Pre-Production·Production을 모두 가장 짧은 값(`1 day`)으로 선택 뒤 Save
3. 항상 남는 배포: 현재 Production 별칭 배포, 최근 생성 배포 3개, Ready 상태 Production 배포 3개(Hobby 기준)

## 범위

- 제품 패키지와 설치본에 포함되지 않는 원본 저장소 전용 자료
- 도메인은 미정
