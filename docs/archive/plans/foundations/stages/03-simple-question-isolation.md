# Stage 3. 단순 질문 격리

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

명시 `AnswerSimpleQuestion` 또는 `hive-simple-question`은 다음 capability 없이 답변:

- project memory
- Wiki ingest/query
- repository mutation
- subagent
- external orchestration
- persistent run creation

질문 자체에 repository 정보가 필요하면 simple path를 거부하고 `RunWork` 전환을 제안. 자동 전환이나 write 없음.

#### 구현

- 최소 system contract만 포함한 portable Skill
- project root와 `.hive` mount 차단 가능한 host에서는 실제 차단
- 차단할 수 없는 host는 instruction-only로 표시하고 support matrix에서 구분
- simple response의 memory capture trigger 차단

#### 완료 조건

- [x] simple fixture에서 project file read/write 0회
- [x] subagent와 Skill 추가 load 0회
- [x] project-dependent 질문은 명시 전환 전 실행 0회
