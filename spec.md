# tenet — v0 specification

> Implementation spec for `tenet`, a structured context substrate for codebases. Compiles team-authored `.context/` files into nested `AGENTS.md` trees. This document is the source of truth for v0 implementation — when in doubt, it overrides the earlier design doc.

---

## 1. Purpose & scope

`tenet` replaces a single drifting `AGENTS.md` file with a structured directory of typed rule files. Humans author and review rules as discrete markdown files with metadata. `tenet compile` produces nested `AGENTS.md` files that every existing agent tool reads unchanged.

**In scope for v0**:
- Local CLI (`tenet`) for authoring, reviewing, and compiling rules.
- Deterministic compilation to nested `AGENTS.md` files.
- Linting and staleness detection.
- Pre-commit hook to keep compiled output in sync.
- Migration command to split an existing `AGENTS.md` / `CLAUDE.md` into typed files.

**Explicitly out of scope for v0**:
- MCP server, embeddings, semantic search.
- Cross-repo rule includes.
- Multi-target compile (`.cursorrules`, `copilot-instructions.md`). v0 generates `AGENTS.md` only.
- Web UI, dashboard, telemetry.
- Any network access at runtime. The v0 binary must be fully offline.

---

## 2. Data model

### 2.1 Rule file

A rule is a single markdown file under `.context/<type>/<slug>.md`.

The **rule ID** is `<type>/<slug>` (e.g., `invariants/auth-session-backend`). The ID is stable; renaming the file changes the ID and must be done via `tenet rename` (out of scope for v0 — filename changes in v0 require manual reference updates).

The file format is:

```
---
<optional YAML frontmatter>
---
<markdown body>
```

### 2.2 Rule types

The `<type>` segment is one of exactly five values:

| Type          | Meaning                                                                  |
|---------------|--------------------------------------------------------------------------|
| `invariants`  | Facts that must not change without team agreement.                       |
| `conventions` | How the team does things by default.                                     |
| `decisions`   | Dated decisions with reasoning.                                          |
| `gotchas`     | Pitfalls that bit someone and should not bite the agent.                 |
| `glossary`    | Domain terms and acronyms.                                               |

Files in any other subdirectory of `.context/` emit a lint warning and are ignored during compile.

### 2.3 Frontmatter schema

All fields are optional. Frontmatter itself is optional — a file with no frontmatter is valid and uses all defaults.

| Field       | Type                                      | Default | Validation                                            |
|-------------|-------------------------------------------|---------|-------------------------------------------------------|
| `scope`     | string (gitignore-style glob)             | `**`    | Must parse via `globset::Glob::new`.                  |
| `owner`     | string                                    | none    | Free-form; no validation.                             |
| `reviewed`  | string (ISO 8601 date, `YYYY-MM-DD`)      | none    | Must parse via `chrono::NaiveDate::parse_from_str`.   |
| `priority`  | enum: `high` / `normal` / `low`           | `normal`| Must match exactly, lowercase.                        |
| `tags`      | array of strings                          | `[]`    | Free-form.                                            |

Unknown fields produce a lint warning but do not fail parsing.

Invalid values produce a lint error. During compile, rules with invalid frontmatter are **skipped with an error printed to stderr** and compile exits 1.

### 2.4 Scope semantics

Scopes are gitignore-style globs, resolved relative to the repo root (the directory containing `.context/`).

A rule "applies to" a file path if `globset::Glob::compile_matcher(scope).is_match(path)` returns true.

**Special cases**:
- `scope: **` or omitted — applies everywhere.
- `scope: apps/bundle` — applies only to the file `apps/bundle` (likely not what the user wants; lint warns if scope has no glob metacharacters and the path is a directory).
- `scope: apps/bundle/**` — applies to any file under `apps/bundle/`.
- Absolute paths (leading `/`) — lint error. Scopes are always repo-relative.
- Symlinks — not followed during scope matching in v0.

### 2.5 Anchor directory

Every rule has an **anchor directory**, which is the deepest existing directory that is a literal prefix of the scope glob. This determines where the rule appears in the compiled output.

Algorithm to compute anchor:

