# Contributing to diself

Thanks for your interest in contributing! This document outlines how to get started.

## Getting Started

1. Fork the repository and clone your fork
2. Create a new branch from `master` for your changes
3. Make sure everything compiles and passes before submitting

```bash
cargo check
cargo clippy --all-targets --all-features
cargo test
```

## How to Contribute

### Reporting Bugs

Open an issue with:

- A clear title and description
- Steps to reproduce the problem
- Expected vs actual behavior
- Rust version (`rustc --version`) and OS

### Suggesting Features

Open an issue with the `enhancement` label. Describe:

- The use case (what you're trying to do)
- The proposed API or behavior
- Any alternatives you've considered

### Submitting Pull Requests

1. **Open an issue first** for non-trivial changes so we can discuss the approach
2. **Keep PRs focused** -- one feature or fix per PR
3. **Include tests** when adding new functionality or fixing a bug
4. **Follow existing code style** -- match the patterns you see in the codebase
5. **Update examples** if your change affects the public API

## Code Guidelines

### Style

- Follow standard Rust conventions (`rustfmt` defaults)
- Use `impl AsRef<str>` / `impl Into<String>` for string parameters in public APIs
- Prefer `Result<T>` over `.unwrap()` / `.expect()` in library code
- Use `tracing` macros (`tracing::info!`, `tracing::warn!`) for logging, not `println!`

### Architecture

- **`src/http/`** -- HTTP client, request building, response handling
- **`src/gateway/`** -- WebSocket connection, heartbeat, reconnect logic
- **`src/client/`** -- High-level client, context, event handler, managers
- **`src/model/`** -- Discord data structures and builders
- **`src/cache/`** -- In-memory cache for Discord entities
- **`src/error.rs`** -- Error types

### Testing

- Unit tests go in the same file as the code (`#[cfg(test)] mod tests`)
- Integration tests go in `tests/`
- Live endpoint tests go in `tests/endpoints_live.rs` and must be `#[ignore]`d
- Run the full suite before submitting: `cargo test`

### Commit Messages

- Use clear, concise messages describing the *why*, not just the *what*
- Prefix with a category when relevant: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`

Examples:

```
feat: add EmbedBuilder for fluent embed construction
fix: retry rate-limited requests with request_delay
refactor: extract build_request() to deduplicate HTTP headers
```

## What's In Scope

Good first contributions:

- Adding missing Discord API endpoints to managers
- Improving test coverage
- Fixing clippy warnings
- Documentation improvements
- Adding examples

Currently out of scope (tracked for future releases):

- Voice gateway
- File upload / multipart
- Webhook management
- Bot-specific interaction handling

## Code of Conduct

This project follows a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold it.

## Questions?

Open an issue or start a discussion. We're happy to help you find a good first contribution.
