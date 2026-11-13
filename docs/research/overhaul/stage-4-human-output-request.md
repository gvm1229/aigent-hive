# 2026-11-13 최우선 요청 원문

출처: 현재 유지관리자 요청. 범위: current-project, 0.15.1 재정비. 아래 세 문단의 정확한 외부 인용

> "upon inspecting the output Hive gave me in the CLI via the usage threshold command, that is hardly recognizable by the average user, as it's a JSON format rather than an actual interactive conversation. Rather, it should be something like "Success. Hive usage guard threshold limit lowered from 30% to 10%.". This applies to all Hive commands, not just the usage command. It must be fixed with the highest priority."

> "0.15.1 introduced a new feature. When introducing a new feature, it must be in full context. But the 0.15.0 update notes did not detail each 9 gods, but rather only used a handful of examples. The update note skill must be amended to provide the full context of a new feature to the user in a user-friendly way that doesn't require developer-level context. After that's done, I would like you to edit the 0.15.0 update notes on discord to correctly provide context to the user as in what's actually updated and what they will get. If editing Discord messages is not possible, then erase the 0.15.0 update note and send a new one."

> "The above are to be included in the overhaul scope, and to be applied as soon as possible with the highest priority."

실행 경계: 단계 4 브랜치의 계획·구현·시험·커밋·push, Fable 추가 검토 우선. 현재 태그·게시·병합 제외. Discord 정정의 사전 승인과 main 반영 뒤 실행 순서 구분

## 2026-11-14 재확인과 정정

출처: 현재 유지관리자 요청. 아래 두 문단과 서두의 정확한 외부 인용. 이 원문이 위 2026-11-13 원문보다 우선

> "And the following must be included in the overhaul scope and must be done with the hightest priority, preceding anything else:"

> "Upon inspecting the output Hive gave me in the CLI via the usage threshold command, that is hardly recognizable by the average user, as it's a JSON format rather than an actual interactive conversation. Rather, it should be something like "Success. Hive usage guard threshold limit lowered from 30% to 10%.". This applies to all Hive commands, not just the usage command. It must be fixed with the highest priority."

> "Also, one other thing to point out is the Discord notification update note. 0.15.0 introduced a new feature. When introducing a new feature, it must be in full context. But the 0.15.0 update notes did not detail each 9 gods, but rather only used a handful of examples. The update note skill must be amended to provide the full context of a new feature to the user in a user-friendly way that doesn't require developer-level context. After that's done, I would like you to edit the 0.15.0 update notes on discord to correctly provide context to the user as in what's actually updated and what they will get. If editing Discord messages is not possible, then erase the 0.15.0 update note and send a new one."

- 정정: 새 기능(신 아홉)을 도입한 버전은 **0.15.0**. 2026-11-13 원문의 "0.15.1 introduced"는 이 원문으로 대체
- 우선순위: 재정비의 다른 모든 작업보다 먼저. 단계 4의 병합·공개 시험·정식, 단계 5 이후 시작 모두 OV151-013–018 완료 뒤
- 같은 날 추가 지시: "the 0.15.0 update note on discord must be either edited or sent again BEFORE the 0.15.1 update note hits. To keep the version update note ordered." [게시 순서](stage-4-discord-correction.md#실행-순서)
