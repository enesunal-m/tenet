# tenet

`tenet` compiles team-authored `.context/` rule files into nested `AGENTS.md` files.

## Commands

- `tenet init`
- `tenet add <type>`
- `tenet list`
- `tenet show <rule-id>`
- `tenet edit <rule-id>`
- `tenet review <rule-id>`
- `tenet stale`
- `tenet compile`
- `tenet lint`
- `tenet migrate`

## Development

```bash
cargo fmt --check
cargo test
cargo clippy -- -D warnings
```