1. Take the scope string. Replace any trailing `/**` with empty. Strip any trailing slash.
2. Split on `/`. Walk components left to right, keeping only those with no glob metacharacters (`*`, `?`, `[`, `{`).
3. Join the kept components. This is the anchor path (relative to repo root).
4. If anchor path is empty or `.`, anchor is the repo root.
5. If anchor path does not exist on disk, anchor is the repo root and a lint warning is emitted ("scope references missing directory").

Examples:

| Scope                           | Anchor                 |
|---------------------------------|------------------------|
| `**`                            | repo root              |
| `apps/bundle/**`                | `apps/bundle/`         |
| `apps/bundle/src/auth/*.rs`     | `apps/bundle/src/auth/`|
| `**/*.test.ts`                  | repo root              |
| `apps/*/src/**`                 | `apps/`                |

---

## 3. Directory layout

A tenet-initialized repo has this structure:

```
<repo-root>/
├── .context/
│   ├── invariants/
│   ├── conventions/
│   ├── decisions/
│   ├── gotchas/
│   └── glossary/
├── .tenetrc                    # project config (TOML)
├── AGENTS.md                   # generated, root-level
├── <other dirs>/AGENTS.md      # generated as needed, nested
└── .git/hooks/pre-commit       # installed by `tenet init`
```

`tenet init` creates the five subdirectories of `.context/` as empty directories (with a `.gitkeep` inside each so git tracks them), writes a default `.tenetrc`, writes an example rule, installs the pre-commit hook, and runs `tenet compile` to produce the initial `AGENTS.md`.

---

## 4. Configuration: `.tenetrc`

TOML file at repo root. Absent file = all defaults.

```toml
[defaults]
# Days after `reviewed` before a rule is considered stale.
grace_days = 90

# Default priority when frontmatter omits the field.
priority = "normal"

[compile]
# Comment inserted at the top of every generated AGENTS.md.
header = "<!-- generated by tenet from .context/ — do not edit directly -->"

# If false, stale rules are excluded from compile by default.
include_stale = true

# If true, move stale rules to a '## Needs review' section at the bottom.
# If false and include_stale is true, stale rules are mixed in with fresh ones.
segregate_stale = true

[lint]
# Scan rule files for obvious secret patterns.
check_secrets = true

# Warn on filenames that aren't lowercase-kebab-case.
check_filenames = true

[hook]
# If true, `tenet init` installs a pre-commit hook.
install_pre_commit = true
```

Parsing: strict. Unknown top-level sections emit lint errors. Unknown keys within known sections emit warnings. Type mismatches are errors. The binary ships with the above as its defaults; `.tenetrc` overrides individual keys.

---

## 5. CLI specification

Every command exits 0 on success, 1 on user-level error, 2 on internal error. `--help` and `--version` are handled by `clap` and exit 0.

### 5.1 `tenet init [--force] [--no-hook]`

Scaffolds `.context/` in the current directory.

**Behavior**:
1. Detect repo root by walking up to find `.git/`. If not found, error ("not inside a git repo") and exit 1.
2. If `.context/` already exists and `--force` is not set, error ("already initialized") and exit 1.
3. Create the five `.context/<type>/` subdirectories, each with a `.gitkeep` file.
4. Create `.tenetrc` with default contents (section 4).
5. Create `.context/invariants/example.md` with a helpful starter rule (see 5.1.1).
6. Unless `--no-hook`: install pre-commit hook (section 9).
7. Run `tenet compile` internally to produce the initial `AGENTS.md`.
8. Print summary to stdout.

**Exit codes**: 0 success, 1 already initialized or not in a git repo.

#### 5.1.1 Example rule content

```markdown
---
scope: "**"
priority: normal
---
Delete this file after reading it.

This is an example tenet. Tenets are short statements of truth about your
codebase that you want every AI agent session to know. Create new ones with
`tenet add <type>`.

Types:
- invariants: must-not-change facts
- conventions: team defaults
- decisions: dated choices with reasoning
- gotchas: pitfalls to avoid
- glossary: domain terms
```

### 5.2 `tenet add <type> [--scope GLOB] [--owner NAME] [--priority LEVEL] [--title TITLE]`

Creates a new rule file.

