# Contributing to UltraWin MCP

Thank you for your interest in contributing to the UltraWin MCP project. We are building a high-performance, Rust-based Model Context Protocol server for Windows.

## Development Environment

To contribute to this project, you will need:
- **Rust 2024 Edition**: Install via [rustup](https://rustup.rs/).
- **Windows 10/11**: For access to Win32, DXGI, and UIA3 APIs.
- **C++ Build Tools**: Required for compiling some dependencies.

## Tooling & Standards

We strictly adhere to Rust ecosystem standards to ensure code quality and maintainability.

### 1. Package Management
We use **Cargo** for all dependency management and build tasks.
```bash
cargo build
cargo run
```

### 2. Linting & Formatting
Before submitting a PR, ensure your code is formatted and passes all lints.
- **rustfmt**: Use `cargo fmt` to format your code.
- **clippy**: Use `cargo clippy` for static analysis. We aim for zero clippy warnings.

### 3. Testing
We use `tokio` for asynchronous tests. Ensure all tests pass before contributing.
```bash
cargo test
```

## Architectural Guidelines

- **Safety First**: Avoid `unsafe` blocks unless absolutely necessary for Win32/COM interop. Document all `unsafe` usage thoroughly.
- **Async Efficiency**: Use `tokio` for I/O bound tasks. For CPU-bound tasks (like image processing), offload to a dedicated thread pool.
- **Windows APIs**: Use the `windows` crate for all OS interactions. Avoid wrapping C++ libraries if a native Rust implementation or direct Win32 call is possible.
- **MCP SDK**: Follow the patterns established in `mcp-sdk-rs`.

## Contribution Workflow

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/amazing-feature`).
3. Implement your changes.
4. Run `cargo fmt` and `cargo clippy`.
5. Run `cargo test`.
6. Commit your changes.
7. Push to the branch and open a Pull Request.

## Performance Considerations

Since this is a "Unified Rust Architecture" project, always consider the performance implications of your changes. Minimize cross-process calls and avoid unnecessary data marshaling.
