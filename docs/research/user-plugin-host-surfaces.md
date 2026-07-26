# User plugin host surface 조사

- 조사일: 2026-07-26
- 범위: Codex·Claude Code·Antigravity user plugin, global guidance, global·project Skill
- 결론: 세 host native plugin packaging 채택, host별 guidance marker append, project
  portable `.agents/skills/` 유지

## Capability matrix

| Host | Native plugin package | User install·discovery | Global guidance | Global Skill | Project Skill | Qualification |
| --- | --- | --- | --- | --- | --- | --- |
| Codex CLI | `.codex-plugin/plugin.json`, `skills/` | local marketplace 등록 후 `codex plugin add` | `~/.codex/AGENTS.override.md` 우선, 없으면 `~/.codex/AGENTS.md` | `$HOME/.agents/skills/` | `<repo>/.agents/skills/` | local `0.145.0` fixed argv 성공 |
| Claude Code | `.claude-plugin/plugin.json`, `skills/` | marketplace 등록·`--scope user` install | `~/.claude/CLAUDE.md` | `~/.claude/skills/` | `<repo>/.claude/skills/` | 공식 CLI contract 확인, local executable 부재 |
| Antigravity | root `plugin.json`, `skills/` | `~/.gemini/config/plugins/<plugin>/` 자동 scan | `~/.gemini/GEMINI.md` | `~/.gemini/config/skills/` | `<repo>/.agents/skills/` | 공식 Antigravity `2.3.1` surface 확인 |

## Hive adapter

### Codex

- Package: `~/.hive/marketplaces/codex/plugins/aigent-hive/`
- Marketplace: `~/.hive/marketplaces/codex/.agents/plugins/marketplace.json`
- Activation:
  `codex plugin marketplace add <marketplace-root> --json` →
  `codex plugin add aigent-hive@aigent-hive --json`
- Guidance target: active non-empty `AGENTS.override.md` 우선, 그 외 `AGENTS.md`
- Existing OMX·foreign bytes 보존과 `AIGENT-HIVE:USER` own-block 교체

### Claude Code

- Package: `~/.hive/marketplaces/claude/plugins/aigent-hive/`
- Marketplace: `~/.hive/marketplaces/claude/.claude-plugin/marketplace.json`
- Activation: plugin validate → marketplace add/update → user-scope install/update
- Guidance target: `~/.claude/CLAUDE.md`
- Existing OMC·foreign bytes 보존과 `AIGENT-HIVE:USER` own-block 교체
- Actual Claude executable qualification: release 보호 환경 과제

### Antigravity

- Native package: `~/.gemini/config/plugins/aigent-hive/`
- Required marker: root `plugin.json`
- Bundled Skill: `~/.gemini/config/plugins/aigent-hive/skills/`
- Compatibility projection: `~/.gemini/config/skills/`
- Guidance target: `~/.gemini/GEMINI.md`
- Native CLI mutation 없음, documented global directory scan 사용

## Version boundary

| Surface | Supported range metadata | Evidence |
| --- | --- | --- |
| Codex plugin | `>=0.145.0 <1.0.0` | local CLI help·install command qualification |
| Claude Code plugin | `>=2.1.0 <3.0.0` | current plugin marketplace·scope documentation |
| Antigravity plugin | `>=2.3.1 <3.0.0` | current plugin documentation header와 global plugin path |
| Antigravity usage sensor | CLI `1.1.7` | 별도 usage TUI 조사; plugin host range과 분리 |

## 근거

- [OpenAI Codex AGENTS.md](https://developers.openai.com/codex/guides/agents-md)
- [OpenAI Codex Skills](https://developers.openai.com/codex/skills)
- [OpenAI Codex app-server plugin methods](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md)
- [Anthropic Claude Code marketplace](https://code.claude.com/docs/en/plugin-marketplaces)
- [Anthropic Claude Code plugin reference](https://code.claude.com/docs/en/plugins-reference)
- [Anthropic Claude Code memory](https://code.claude.com/docs/en/memory)
- [Anthropic Claude Code Skills](https://code.claude.com/docs/en/skills)
- [Google Antigravity Plugins](https://www.antigravity.google/docs/plugins)
- [Google Antigravity Skills](https://antigravity.google/docs/skills?app=antigravity-ide)
- [Google Antigravity Rules](https://antigravity.google/docs/ide/rules)