**Behavior**:
1. If `<type>` is not one of the five valid types, error and exit 1.
2. If stdout is a TTY and any of `--scope`, `--owner`, `--priority`, `--title` are not provided, prompt interactively via `dialoguer`.
3. Generate a filename slug from the title (lowercase, spaces to hyphens, strip non-ASCII-alphanumeric-plus-hyphen). If file already exists, append `-2`, `-3`, etc.
4. Write the file with frontmatter and an empty body.
5. Open `$EDITOR` (fall back to `vi` if unset) on the new file.
6. After editor closes, validate frontmatter. If invalid, print error and reopen editor. After 3 failed attempts, abandon the file (leave on disk, exit 1).
7. Print the rule ID to stdout.

**Non-interactive use** (for scripts and tests): all four flags must be provided; no prompts. The body is read from stdin if stdin is not a TTY.

**Exit codes**: 0 success, 1 invalid input or user abort, 2 internal error.

### 5.3 `tenet list [--type TYPE] [--scope PATH] [--owner NAME] [--stale] [--json]`

Lists rules matching the filters.

**Filters (AND-combined)**:
- `--type`: exact match on rule type.
- `--scope PATH`: list rules that would apply when an agent is editing `PATH`. (I.e., rules whose `scope` glob matches `PATH`.)
- `--owner`: exact match.
- `--stale`: only rules past `reviewed + grace_days`.

**Output**: a table with columns `ID`, `TYPE`, `SCOPE`, `OWNER`, `REVIEWED`. Grouped by type. Stale rules prefixed with `!`. With `--json`, output a machine-readable array of rule objects (full frontmatter plus computed fields `is_stale`, `anchor`).

**Exit codes**: 0 always (empty list is not an error).

### 5.4 `tenet show <rule-id>`

Prints the raw file contents including frontmatter.

**Exit codes**: 0 success, 1 not found.

### 5.5 `tenet edit <rule-id>`

Opens the file in `$EDITOR`. Validates on close as in `add`. Does not recompile automatically (user runs `tenet compile` separately, or the pre-commit hook catches it).

**Exit codes**: 0 success, 1 validation failed after retries or not found.

### 5.6 `tenet review <rule-id>`

Updates the `reviewed` field to today's date (system local time, ISO 8601). If no frontmatter exists, prepends one with just `reviewed:`. Does not open `$EDITOR`.

**Exit codes**: 0 success, 1 not found.

### 5.7 `tenet stale [--grace DAYS] [--owner NAME] [--json]`

Lists rules past their `reviewed + grace` date.

Default grace comes from `.tenetrc`. Rules with no `reviewed` field are never stale.

**Output**: grouped by owner (or "unowned" if empty). Each entry shows rule ID, type, reviewed date, days overdue.

**Exit codes**: 0 if no stale rules, 1 if any stale rules exist (useful in CI).

### 5.8 `tenet compile [--dry-run] [--exclude-stale]`

Regenerates the `AGENTS.md` tree. This is the core command. See section 6 for the algorithm.

**Behavior**:
1. Load all rule files from `.context/**/*.md`.
2. For each, parse frontmatter (lenient — files with bad frontmatter are skipped with error to stderr).
3. Compute anchor directory per rule.
4. Group rules by anchor.
5. For each anchor directory, render an `AGENTS.md` per section 6.2.
6. If any previously-generated `AGENTS.md` exists under an anchor that now has no rules, delete it.
7. Write files atomically (write to `.tmp`, rename).
8. With `--dry-run`: print the plan (which files would be written, deleted) and the first 10 lines of each new content, but do not touch disk.

**Identifying generated files** (critical for safety): an `AGENTS.md` is considered tenet-generated if its first line starts with the compile header from `.tenetrc`. `tenet compile` will never overwrite or delete a file that does not have this marker. If a hand-written `AGENTS.md` exists at a path tenet wants to write, it errors and exits 1 with a message pointing at the conflict.

**Determinism guarantee**: same inputs produce byte-identical outputs. Rule ordering within a file is specified in section 6.3.

**Exit codes**: 0 success, 1 conflict with hand-written file or invalid rule, 2 I/O error.

### 5.9 `tenet lint [--check-compiled] [--check-secrets]`

Runs all lint rules (section 7).

**Flags**:
- `--check-compiled`: after normal lint, regenerate compile output in memory, diff against what's on disk. Warn on drift.
- `--check-secrets`: scan rule files for secret patterns regardless of `.tenetrc` setting.

