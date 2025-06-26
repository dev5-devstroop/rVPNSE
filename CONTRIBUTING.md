# Contributing to SoftEther VPN Rust

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Development Setup

### Prerequisites
- Rust 1.70+ with `cargo`
- Platform-specific dependencies (see README.md)
- Git for version control

### Getting Started
```bash
# Clone the repository
git clone <repo>
cd rvpnse

# Install development dependencies
cargo install cargo-fmt cargo-clippy

# Build and test
cargo build
cargo test
cargo fmt
cargo clippy
```

## Code Standards

### Rust Guidelines
- Follow [Rust's official style guide](https://doc.rust-lang.org/style-guide/)
- Use `cargo fmt` for consistent formatting
- Run `cargo clippy` and resolve all warnings
- Write comprehensive documentation with `rustdoc`
- Use `async/await` for all I/O operations

### Error Handling
- Use `anyhow` for error handling throughout the codebase
- Provide meaningful error messages with context
- Handle all `Result` types explicitly

### Testing
- Write unit tests for all public functions
- Use integration tests for end-to-end scenarios
- Property-based testing for protocol parsing
- Mock external dependencies in tests

## Contribution Process

### Issues
- Check existing issues before creating new ones
- Use issue templates when available
- Provide detailed reproduction steps for bugs
- Include system information (OS, Rust version, etc.)

### Pull Requests
- Fork the repository and create feature branches
- Write clear, descriptive commit messages
- Include tests for new functionality
- Update documentation as needed
- Ensure CI passes before requesting review

### Branch Naming
- `feature/description` for new features
- `fix/description` for bug fixes
- `docs/description` for documentation updates

## Code Review
- All changes require review before merging
- Address reviewer feedback promptly
- Maintain a respectful and constructive tone
- Break large changes into smaller, reviewable chunks

## Release Process
- Follow semantic versioning (SemVer)
- Update CHANGELOG.md for all releases
- Tag releases in Git
- Publish to crates.io for library releases

## Security
- Report security vulnerabilities privately
- Follow responsible disclosure practices
- Security fixes take priority over other changes

## Community
- Be respectful and inclusive
- Help newcomers get started
- Follow the Code of Conduct
- Participate in discussions constructively
