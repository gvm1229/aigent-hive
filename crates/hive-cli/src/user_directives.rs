#[derive(Clone, Copy)]
pub(crate) enum UserDirectiveLanguage {
    En,
    Ko,
}

pub(crate) fn work_completion_block(language: UserDirectiveLanguage) -> &'static str {
    match language {
        UserDirectiveLanguage::En => {
            "- Treat follow-up messages, including mid-turn steering, as additions by default. Preserve earlier goals, constraints, approvals, and compatible instructions; recency alone does not cancel them.\n\
- Replace or withdraw an earlier instruction only on the user's explicit request and only within its stated scope. Honor explicit cancellation or interruption immediately.\n\
- Without explicit replacement, even an obvious conflict requires presenting both requirements and asking which to keep. Pause the conflicting action and dependent changes until the user answers; continue independent compatible work. Silence and elapsed time are not an answer. Apply the answer only to that conflict.\n\
- Preserve active constraints and unresolved conflicts in existing plan or handoff summaries across compaction and resumption. A status question does not reset the task. Recover missing context or ask before the affected action; never invent consent or store raw transcripts. Respect higher-priority host instructions; this rule does not guarantee semantic detection or tool interception.\n\
- Continue `all todos`, `until completion`, `do not stop`, and equivalent requests while any scoped action is agent-owned; a progress report is not closure.\n\
- Before a whole Goal or task becomes `blocked`, require no independent agent-owned criterion and continue around partial host, fixture, test, stale-reference, or evidence failures.\n\
- Abort continued work only for an exact user-owned manual blocker, a required Codex restart, or completed criteria; user cancel or interrupt takes priority.\n\
- Classify remaining work as `agent-owned`, `awaiting-user-authority`, `awaiting-external-evidence`, or `blocked`, and finish every safe agent-owned action before handoff.\n\
- Every release request defaults to implementation, verification, or a numbered public test. Stable tag, protected-branch integration, publication, and installation require current version-specific user approval.\n\
- For every passed, failed, skipped, deferred, unverified, or unsupported result, state the scope, exact reason, actual host or platform execution, proven range, and unproven range.\n"
        }
        UserDirectiveLanguage::Ko => {
            "- 작업 중 추가 메시지는 기본적으로 기존 요청에 추가. 이전 목표·제약·승인·양립 가능한 지시 유지, 최신 메시지라는 이유만으로 이전 지시 취소 금지.\n\
- 이전 지시의 교체·철회는 사용자의 명시적 요청이 지정한 범위만 반영. 명시적 취소·중단은 즉시 존중.\n\
- 명시적 교체 없는 충돌은 명백한 모순이라도 두 조건을 제시하고 어느 쪽을 유지할지 질문. 답변 전 충돌 행동과 종속 변경 보류, 독립적인 양립 가능 작업 지속. 침묵·시간 경과는 답변 아님. 답변은 해당 충돌에만 적용.\n\
- 대화 요약·재개 시 기존 계획·인계 요약에 유효한 제약과 미해결 충돌 보존. 상태 질문은 작업 초기화 아님. 필요한 맥락이 없으면 해당 행동 전 복원 또는 질문, 승인 추정·대화 원문 저장 금지. 상위 호스트 지침 준수, 의미 충돌 탐지·도구 실행 가로채기의 보장 아님.\n\
- `all todos`, `until completion`, `do not stop` 또는 같은 요청: Agent 소유 작업 잔존 시 계속 진행. 진행 보고를 closure로 사용 금지.\n\
- 전체 Goal·task의 `blocked` 전 독립 Agent 소유 criterion `0건` 확인. 일부 host·fixture·시험·stale reference·증거 결손 우회 뒤 나머지 지속.\n\
- 중단 허용: 사용자 수동 해결 blocker, Codex restart, 모든 criterion 완료. 사용자 cancel·interrupt 우선.\n\
- 남은 작업 분류: `agent-owned|awaiting-user-authority|awaiting-external-evidence|blocked`. 인계 전 안전한 Agent 소유 작업 완료.\n\
- 모든 출시 요청의 기본값: 구현·검증·번호 공개 시험판. Stable tag·protected branch 통합·게시·설치: 현재 version별 사용자 명시 승인 필수.\n\
- 통과·실패·건너뜀·연기·미검증·미지원 결과: 범위·정확한 이유·실행 host·운영체제·증명 범위·미증명 범위 명시.\n"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{work_completion_block, UserDirectiveLanguage};

    #[test]
    fn bilingual_blocks_preserve_the_same_boundaries() {
        for block in [
            work_completion_block(UserDirectiveLanguage::En),
            work_completion_block(UserDirectiveLanguage::Ko),
        ] {
            assert!(block.contains("agent-owned") || block.contains("Agent 소유"));
            assert!(block.contains("blocked"));
            assert!(block.contains("Codex restart"));
            assert!(block.contains("cancel"));
            assert!(block.contains("Stable tag"));
            assert!(block.contains("failed") || block.contains("실패"));
        }
    }
}
