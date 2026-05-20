# SECURITY.md

## Reporting a Vulnerability

If you discover a security vulnerability, please do **not** open a public issue.

Instead, send a description of the issue to the project maintainer directly.  
We will respond within 7 days and work on a fix before disclosing publicly.

## Security Practices

- **JWT secrets** — must be randomly generated and kept secret in production
- **Default credentials** — change the default `admin` password immediately after first login
- **SQL injection** — `sort_by` uses an allowlist; all queries use parameterized SQLx bindings
- **Password storage** — Argon2id hashing via `argon2` crate
- **Dependencies** — review `cargo audit` for Rust and `npm audit` for Node.js before release
