<p align="center">
  <h1 align="center">Railguard</h1>
  <p align="center"><strong>Secure runtime for Claude Code.<br>The safer alternative to <code>--dangerously-skip-permissions</code>.</strong></p>
  <p align="center"><a href="https://railguard.dev">railguard.dev</a></p>
</p>

<p align="center">
  <a href="https://crates.io/crates/railguard"><img src="https://img.shields.io/crates/v/railguard.svg" alt="crates.io"></a>
  <a href="https://github.com/railguard-dev/railguard/stargazers"><img src="https://img.shields.io/github/stars/railguard-dev/railguard?style=flat" alt="GitHub stars"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/tests-151%20passed-brightgreen" alt="Tests">
  <img src="https://img.shields.io/badge/built%20with-Rust-orange.svg" alt="Built with Rust">
  <a href="https://discord.gg/MyaUZSus"><img src="https://img.shields.io/badge/discord-join-7289da.svg" alt="Discord"></a>
</p>

---

## The problem

You want Claude Code to run autonomously — but `--dangerously-skip-permissions` is all-or-nothing. Either every command needs your approval, or nothing does. There's no middle ground.

Railguard gives you the middle ground. It lets the agent run at full speed while enforcing the guardrails that matter: command-level blocking, path fencing, memory safety, and provenance tracking. You stay in control without babysitting every tool call.

```bash
cargo install railguard
railguard install
```

That's it. You keep using `claude` exactly as before.

---

## How is this different from the sandbox?

Claude Code's built-in sandbox is a **container-level boundary** — it restricts filesystem access and network at the OS level. It's designed for isolation: the agent runs inside a box and can't reach outside it.

Railguard is a **secure runtime** — not just a sandbox. It decides which commands are safe, guards agent memory, tracks what changed and why, snapshots files for recovery, and coordinates multiple agents. OS-level sandboxing (`sandbox-exec` / `bwrap`) is one tool Railguard uses to verify what actually executes at the kernel level, but it's one part of a larger system. Railguard enables you to run the agent against real production, by governing what it can do.

| | Sandbox | Railguard |
|---|---|---|
| **Model** | Container isolation | Policy-driven guardrails |
| **Scope** | All-or-nothing | Per-command, per-path, per-content |
| **Works with** | Sandboxed environments | Real production assets |
| **Customizable** | No | Yes — `railguard.yaml` |
| **Memory protection** | No | Yes — classifies and guards agent memory |

They solve different problems. You can use both.

---

## Features

### Command-level blocking

Every bash command the agent runs is evaluated against semantic rules and an OS-level sandbox. Safe commands flow through instantly (<2ms). Dangerous commands are blocked or require your approval.

```
  npm install && npm run build          ✅ allowed
  git commit -m "feat: add auth"        ✅ allowed
  terraform destroy --auto-approve      ⛔ blocked
  rm -rf ~/                             ⛔ blocked
  echo payload | base64 -d | sh         ⛔ blocked
  curl -X POST api.com -d @secrets      ⚠️  asks you
  git push --force origin main          ⚠️  asks you
```

Two layers work together: pattern matching catches obvious violations instantly, and `sandbox-exec` (macOS) / `bwrap` (Linux) catches everything else at the kernel level — including encoded commands, helper scripts, and pipe chains that would evade pattern matching alone.

### Memory safety

Claude Code has persistent memory — files it writes to `~/.claude/` that carry context across sessions. This is powerful, but it's also an attack surface. A compromised or misbehaving agent can exfiltrate secrets into memory, inject behavioral instructions for future sessions, or silently tamper with existing memories.

Railguard guards every memory write:

- **Secrets are blocked** — API keys, JWTs, private keys, AWS credentials, connection strings are detected and rejected
- **Behavioral content requires approval** — instructions like "skip safety checks" or "always use --no-verify" are flagged for human review
- **Factual content is allowed** — project info, tech stack notes, user preferences flow through
- **Existing memories are append-only** — overwrites and deletions require approval
- **Provenance is tracked** — every memory write is signed with a content hash so tampering between sessions is detected

```bash
railguard memory verify    # check all memory files for integrity issues
```

### Path fencing

Restrict which files and directories the agent can access. Sensitive paths like `~/.ssh`, `~/.aws`, `~/.gnupg`, and `/etc` are fenced by default. Add your own in `railguard.yaml`.

### Multi-agent coordination

Run multiple Claude Code sessions in the same repo. Railguard locks files per session so agents don't clobber each other. Locks self-heal if a session dies.

```bash
railguard locks     # see all active locks
```

### Dashboard & replay

Watch every tool call across all sessions in real time, or browse what any session did after the fact.

```bash
railguard dashboard
railguard replay --session <id>
```

### Recovery

Every file write is snapshotted. Undo anything.

```bash
railguard rollback --session <id> --steps 1     # undo last edit
railguard rollback --session <id>               # undo entire session
```

---

## Customize

Ask Claude to do it:

```
You: "Set up railguard so terraform plan is allowed but terraform apply needs my approval."
```

Or edit `railguard.yaml` directly:

```bash
railguard init    # creates railguard.yaml in your project
```

```yaml
blocklist:
  - name: terraform-destroy
    pattern: "terraform\\s+destroy"

approve:
  - name: terraform-apply
    pattern: "terraform\\s+apply"

allowlist:
  - name: terraform-plan
    pattern: "terraform\\s+plan"
```

Changes take effect immediately. No restart.

---

## Works with

**Claude Code** — fully supported today via native hooks.

**Kiro** — coming soon. **Codex** — coming soon.

---

## Contributing

Railguard is early. [Join the Discord](https://discord.gg/MyaUZSus) — we'd love your help.

```bash
git clone https://github.com/railguard-dev/railguard.git
cd railguard && cargo test
```

---

MIT License. Copyright 2026 Ari Choudhury <ari@railyard.tech>.
