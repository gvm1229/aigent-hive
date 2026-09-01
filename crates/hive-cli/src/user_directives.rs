#[derive(Clone, Copy)]
pub(crate) enum UserDirectiveLanguage {
    En,
    Ko,
}

pub(crate) fn work_completion_block(language: UserDirectiveLanguage) -> &'static str {
    match language {
        UserDirectiveLanguage::En => {
            "- Continue `all todos`, `until completion`, `do not stop`, and equivalent requests while any scoped action is agent-owned; a progress report is not closure.\n\
- Before a whole Goal or task becomes `blocked`, require no independent agent-owned criterion and continue around partial host, fixture, test, stale-reference, or evidence failures.\n\
- Abort continued work only for an exact user-owned manual blocker, a required Codex restart, or completed criteria; user cancel or interrupt takes priority.\n\
- Classify remaining work as `agent-owned`, `awaiting-user-authority`, `awaiting-external-evidence`, or `blocked`, and finish every safe agent-owned action before handoff.\n\
- Every release request defaults to implementation, verification, or a numbered public test. Stable tag, protected-branch integration, publication, and installation require current version-specific user approval.\n\
- For every passed, failed, skipped, deferred, unverified, or unsupported result, state the scope, exact reason, actual host or platform execution, proven range, and unproven range.\n"
        }
        UserDirectiveLanguage::Ko => {
            "- `all todos`, `until completion`, `do not stop` 또는 같은 요청: Agent 소유 작업 잔존 시 계속 진행. 진행 보고를 closure로 사용 금지.\n\
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
