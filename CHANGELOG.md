# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive benchmarking infrastructure with Criterion.rs
- Multi-platform CI/CD pipeline with GitHub Actions
- Automated performance regression detection
- FFI interface benchmarking suite
- Configuration parsing and validation benchmarks
- Client lifecycle operation benchmarks
- HTML, JSON, and CSV benchmark reporting
- Automated changelog generation and management
- Release automation with multi-platform binary builds
- Code coverage reporting with codecov integration
- Static analysis with clippy and cargo-audit
- Memory safety verification with Miri
- Binary size monitoring and reporting

### Performance
- Optimized CI/CD workflows to reduce redundant runs
- Resource-efficient test and benchmark execution
- Conditional full test suite execution based on change scope
- Smart caching for build dependencies and artifacts

### Documentation
- Complete benchmarking guide and best practices
- CI/CD pipeline documentation
- Release process documentation
- Contributing guidelines and development setup

## [0.1.0] - 2025-06-25

### Added
- Initial rVPNSE static library implementation
- Basic VPN client functionality
- Configuration management system
- FFI interface for C integration
- Core networking and protocol handling
- Basic test suite
- Project structure and build system