**Output**: `<severity>: <file>:<line>: <message>` for each finding, one per line. Exit code reflects highest severity.

**Exit codes**: 0 clean, 1 warnings only, 2 errors.

### 5.10 `tenet migrate --from PATH`

Reads an existing `AGENTS.md` / `CLAUDE.md` / other plain markdown and splits it into typed `.context/` files.

**Behavior**:
1. Parse the source file as markdown.
2. Walk top-level (H1/H2) sections. For each section with >0 content:
   - Show the section title and first 200 chars to the user.
   - Prompt: which type? (invariants/conventions/decisions/gotchas/glossary/skip).
   - Prompt: scope? (default `**`).
   - Write to `.context/<type>/<slug>.md`.
3. Do not delete the source file. Print next steps.

**Non-interactive mode**: if `--yes` is passed along with a mapping file `--mapping FILE` (a TOML file that maps section titles to types and scopes), run without prompts.

**Exit codes**: 0 success, 1 parse error, 2 I/O error.

### 5.11 `tenet version` / `tenet --version`

Prints `tenet <semver>`. Exit 0.

### 5.12 `tenet help [COMMAND]` / `tenet --help`

Standard clap help. Exit 0.

---

## 6. Compile algorithm

### 6.1 Planning phase

Input: the set of all parsed, valid rules.

For each rule:
1. Compute anchor directory per section 2.5.
2. Record: `anchor → [rules]`.

After all rules are bucketed, for each anchor directory:
1. Sort rules (section 6.3).
2. Render the `AGENTS.md` content (section 6.2).
3. Compute the target path: `{anchor}/AGENTS.md`.

Finally:
1. Walk existing generated `AGENTS.md` files (identified by the header marker). Any generated file whose path is not in the set being written is scheduled for deletion.
2. Execute writes and deletions. Writes are atomic (write-then-rename).

### 6.2 Rendered file format

Every generated `AGENTS.md` starts with exactly:

```
<!-- generated by tenet from .context/ — do not edit directly -->
<!-- last compiled: <ISO 8601 timestamp in UTC> -->
<!-- rules: <count> -->

# Project context
```

Then one `## <Type>` section per rule type that has rules at this anchor, in this fixed order:

1. Invariants
2. Conventions
3. Decisions
4. Gotchas
5. Glossary

Empty sections are omitted.

Within each section, rules are rendered as:

```
- **<first line of body>**  
  <remaining body, indented 2 spaces>
  
  <metadata footer, if any non-default fields: scope, owner, reviewed>
```

If the rule body's first line is a heading (`# Title`), that heading is stripped and its text becomes the bold lead.

Metadata footer format (single line, italicized, only non-default fields shown):

```
*scope: apps/bundle/** · owner: enes · reviewed: 2026-04-15*
```

Omit the footer entirely if all fields are default.

If the rule is stale and `segregate_stale` is true and `--exclude-stale` is not set, render in a `## Needs review` section at the bottom of the file instead, with each entry also showing `days overdue`.

At the very end of a non-root `AGENTS.md`, include a footer:

```
---
*Generated from `<relative path to .context/>`. Do not edit directly.*
```

The root `AGENTS.md` includes an additional "See also" section listing all other generated `AGENTS.md` files by path, so humans reading the root file can navigate.

### 6.3 Rule ordering

Within a section of a compiled file, rules are sorted by:

1. Priority: high → normal → low.
2. Within priority, alphabetical by rule ID (case-sensitive, Unicode codepoint order).

This is deterministic and does not depend on filesystem enumeration order (which varies by platform).

### 6.4 Edge cases

- **Empty `.context/`**: root `AGENTS.md` is written with only the header and a note "No rules defined." No other files are written.
- **Anchor dir was deleted since last compile**: if a previously-generated `AGENTS.md` exists at a path whose parent directory no longer exists, it is already gone — skip.
- **Compiled file exists but has hand-edited content** (no tenet marker): compile errors with a message listing the conflicting paths. User must either delete or move them.
- **Multiple rules at exactly the same ID**: impossible by filesystem. Two rules with similar titles produce different slugs.
- **Windows line endings in source files**: normalized to LF on parse. Generated output always uses LF.
- **UTF-8 BOM at start of source files**: stripped during parse.
- **Rule body contains markdown that collides with generator syntax** (e.g., starts with `##`): preserved verbatim under a bold lead. No escape processing.

