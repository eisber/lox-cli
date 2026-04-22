# API Design Guidelines

Rules for adding or modifying CLI commands. Derived from clig.dev, gh, docker, kubectl patterns, and the #46 UX audit.

## Command Grammar

Commands follow three patterns depending on context:

| Pattern | When to use | Examples |
|---------|------------|---------|
| `lox <verb> <target>` | Generic actions that work on any control type | `lox on "X"`, `lox off "X"`, `lox get "X"` |
| `lox <noun> <target> <action>` | Device-specific commands with typed actions | `lox blind "X" up`, `lox gate "X" open`, `lox door "X" lock` |
| `lox <noun> <subcommand>` | Resource management / multi-operation groups | `lox config download`, `lox token fetch`, `lox light mood "X" plus` |

**Rules:**
- New device-type commands → pattern 2 (`lox <device> <target> <action>`)
- New admin/management commands → pattern 3 (`lox <noun> <subcommand>`)
- Only add pattern 1 for truly universal operations (on/off/get are the only ones)

## Global Flags

These flags are available on every command:

| Flag | Short | Purpose |
|------|-------|---------|
| `--output <format>` | `-o` | Output format: `table` (default), `json`, `csv` |
| `--quiet` | `-q` | Suppress confirmation messages (✓/✗/▶) |
| `--no-color` | — | Disable colored output (also respects `NO_COLOR` env var) |
| `--no-header` | — | Suppress table column headers |
| `--help` | `-h` | Show help |
| `--version` | `-V` | Show version |

**Never** add a global flag for something that only applies to one command.

## Naming Conventions

### Commands
- **Lowercase single words**: `status`, `discover`, `reboot`
- **Subcommands**: `config download`, `token fetch`, `light dim`
- **Device nouns are singular**: `blind`, `gate`, `alarm`, `door` (you act on ONE device)
- **Listing commands are plural**: `rooms`, `categories`, `modes`, `extensions` (you list MANY)
- **Use `ls`** (not `list`) for listing subcommands: `alias ls`, `scene ls`, `config ls`
  - Keep `list` as a hidden alias for discoverability

### Flags
- **Short flags** only for frequently-used options: `-r` (room), `-t` (type), `-o` (output), `-q` (quiet), `-y` (yes), `-n` (lines), `-i` (interval)
- **Long flags** use kebab-case: `--save-as`, `--dry-run`, `--no-motion`, `--all-in-room`
- **Boolean negation** uses `--no-` prefix: `--verify-ssl` / `--no-verify-ssl`, `--no-color`, `--no-header`
- **Never** use `--flag <bool>` (requiring `true`/`false` as an argument) — use flag pairs instead
- **Dangerous operations** use `--yes`/`-y` (to skip confirmation) or `--force` (to override safety)
  - `--yes` = "skip the interactive prompt" (reboot, update install)
  - `--force` = "I know this is dangerous, do it anyway" (config upload)

### Arguments
- **Positional** for the primary target and action: `lox blind "Beschattung" up`
- **Flags** only when there are multiple optional parameters on the same command (rare)
- **Room disambiguation** uses `-r`/`--room` on every command that resolves a control name

## Output

### Formats
- **Table** (default): Human-readable with column headers, `─` dividers, Unicode status indicators (✓/✗/▶)
- **JSON**: Pretty-printed, complete data. Use for scripting and AI agent integration
- **CSV**: Tab/comma-delimited, no decorations. Use for data export and piping

### Conventions
- Default to **table** output — optimize for humans first
- JSON output should include **more fields** than the table view (UUIDs, types, etc.)
- Table headers use **UPPERCASE**: `NAME`, `TYPE`, `ROOM`, `UUID`
- Respect `--no-header` to suppress the header row and divider line
- Use `✓` for success, `✗` for errors, `▶` for "starting an action" — suppress all with `--quiet`
- End table output with a count line: `\n42 controls`

### Status/confirmation messages
- Success: `✓  control → action = result`
- Error: `bail!("descriptive message with context")`
- Suppress with `--quiet` — never suppress errors

## Adding a New Command

1. Add the enum variant to `Cmd` in the appropriate section (`Control`, `Inspect`, `System`, `Configuration`)
2. Add `-r`/`--room` if the command resolves a control name
3. Support `-o json` output — always
4. Support `--quiet` for confirmation messages
5. Support `--no-header` if the command outputs a table
6. Update the help template grouping at the top of `Cli`
7. Update `COMMANDS.md` with the full syntax and examples
8. Update `README.md` command overview if it's a significant addition

## Environment Variables

Config fields support env vars with `LOX_` prefix:

| Env var | Config field |
|---------|-------------|
| `LOX_HOST` | `host` |
| `LOX_USER` | `user` |
| `LOX_PASS` | `pass` |
| `LOX_SERIAL` | `serial` |
| `NO_COLOR` | Disable colored output |

## Error Messages

Follow the three-component pattern:
1. **Context**: What were you trying to do?
2. **Error**: What went wrong?
3. **Fix**: How do you fix it?

```rust
// Good
bail!("'{}' is type '{}', not a LightController. Use `lox ls -t LightController` to find light controls.", name, typ);

// Bad
bail!("Invalid control type");
```

## Shell Completions

`lox completions <shell>` generates completions for bash, zsh, and fish via `clap_complete`. When adding new commands or flags, completions update automatically from the clap definitions.

## Backwards Compatibility

When renaming a command:
1. Add `#[command(alias = "old_name")]` to preserve the old form
2. Add `#[command(hide = true)]` to remove the old form from `--help`
3. Document the migration in CHANGELOG.md
