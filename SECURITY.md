# Security Policy

## Supported Versions

Security fixes target the latest released version of `tenet`.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

## Reporting a Vulnerability

Please report suspected vulnerabilities privately through GitHub Security Advisories:

https://github.com/enesunal-m/tenet/security/advisories/new

If private vulnerability reporting is not enabled on the repository, contact the maintainers through a private channel and include:

- affected version or commit
- operating system
- reproduction steps
- expected impact
- any known workaround

Please do not open a public issue for a suspected vulnerability.

## Security Scope

`tenet` is intended to run fully offline at runtime. Security-sensitive areas include:

- rule parsing and generated file safety
- protection against overwriting hand-written `AGENTS.md`
- pre-commit hook installation behavior
- secret-pattern linting
- dependency supply-chain hygiene

## Disclosure

Maintainers will acknowledge valid reports as soon as practical, investigate privately, and coordinate a fix and release before public disclosure.
