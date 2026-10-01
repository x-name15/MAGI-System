# Contributing to magi-system

Thank you for considering a contribution. Please read this guide before submitting any pull request or issue.

## Contact

Questions or design discussions: open a [GitHub Issue](https://github.com/x-name15/magi-system/issues).
For security issues, see [SECURITY.md](SECURITY.md).

---

## Development Workflow

1. Fork the repository and clone it locally.
2. Ensure you have Docker / Docker Compose. The project uses Rust 1.90 with the `wasm32-unknown-unknown` target inside Docker.
3. Create a topic branch from `main` (e.g., `feature/custom-provider`).
4. Make your changes adhering to `docs/ARCHITECTURE.md`.
5. Run the full verification pipeline:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
6. Open a Pull Request against `main`.

---

## Commit Conventions

- Code, comments, and documentation must be in **English**.
- Commit messages: clear and specific, imperative mood, ≤72 chars.
- Follow Conventional Commits: `feat`, `fix`, `test`, `docs`, `chore`, `refactor`.
- Public APIs and Reducers must be documented with Rust doc comments (`///`).

---

## Pull Request Checklist

- [ ] Formatting (`cargo fmt`) passes
- [ ] Clippy checks (`cargo clippy`) pass with zero warnings
- [ ] All unit and integration tests pass (`cargo test --workspace`)
- [ ] New public structs, traits, or reducers are documented
- [ ] No hardcoded endpoints, models, or tokens
- [ ] Commit messages follow conventional commit style in English

---

Thank you for contributing to MAGI System.