---

## 7. Lint rules

Each rule has an ID, a severity (`warning` or `error`), and a description. Output format: `<severity>: <file>:<line>: <id>: <message>`.

| ID            | Severity | Description                                                                       |
|---------------|----------|-----------------------------------------------------------------------------------|
| `bad-frontmatter` | error    | Frontmatter is present but does not parse as YAML.                             |
| `bad-scope`   | error    | `scope` field is not a valid globset pattern.                                     |
| `abs-scope`   | error    | `scope` begins with `/`. Scopes must be repo-relative.                            |
| `bad-date`    | error    | `reviewed` does not parse as ISO 8601 date (`YYYY-MM-DD`).                        |
| `bad-priority`| error    | `priority` is not `high`, `normal`, or `low`.                                     |
| `unknown-field` | warning | Frontmatter contains a key not in the v0 schema.                                 |
| `unknown-type-dir` | warning | Subdirectory of `.context/` is not one of the five recognized types.          |
| `empty-body`  | warning  | Rule file has no markdown body after frontmatter.                                 |
| `bad-filename`| warning  | Filename is not lowercase-kebab-case (only enabled if `check_filenames = true`).  |
| `missing-dir` | warning  | Rule's scope references a directory that does not exist.                          |
| `secret-aws`  | warning  | Body contains a string matching the AWS access key pattern `AKIA[0-9A-Z]{16}`.    |
| `secret-github` | warning | Body contains a string matching the GitHub token pattern `ghp_[A-Za-z0-9]{36}`.  |
| `secret-pem`  | warning  | Body contains `-----BEGIN .* PRIVATE KEY-----`.                                   |
| `compiled-drift` | warning | `--check-compiled` finds drift between `.context/` and the generated tree.      |

Exit code is 0 if no findings, 1 if only warnings, 2 if any errors.

Secret patterns are intentionally minimal in v0 — false-positive cost is high and these three have very low FP rates. More patterns can be added via `.tenetrc` in v0.5.

---

## 8. Staleness

A rule is stale if and only if:

```
reviewed is present
AND reviewed + grace_days < today
```

`today` is the system local date. `grace_days` comes from `.tenetrc` (default 90) or `--grace` flag.

Rules with no `reviewed` field are **never stale**. This is intentional: review dates are opt-in. Timeless facts don't need review; only volatile ones do.

`tenet stale` groups stale rules by owner (or "unowned" if owner is empty). Output for each: rule ID, type, reviewed date, days overdue.

Compile behavior for stale rules is governed by `compile.include_stale` and `compile.segregate_stale` in `.tenetrc` (section 4). The `--exclude-stale` flag on `tenet compile` overrides `include_stale` to false.

---

## 9. Pre-commit hook

`tenet init` installs this script at `.git/hooks/pre-commit` (making it executable, mode 0755):

```bash
#!/bin/sh
# Installed by tenet. Ensures generated AGENTS.md files are in sync
# with .context/ whenever rule files are committed.

set -e

# Only run if .context/ or AGENTS.md files are staged.
if ! git diff --cached --name-only --diff-filter=ACMR | \
     grep -qE '^(\.context/|.*AGENTS\.md$)'; then
    exit 0
fi

# Verify compiled tree matches .context/.
if ! command -v tenet >/dev/null 2>&1; then
    echo "warning: tenet command not found; skipping compile check." >&2
    exit 0
fi

if ! tenet lint --check-compiled --quiet 2>/dev/null; then
    echo "error: .context/ is out of sync with generated AGENTS.md files." >&2
    echo "  fix: tenet compile && git add -A" >&2
    exit 1
fi

exit 0
```

