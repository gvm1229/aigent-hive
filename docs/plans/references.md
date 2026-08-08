# Review 후보와 reference

> 상태: 비규범 reference; 조건 충족 시에만 load

## Review needed

| 항목 | 현재 판정 | 다시 검토할 조건 |
| --- | --- | --- |
| OpenClaw | core와 초기 release 제외 | 세 host 기반이 안정된 뒤 별도 host adapter 수요와 conformance 증거 |
| CodexBar | optional local Codex usage sensor `0.45.2` qualified | machine-readable account/window/freshness contract가 바뀌거나 multi-platform sensor 수요가 확정될 때 |
| usage-coach | dependency 제외, reference only | Hive usage guard에서 검증된 기능 결손 발생 |
| multi-agent-starter | reference only | role/run schema에서 검증된 결손 발생 |
| Copier | authoring·CI에 채택, runtime 제외 | Rust parity/ownership 방식 대비 live Copier의 안전성 증거가 생길 때만 경계 재검토 |
| qmd/vector DB | 제외 | Markdown index+SQLite FTS recall/latency corpus 실패 |
| Obsidian integration | 유보 | local Markdown workflow 안정 후 실제 탐색 UX 수요 |
| cloud DB/VPS | 제외 | multi-machine concurrent writer 요구가 확정 |
| dashboard/desktop app | 유보 | CLI public release와 recovery gate 완료 |
| Rust TUF/signing library | `ed25519-dalek` verifier-only 채택, signing은 external | Private-key custody를 product에 넣지 않고 verifier audit 결손이 확인될 때만 재검토 |
| Antigravity projection surface | native global plugin·Skill path qualified | host major 또는 plugin manifest contract 변경 |
| cargo-dist | `0.8.0` 즉시 채택 제외, fit-gap reference | Hive ownership receipt·atomic recovery·단일 artifact 계보를 생성 workflow가 보존할 때 |
| web/unreal profile source | 아직 미이식 | 각 reference의 generic 부분을 별도 검토하고 domain fixture·precedence test 작성 |

## References

### 지식·지침·Skill

- [Andrej Karpathy — Wiki reference](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- [multica-ai — andrej-karpathy-skills](https://github.com/multica-ai/andrej-karpathy-skills)
- [Agent Skills specification](https://agentskills.io/specification)
- [OpenAI Codex — ExecPlans](https://developers.openai.com/cookbook/articles/codex_exec_plans)
- [Anthropic Claude Code — Memory](https://code.claude.com/docs/en/memory)

### Template와 update

- [Copier — Configuring a template](https://copier.readthedocs.io/en/stable/configuring/)
- [Copier — Updating a project](https://copier.readthedocs.io/en/stable/updating/)
- [RFC 8785 — JSON Canonicalization Scheme](https://www.rfc-editor.org/info/rfc8785)
- [The Update Framework](https://theupdateframework.io/)
- [GitHub artifact attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations)
- [cargo-dist — configuration](https://axodotdev.github.io/cargo-dist/book/reference/config.html)
- [cargo-dist — artifact attestations](https://axodotdev.github.io/cargo-dist/book/signing/attestations.html)

### CLI 배포

- [OpenAI Codex — npm·install script](https://github.com/openai/codex#installation)
- [Anthropic Claude Code — install methods](https://docs.anthropic.com/en/docs/claude-code/setup)
- [Google Gemini CLI — npm installation](https://github.com/google-gemini/gemini-cli#quickstart)
- [npm — `package.json` `bin`·optional dependencies](https://docs.npmjs.com/cli/v11/configuring-npm/package-json)
- [npm — Trusted Publishing](https://docs.npmjs.com/trusted-publishers)
- [npm — dist-tags](https://docs.npmjs.com/adding-dist-tags-to-packages)

### 라이선스

- [Apache Software Foundation — Apache-2.0 적용 방법](https://www.apache.org/legal/apply-license)
- [REUSE Specification 3.3](https://reuse.software/spec/)

### Orchestration compatibility

- [Yeachan-Heo — oh-my-codex](https://github.com/Yeachan-Heo/oh-my-codex)
- [Yeachan-Heo — oh-my-claudecode](https://github.com/Yeachan-Heo/oh-my-claudecode)
- [OMX — Codex native hook mapping](https://github.com/Yeachan-Heo/oh-my-codex/blob/main/docs/codex-native-hooks.md)
- [OMC — Architecture and Skill routing](https://github.com/Yeachan-Heo/oh-my-claudecode/blob/main/docs/ARCHITECTURE.md)
- [netwaif — multi-agent-starter](https://github.com/netwaif/multi-agent-starter)

### Host Skills와 hooks

- [OpenAI Codex — Hooks](https://learn.chatgpt.com/docs/hooks)
- [OpenAI Codex — Skills](https://developers.openai.com/codex/skills)
- [OpenAI Codex — AGENTS.md](https://developers.openai.com/codex/guides/agents-md)
- [Anthropic Claude Code — Skills](https://code.claude.com/docs/en/skills)
- [Anthropic Claude Code — Plugins](https://code.claude.com/docs/en/plugins-reference)
- [Anthropic Claude Code — Hooks](https://code.claude.com/docs/en/hooks)
- [Google Antigravity — Skills](https://antigravity.google/docs/skills?app=antigravity-ide)
- [Google Antigravity — Plugins](https://www.antigravity.google/docs/plugins)
- [Google Antigravity — Hooks](https://antigravity.google/docs/hooks?app=antigravity)

### Usage

- [steipete — CodexBar](https://github.com/steipete/CodexBar)
- [netwaif — usage-coach](https://github.com/netwaif/usage-coach)

### 배경 영상

- [Video 1 — Claude Code project principles](https://youtu.be/KWrsLqnB6vA)
- [Video 2 — Obsidian second brain and AI team](https://youtu.be/R2aSqw7S3Ws)
- [Video 3 — Agentic OS](https://youtu.be/HRw-vP0j8OM)
