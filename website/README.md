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

## 범위

- 제품 패키지와 설치본에 포함되지 않는 원본 저장소 전용 자료
- 배포 위치와 도메인은 미정