**Behavior**:
- Silent when `.context/` and `AGENTS.md` files are not staged.
- Silent when `tenet` is not on PATH (developer may be cloning fresh; don't block them).
- Blocks commit when `.context/` is staged but generated files are stale.

**Installation concerns**:
- If `.git/hooks/pre-commit` already exists, `tenet init` appends our script with a marker comment rather than overwriting. `tenet uninstall-hook` removes our section cleanly.
- Windows: git for Windows runs `sh` from MSYS, so the shebang works. No separate Windows implementation needed for v0.

---

## 10. Error model

Two error kinds, serialized to exit codes:

| Kind | Exit | When                                                                  |
|------|------|-----------------------------------------------------------------------|
| User | 1    | Invalid input, not found, stale rules (for `stale`), conflict.        |
| Sys  | 2    | I/O failure, permission denied, malformed config, broken rule parse.  |

Internally: one `thiserror`-derived `TenetError` enum with variants per failure mode. `main` maps variants to exit codes and prints user-facing messages to stderr. Messages follow the form:

```
error: <short summary>
  <context line>
  <fix hint>
```

Example:

```
error: .context/ already exists in this repo
  path: /home/enes/myrepo/.context
  fix: use `tenet init --force` to overwrite
```

No stack traces in normal output. `RUST_BACKTRACE=1` enables them via `anyhow`.

---

## 11. Rust project structure

```
tenet/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE (Apache-2.0)
├── CHANGELOG.md
├── .context/                     # dogfood: tenet uses tenet
│   ├── invariants/
│   ├── conventions/
│   ├── decisions/
│   ├── gotchas/
│   └── glossary/
├── .tenetrc
├── AGENTS.md                     # generated
├── src/
│   ├── main.rs                   # thin entry, calls lib
│   ├── lib.rs                    # re-exports, error types
│   ├── cli.rs                    # clap definitions
│   ├── config.rs                 # .tenetrc parsing
│   ├── error.rs                  # TenetError enum
│   ├── cmd/
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── add.rs
│   │   ├── list.rs
│   │   ├── show.rs
│   │   ├── edit.rs
│   │   ├── review.rs
│   │   ├── stale.rs
│   │   ├── compile.rs
│   │   ├── lint.rs
│   │   └── migrate.rs
│   ├── rule/
│   │   ├── mod.rs                # Rule struct, load_all()
│   │   ├── frontmatter.rs        # parse frontmatter
│   │   ├── scope.rs              # anchor computation, matching
│   │   └── id.rs                 # rule ID <-> path
│   ├── compile/
│   │   ├── mod.rs                # orchestrator
│   │   ├── plan.rs               # bucket rules by anchor
│   │   ├── render.rs             # markdown generation
│   │   └── marker.rs             # detect generated files
│   ├── lint/
│   │   ├── mod.rs
│   │   ├── rules.rs              # each lint check
│   │   └── secrets.rs            # secret regex patterns
│   ├── hook/
│   │   ├── mod.rs
│   │   └── pre_commit.sh         # embedded via include_str!
│   └── util/
│       ├── mod.rs
│       ├── paths.rs              # repo root detection, etc.
│       └── atomic_write.rs       # write-then-rename
├── tests/
│   ├── init_test.rs
│   ├── add_test.rs
│   ├── compile_test.rs
│   ├── compile_nested_test.rs
│   ├── compile_overlap_test.rs
│   ├── compile_stale_test.rs
│   ├── lint_test.rs
│   ├── migrate_test.rs
│   ├── hook_test.rs
│   ├── determinism_test.rs
│   └── fixtures/
│       ├── simple_repo/
│       ├── nested_scopes/
│       ├── stale_rules/
│       ├── invalid_frontmatter/
│       ├── existing_agents_md/
│       └── conflict_handwritten/
└── .github/
    └── workflows/
        └── ci.yml
```

---

## 12. Dependencies

```toml
[package]
name = "tenet"
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
rust-version = "1.75"

[dependencies]
clap = { version = "4", features = ["derive"] }       # CLI
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"                                    # frontmatter
toml = "0.8"                                          # .tenetrc
globset = "0.4"                                       # scope matching
walkdir = "2"                                         # tree traversal
chrono = { version = "0.4", features = ["serde"] }    # dates
anyhow = "1"                                          # top-level errors
thiserror = "1"                                       # error types
regex = "1"                                           # secret detection
dialoguer = "0.11"                                    # interactive prompts
owo-colors = "4"                                      # terminal colors
similar = "2"                                         # diff for --check-compiled

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

**Justification for each direct dep**:

- `clap`: CLI parsing. Derive macros keep the surface terse. Alternative (writing by hand) is not worth the saved deps.
- `serde` + `serde_yaml`: frontmatter is YAML. `serde_yaml` is unmaintained since 2024 — acceptable for v0 but track a migration to `serde_yml` or similar in v0.5.
- `toml`: `.tenetrc` is TOML (Rust-native, well-supported).
- `globset`: battle-tested glob implementation used by `ripgrep`. Handles gitignore-style semantics correctly.
- `walkdir`: standard tree walker.
- `chrono`: date parsing and arithmetic. `time` is the main alternative; `chrono` is more ubiquitous.
- `anyhow` + `thiserror`: standard error-handling pair.
- `regex`: for secret patterns.
- `dialoguer`: interactive prompts. Used only in interactive commands.
- `owo-colors`: terminal colors. No tty dependency.
- `similar`: diff output for `--check-compiled`.

**No network crates.** `reqwest`, `hyper`, `ureq`, etc. must not appear in v0. `cargo deny` config enforces this.

**No git library.** Git interactions (detecting repo root, installing hooks, reading author) are done by shelling out to `git` or by manipulating `.git/` directly with filesystem primitives. Dropping `git2` / `gix` saves a large transitive dep tree and sidesteps libgit2 build issues.

Target transitive dep count: under 150 crates. Enforced in CI via `cargo tree | wc -l`.

---

## 13. Integration test plan

Each test uses `tempfile::TempDir`, initializes a git repo (`git init`), runs the `tenet` binary via `assert_cmd`, and asserts on file contents and exit codes.

**Test list**:

1. **init_test** — `tenet init` in a fresh git repo. Verify: `.context/` with five subdirs exists, each has `.gitkeep`, `.tenetrc` exists with expected defaults, pre-commit hook installed and executable, `AGENTS.md` generated with example rule, second `init` without `--force` exits 1.

2. **add_test** — `tenet add invariants --scope "**" --title "Test rule"` non-interactively. Verify: file created at expected path, frontmatter correct, exit 0. `add` with invalid priority exits 1.

3. **compile_test** — fixture with one invariant at scope `**`. Verify: root `AGENTS.md` contains the rule, no other `AGENTS.md` files exist, header marker present, second compile produces byte-identical output.

4. **compile_nested_test** — fixture with one invariant at scope `**` and one at `apps/bundle/**`. Verify: root `AGENTS.md` has only the first, `apps/bundle/AGENTS.md` has only the second.

5. **compile_overlap_test** — fixture with three rules: scope `**`, scope `apps/**`, scope `apps/bundle/**`. Verify: each `AGENTS.md` has only the rules anchored there, agent walking up from `apps/bundle/src/auth/foo.rs` would see all three.

6. **compile_stale_test** — fixture with one fresh rule and one rule where `reviewed` is 200 days ago (grace 90). Default compile: stale rule in `## Needs review` section. `--exclude-stale`: stale rule absent. `tenet stale` exits 1.

7. **lint_test** — fixture with: a file with malformed YAML, a file with `scope: /absolute`, a file with a GitHub token in body, a file in `.context/random/` (bad type dir). Verify: each produces expected lint finding, exit code 2 (errors present).

8. **migrate_test** — fixture with an `AGENTS.md` containing three H2 sections. Non-interactive migrate with a mapping file. Verify: three `.context/` files created, original `AGENTS.md` untouched.

9. **hook_test** — init a repo, add a rule, commit it, modify the rule without running compile, `git commit` should fail with the expected error message.

10. **determinism_test** — compile the same fixture ten times; all outputs byte-identical.

11. **conflict_handwritten_test** — fixture has a hand-written `apps/bundle/AGENTS.md` (no tenet marker) and a rule scoped there. `tenet compile` exits 1 with a message naming the conflict.

12. **marker_detection_test** — fixture has an `AGENTS.md` with a tenet header but outdated content. `compile` overwrites it. Another fixture has a file named `AGENTS.md` with hand-written content — `compile` refuses.

All tests run in `cargo test` with no network, no global state. CI runs on Linux, macOS, and Windows.

---

## 14. Edge cases & platform notes

- **Line endings**: all reads normalize CRLF → LF. All writes use LF. Tests run on Windows must tolerate git's autocrlf by disabling it in the test repo init.
- **Unicode in rule bodies**: preserved verbatim in output. Generated comments use ASCII.
- **Paths**: all internal representations are `PathBuf`. Paths are normalized to forward slashes when rendered in output (so AGENTS.md is portable across platforms).
- **Filesystem case sensitivity**: `tenet` is case-sensitive in rule IDs. On case-insensitive filesystems (macOS default, Windows), two rules with names that differ only in case are a lint error.
- **Symlinks inside `.context/`**: not followed. Walked as regular files; if the target is a dir, a lint warning is emitted ("symlink ignored").
- **Very long rule bodies**: no hard limit in v0. Lint warns if a single rule file exceeds 10 KB.
- **Locale for date formatting**: output is always ISO 8601 (`YYYY-MM-DD`). Input parsing accepts only ISO 8601.
- **System time**: `chrono::Local::now().date_naive()`. Tests that touch "today" use an injectable clock via a `TenetClock` trait with a fake in tests.

---

## 15. Implementation order

Recommended build order. Each step should produce a committable, testable state.

1. **Scaffold + CLI skeleton.** `cargo new tenet`, add `clap` deps, wire up subcommand stubs that print "not implemented" and exit 0. Write `--help` output.
2. **Config parsing.** Define `.tenetrc` struct, load-with-defaults, one unit test.
3. **Rule loading.** Define `Rule` struct, frontmatter parser, walk `.context/`. Unit tests for frontmatter parsing edge cases.
4. **Scope and anchor.** Implement anchor computation, glob matching. Unit tests for each case in section 2.5 table.
5. **Compile — write path.** Plan → render → atomic write. No marker detection yet. `compile_test` and `compile_nested_test` pass.
6. **Compile — marker detection and safety.** Never overwrite hand-written files. `conflict_handwritten_test` and `marker_detection_test` pass.
7. **Init.** Creates directories, writes `.tenetrc`, writes example rule, installs hook, runs compile. `init_test` passes.
8. **Add, show, edit, review.** Straightforward file ops. Tests for each.
9. **List and stale.** Filtering and output. `compile_stale_test` passes.
10. **Lint.** All 13 lint rules from section 7. `lint_test` passes.
11. **Hook.** The shell script (embedded), the installer, `hook_test` passes.
12. **Migrate.** Non-interactive mode first, interactive second. `migrate_test` passes.
13. **Polish.** Error messages, help text, README, CHANGELOG, first release.

This order is dependency-ordered: each step builds only on prior steps. A reasonable pace is 2–4 hours per numbered step for someone comfortable with Rust; v0 is roughly 2–3 weekends of focused work.

---

## 16. What is out of scope, deliberately

Do not implement any of the following in v0, even if tempted:

- MCP server. Adds complexity and a large dep. Separate v1 deliverable.
- Multi-target compile (`.cursorrules`, `copilot-instructions.md`). AGENTS.md is enough for v0; Cursor and Copilot users get a flat root file and that is fine.
- Cross-repo includes. Real gap, worth a separate design.
- Web UI, dashboard.
- Embeddings, semantic search over rules.
- Rename command. Rename by hand in v0.
- Team roles, capability scoping, permissioning.
- Automatic rule generation from agent sessions. Explicitly a non-goal.
- Network access of any kind. `cargo deny` enforces.

---

## 17. Open decisions to confirm before building

These are choices the spec makes that the implementer should validate are correct:

1. **YAML for frontmatter** vs. TOML. Picked YAML because it matches Jekyll/Hugo/Obsidian convention and devs recognize it. TOML would be more Rust-idiomatic but feels foreign in markdown files.

2. **TOML for `.tenetrc`** vs. YAML. Picked TOML because Rust convention (`Cargo.toml`, `rustfmt.toml`) and because config files benefit from stricter parsing.

3. **Shell out to git** vs. `git2`/`gix`. Picked shell-out for minimal deps. Cost: `git` must be on PATH (it always is, for users of this tool).

4. **Hand-written AGENTS.md collision → error**, not auto-merge. Auto-merge is a correctness hazard. Error is annoying but safe. User can use `--force` to overwrite.

5. **Generated files committed, not gitignored**. This keeps PR reviewers seeing what agents will see. Teams that prefer gitignored can do so manually; `tenet` does not opinionate.

6. **No version field in `.tenetrc`**. Simpler for v0. Add `version` in v0.5 when backward-compat matters.

If any of these feels wrong, push back before code is written; changing them after is more expensive.
