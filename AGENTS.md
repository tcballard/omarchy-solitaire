# Contribution workflow

Use a feature or fix branch and open a pull request for subsequent changes.
Do not push directly to `main` unless Tom explicitly authorizes that particular
push. The initial repository import had a one-time direct-push approval.

Keep judgment calls in `docs/DECISIONS.md`. Record verification accurately,
distinguishing headless Rust input checks, Arch package checks and real Omarchy desktop
acceptance. Do not call a pending CI job a pass.

Rust is the default implementation language. Run cargo fmt --check, cargo clippy
--locked --all-targets -- -D warnings, and cargo test --locked before handoff.
Preserve Python save compatibility, the stable seeded shuffle and official logo.
