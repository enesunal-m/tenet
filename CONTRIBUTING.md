# Contributing

Thanks for taking the time to improve `tenet`.

## Development Setup

Requirements:

- Rust 1.85 or newer
- Git

Run the standard checks before opening a pull request:

```bash
cargo fmt --all --check
cargo test --locked --all-targets --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
```

Run docs locally:

```bash
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features
```

## Pull Requests

Keep pull requests focused. Include tests when behavior changes.

Update `README.md`, `spec.md`, or `CHANGELOG.md` when the user-facing behavior changes.

## Commit Style

Use clear imperative commit messages:

```text
Add missing lint rule coverage
Fix compile dry-run output
Document GitHub setup
```

## Security

Do not report security vulnerabilities in public issues. Follow `SECURITY.md`.

## License

By contributing, you agree that your contribution is licensed under the Apache License, Version 2.0.
