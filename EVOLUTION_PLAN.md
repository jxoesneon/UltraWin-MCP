# UltraWin MCP Evolution Plan: From Parity to Performance

## The Shift: UltraMac-Parity to Rust-Evolution

The initial phase of UltraWin MCP focused on achieving parity with the UltraMac implementation. However, to truly leverage the Windows ecosystem, we are evolving into a native Rust-based architecture. This shift isn't just about changing languages; it's about redefining performance and reliability for Model Context Protocol servers on Windows.

## Key Architectural Pillars

### 1. "Zero Marshaling" Overhead
By adopting a Unified Rust Architecture, we eliminate the expensive marshaling between different runtime environments. Using the `windows` crate directly, we interact with COM and Win32 APIs with native efficiency, ensuring that data stays in-process and high-performance.

### 2. DXGI Desktop Duplication
For visual context and screen analysis, we are moving to DXGI Desktop Duplication. This provides low-latency, high-frame-rate access to the desktop buffer, far surpassing traditional GDI or BitBlt methods. This is critical for real-time AI visual reasoning.

### 3. Modern UIA3 + CacheRequests
Accessibility is the backbone of our UI understanding. By utilizing UI Automation 3 (UIA3) with `CacheRequests`, we minimize the number of cross-process calls required to fetch element properties. This allows us to snapshot the UI state in a single operation, drastically reducing latency.

### 4. ONNX + DirectML
Local AI acceleration is handled via ONNX Runtime with the DirectML execution provider. This ensures optimal performance across the diverse range of Windows hardware, from integrated GPUs to dedicated NVIDIA/AMD cards, without requiring CUDA.

### 5. Rust 2024 Foundations
We are building on the latest Rust 2024 edition, leveraging modern language features for better concurrency, safety, and developer experience.

## Roadmap

- **Phase 1: Core Migration** - Porting mcp-sdk-rs integration and basic tool set.
- **Phase 2: Visual Engine** - Implementing DXGI pipeline and DirectML integration.
- **Phase 3: Semantic UI** - UIA3 optimization and CacheRequest implementation.
- **Phase 4: Optimization** - Profiling and "Zero Marshaling" verification.
