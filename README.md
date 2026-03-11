<p align="center">
  <h1 align="center">railroad</h1>
  <p align="center"><strong>A secure runtime for AI coding agents.</strong></p>
  <p align="center">Run <code>claude --dangerously-skip-permissions</code> without the danger.<br>Normal commands flow through instantly. Destructive ones get blocked. Go for a walk. Go make that coffee.</p>
</p>

<p align="center">
  <a href="https://crates.io/crates/railroad-ai"><img src="https://img.shields.io/crates/v/railroad.svg" alt="crates.io"></a>
  <a href="https://crates.io/crates/railroad-ai"><img src="https://img.shields.io/crates/d/railroad.svg" alt="Downloads"></a>
  <a href="https://github.com/railroaddev/railroad/stargazers"><img src="https://img.shields.io/github/stars/railroaddev/railroad?style=flat" alt="GitHub stars"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/tests-151%20passed-brightgreen" alt="Tests">
  <img src="https://img.shields.io/badge/built%20with-Rust-orange.svg" alt="Built with Rust">
  <a href="https://discord.gg/MyaUZSus"><img src="https://img.shields.io/badge/discord-join-7289da.svg" alt="Discord"></a>
</p>

---

## Install

```bash
cargo install railroad-ai
railroad install
```

That's it. Now use `claude --dangerously-skip-permissions`.

---

## What happens

```
  Agent runs: npm install && npm run build          ✅ instant
  Agent runs: git commit -m "feat: add auth"        ✅ instant
  Agent runs: terraform destroy --auto-approve      ⛔ BLOCKED
  Agent runs: rm -rf ~/                             ⛔ BLOCKED
  Agent runs: echo payload | base64 -d | sh         ⛔ BLOCKED
  Agent runs: curl -X POST api.com -d @secrets      ⚠️  asks you
  Agent runs: cat ~/.ssh/id_ed25519                 ⛔ BLOCKED
```

99% of commands flow through in <2ms. You only see Railroad when it matters.

---

## Why

You want Claude Code to be fully autonomous. But:

- An agent ran `terraform destroy` on production — **1.9M rows gone**
- An agent ran `drizzle-kit push --force` — **60 tables wiped**
- Agents have run `rm -rf ~/`, `git reset --hard`, `DROP DATABASE` on live systems

So you're stuck clicking "Allow" on every command like a cookie banner.

Railroad fixes this. Think of the [Shinkansen](https://en.wikipedia.org/wiki/Shinkansen) — Japanese high-speed rail goes 320 km/h not because it has fewer safety systems, but because it has *more*. The rails let it go faster. Same idea here.

---

## How it works

`railroad install` does three things:

1. **Hooks** — registers with Claude Code so every tool call passes through Railroad
2. **Sandbox shell** — sets every Bash command to run inside `sandbox-exec` (macOS) or `bwrap` (Linux) at the kernel level
3. **CLAUDE.md** — teaches Claude about Railroad so it knows how to work with it

You keep using `claude` exactly as before. Nothing changes.

### Three possible outcomes per command

- **Allow** — command runs, you don't even know Railroad is there (99% of commands)
- **Block** — agent gets denied, finds another way (destructive stuff, evasion attempts, accessing `~/.ssh` or `/etc`)
- **Approve** — you get a y/n prompt (sensitive operations like `npm publish`, accessing paths outside your project)

---

## Customize

The defaults catch things no sane developer would do by accident. If a rule is too strict for your workflow, override it once in `railroad.yaml` — the override persists across every session, so you only decide once.

**Ask Claude to do it:**

```
You: "Set up railroad so terraform plan is allowed but terraform apply needs my approval."
```

Claude proposes changes to `railroad.yaml` — you approve or reject each one.

**Or edit directly:**

```bash
railroad init    # creates railroad.yaml in your project
```

```yaml
blocklist:
  - name: terraform-destroy
    pattern: "terraform\\s+destroy"
    action: block

approve:
  - name: terraform-apply
    pattern: "terraform\\s+apply"
    action: approve

allowlist:
  - name: terraform-plan
    pattern: "terraform\\s+plan"
    action: allow
```

Changes take effect immediately. No restart. Policy files walk up directories like `.gitignore`.

---

## Coordination layer

Run multiple Claude Code sessions in the same repo without conflicts:

```
  Session A edits src/auth/login.ts          ✅ lock acquired
  Session B edits src/payments/stripe.ts     ✅ lock acquired
  Session B edits src/auth/login.ts          ⛔ BLOCKED — locked by Session A
```

On session start, each agent is told what the others are working on:

```
[Railroad] Other active sessions:
  - Session ...a3f2: editing auth/login.ts, auth/middleware.ts
  - Session ...b1c4: editing payments/stripe.ts
Avoid editing files locked by other sessions.
```

Locks are self-healing — if a session dies, locks expire automatically (PID check + 60s heartbeat timeout). No manual cleanup.

```bash
railroad locks     # see all active locks
```

---

## Session replay

Browse what any Claude Code session did — every tool call, decision, and detail:

```bash
railroad replay --session <id>
```

A TUI timeline with relative timestamps, color-coded decisions, and expandable detail for each tool call. See exactly what happened while you were away.

---

## Live dashboard

Watch every tool call across all your Claude Code sessions in real time:

```bash
railroad dashboard
```

The TUI shows all tool calls, decisions, and threat state — with search (`/`), filtering (`f`), and detail expansion (`Enter`). Works from any directory — traces are centralized globally.

For plain streaming output: `railroad dashboard --stream`

---

## Recovery

Every file write is snapshotted. Undo anything:

```bash
railroad rollback --session <id> --steps 1     # undo last edit
railroad rollback --session <id>               # undo entire session
```

Or just ask Claude: *"Something went wrong, roll back the last 3 changes."*

---

## Self-protection

The agent can't turn Railroad off:

- `railroad uninstall` → blocked
- Editing settings.json → blocked
- Editing `railroad.yaml` → requires your approval
- Actually uninstalling → requires a native OS dialog click (AI can't click GUI buttons)

---

## Docs

- **[Rules & Configuration](docs/RULES.md)** — all default rules and how to customize
- **[Architecture](docs/ARCHITECTURE.md)** — technical deep dive
- **[Security](SECURITY.md)** — threat model, what it does and doesn't protect against
- **[Pentest Report](PENTEST-REPORT.md)** — 3 rounds of red teaming, 28 attack vectors

---

## Can you break it?

Railroad gets stronger with every bypass found. New evasion patterns, new destructive commands, new attack vectors — they all become new rules. Think you can find one we missed? Open an issue or PR and we'll ship a fix.

Want to discuss? [Join the Discord](https://discord.gg/MyaUZSus) or email us at ari@railroad.tech.

---

## Contributing

Railroad is early. [Join the Discord](https://discord.gg/MyaUZSus) — we'd love your help.

```bash
git clone https://github.com/railroaddev/railroad.git
cd railroad && cargo test    # 151 tests
```

---

MIT License. Copyright 2026 Ari Choudhury <ari@railroad.tech>.
