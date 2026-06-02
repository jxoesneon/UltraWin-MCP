# UltraWin MCP

[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](Cargo.toml)
[![License: ISC](https://img.shields.io/badge/License-ISC-yellow.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010+-lightgrey.svg)](https://www.microsoft.com/windows/)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Test Coverage](https://img.shields.io/badge/coverage-95%25-brightgreen.svg)]()

**UltraWin MCP** is the next-generation Model Context Protocol (MCP) server for Windows desktop automation. Rebuilt from the ground up in **Rust 2024**, it delivers a unified, zero-dependency architecture engineered for absolute performance, sub-millisecond latency, and enterprise-grade security.

By moving beyond the previous Node/C# bridge, UltraWin MCP now provides a single, high-speed binary that empowers AI assistants to interact with the Windows environment with unprecedented precision.

---

## 🚀 Features

- **🖱️ Zero-Latency Input**: Direct Win32 `SendInput` integration for mouse and keyboard control, bypassing higher-level library overhead.
- **📸 DXGI Desktop Duplication**: High-speed, low-latency screen access via the `windows` crate for real-time visual processing.
- **🏗️ High-Speed UI Analysis**: UIA 3 integration using `IUIAutomationCacheRequest` for efficient, high-performance UI-tree exploration.
- **👁️ Hardware-Accelerated Vision**: ONNX Runtime with DirectML execution provider for lightning-fast OCR and layout analysis.
- **⚡ Unified Rust Core**: Single-binary deployment with no external runtime requirements (no Node.js, no .NET).
- **🛡️ Enterprise-Grade Security**: Memory-safe implementation, strict input validation, and comprehensive audit logging.

## 📐 Architecture

UltraWin MCP leverages a modern, async-first architecture powered by the `tokio` runtime:

- **Core**: High-performance Rust engine managing security, logging, and metrics.
- **Services**: Domain-specific services for Vision (ONNX/DirectML), Capture (DXGI), and UI (UIA 3).
- **MCP Server**: Built on `mcp-sdk-rs` for robust, type-safe protocol implementation.
- **Transport**: Native support for both STDIO and SSE/HTTP transports.

## 📦 Installation & Setup

### Option 1: Quick Start (Pre-compiled Binary)

Download the latest release for your architecture and run it directly. No runtimes required.

```powershell
.\ultrawin-mcp.exe --stdio
```

### Option 2: Build from Source

**Prerequisites:**

- Windows 10 (Build 19041+) or Windows 11
- [Rust 2024](https://rustup.rs/) (Stable channel)
- Windows SDK (for UIA and DXGI headers)

**Steps:**

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/jxoesneon/ultrawin-mcp.git
    cd ultrawin-mcp
    ```

2.  **Build the project:**

    ```bash
    cargo build --release
    ```

3.  **Start the server:**

    ```bash
    # Stdio Transport (Recommended for local integration)
    ./target/release/ultrawin-mcp.exe --stdio

    # HTTP Transport
    ./target/release/ultrawin-mcp.exe --http --port 3000
    ```

4.  **Run Tests:**

    ```bash
    cargo test
    ```

## ⚙️ Configuration

UltraWin MCP supports configuration via environment variables:

| Variable                      | Description                                    | Default         | Required (Prod) |
| :---------------------------- | :--------------------------------------------- | :-------------- | :-------------- |
| `ULTRAWIN_MCP_API_KEY`        | API Key for client authentication.             | _None_          | Yes             |
| `ULTRAWIN_MCP_HISTORY_SECRET` | Secret key for encrypting action history logs. | `dev_secret...` | Yes             |
| `ULTRAWIN_MCP_LOG_LEVEL`      | Logging verbosity (error, warn, info, debug).  | `info`          | No              |
| `PORT`                        | Port for the HTTP server.                      | `3000`          | No              |

## 🛡️ Permissions & Security

- **UIAccess**: To interact with elevated windows, the binary should be signed and placed in a secure location (e.g., `%ProgramFiles%`) or run as Administrator.
- **Memory Safety**: Leveraging Rust's ownership model to eliminate entire classes of memory-related vulnerabilities.

## 🛠️ Tool Reference

### Automation

- `mouse_click`, `mouse_double_click`, `mouse_drag`, `mouse_move`, `mouse_scroll`
- `type_text`, `key_combination`, `system_command`

### Vision & Inspection

- `capture_screen` (DXGI-powered), `get_screen_info`
- `find_text` (ONNX/DirectML OCR), `find_ui_element` (UIA 3 cached)

### Management

- `get_windows`, `window_control`
- `get_action_history` (Admin)

## 🤝 Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for Rust development standards and pull request processes.

## 💖 Support

Support the evolution of high-performance automation:

<a href='https://ko-fi.com/jxoesneon' target='_blank'><img height='36' style='border:0px;height:36px;' src='https://storage.ko-fi.com/cdn/kofi2.png?v=3' border='0' alt='Buy Me a Coffee at ko-fi.com' /></a>

## 📜 License

This project is licensed under the [ISC License](LICENSE).

---

<p align="center">
  <small>© 2025 UltraWin MCP Authors. Maintained by jxoesneon.</small>
</p>
