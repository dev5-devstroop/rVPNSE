# GitHub Workflows

This directory contains simplified GitHub Actions workflows following best practices for Rust projects.

## Workflows

### 🔄 CI (`ci.yml`)
**Triggers:** Push to main/develop, Pull Requests
- **Purpose:** Main continuous integration pipeline
- **Jobs:**
  - **Test Suite:** Cross-platform testing (Ubuntu, macOS, Windows)
  - **Code Quality:** Formatting, linting, documentation checks
  - **Coverage:** Code coverage reporting with codecov

### 🚀 Release (`release.yml`)
**Triggers:** Git tags (`v*`), Manual dispatch
- **Purpose:** Create GitHub releases with binary artifacts
- **Jobs:**
  - **Create Release:** Initialize GitHub release
  - **Build:** Cross-platform binary builds for major targets

### 📊 Benchmarks (`benchmarks.yml`)
**Triggers:** Push to main (with bench changes), Weekly schedule, Manual
- **Purpose:** Performance regression testing
- **Jobs:**
  - **Benchmark:** Run cargo bench and track performance over time

### 🔒 Security (`security.yml`)
**Triggers:** Push to main, Pull Requests, Weekly schedule
- **Purpose:** Security vulnerability scanning
- **Jobs:**
  - **Security Audit:** Check dependencies for known vulnerabilities

## Key Improvements

✅ **Simplified Logic:** Removed complex conditional branching  
✅ **Fast Feedback:** Essential checks run first  
✅ **Clear Separation:** Each workflow has a single, clear purpose  
✅ **Best Practices:** Uses community-standard actions  
✅ **Efficient Caching:** Proper dependency caching with Swatinem/rust-cache  
✅ **Cross-Platform:** Reasonable matrix testing without over-engineering  

## Local Development

To run similar checks locally:

```bash
# Code quality checks
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo doc --no-deps

# Testing
cargo test
cargo test --release

# Security
cargo install cargo-audit
cargo audit

# Benchmarks
cargo bench
```

## Migration Notes

The previous workflows (`test.yml`, `benchmark.yml`, `changelog.yml`) were overly complex with:
- Unnecessary conditional logic
- Complex matrix strategies  
- Redundant optimizations
- Poor maintainability

The new structure is simpler, faster, and follows GitHub Actions best practices while maintaining all essential functionality.
