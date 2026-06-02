# API Versioning Strategy

## Overview

UltraWin MCP follows semantic versioning (SemVer) for API stability and backwards compatibility.

## Versioning Scheme

**Format**: \MAJOR.MINOR.PATCH\

- **MAJOR**: Breaking changes to MCP tool interfaces
- **MINOR**: New tools or backwards-compatible features
- **PATCH**: Bug fixes, performance improvements

**Current Version**: \1.0.0\

## Version Header

All MCP responses include version information:

\\\json
{
  "jsonrpc": "2.0",
  "result": {
    "version": "1.0.0",
    "data": {
      /* tool result */
    }
  }
}
\\\

## Breaking Changes Policy

### What Constitutes a Breaking Change

? **Breaking** (requires MAJOR bump):

- Removing a tool
- Removing a required parameter
- Changing parameter types
- Changing response structure
- Renaming tools or parameters

? **Non-Breaking** (MINOR/PATCH):

- Adding new tools
- Adding optional parameters
- Adding fields to responses
- Bug fixes
- Performance improvements

## Deprecation Process

1. **Announce** (Release N): Mark tool/parameter as deprecated

   \\\	ypescript
   server.addTool({
     name: "old_tool",
     description:
       "[DEPRECATED] Use new_tool instead. Will be removed in v2.0.0",
     // ...
   });
   \\\

2. **Support** (N ? N+2): Maintain for at least 2 minor versions

   - Log warnings when deprecated features used
   - Update documentation

3. **Remove** (Next MAJOR): Remove in next major version
   - Include migration guide in changelog

## Version Detection

Clients can query server version:

\\\	ypescript
// Via health endpoint
const health = await server.request("health");
console.log(health.version); // "1.0.0"

// Via package metadata
import { version } from "./package.json";
\\\

## Backwards Compatibility

### Tool Interface Stability

**Guaranteed Stable** (won't change without MAJOR bump):

- Tool names
- Required parameters
- Response schema structure
- Parameter types

**May Change** (MINOR/PATCH):

- Optional parameters (additions)
- Response fields (additions)
- Internal implementation
- Performance characteristics

### Example: Adding Optional Parameter

\\\	ypescript
// v1.0.0
server.addTool({
  name: "screenshot",
  schema: {
    properties: {
      mode: { type: "string" },
    },
    required: ["mode"],
  },
});

// v1.1.0 - Added optional parameter (backwards compatible)
server.addTool({
  name: "screenshot",
  schema: {
    properties: {
      mode: { type: "string" },
      quality: { type: "number" }, // NEW, but optional
    },
    required: ["mode"], // Same required fields
  },
});
\\\

## Version Changelog

### v1.0.0 (2026-06-02)

**Initial Release (UltraWin port)**

- 30+ automation tools
- Windows UI Automation (UIA) support
- WinRT OCR integration
- PowerShell service bridges
- API key authentication
- Health checks

See [CHANGELOG.md](../CHANGELOG.md) for full version history.

## Client Compatibility

### Minimum Supported Versions

| Client  | Minimum Version | Notes                |
| ------- | --------------- | -------------------- |
| FastMCP | 0.1.0+          | MCP protocol support |
| Node.js | 18+             | ESM support          |
| Bun     | 1.0+            | Native runtime       |

## Migration Guides

### Upgrading from 0.x to 1.0

**Breaking Changes:**

- Initial stable release of UltraWin.

**New Features:**

- Windows 10+ native ecosystem support.

## API Stability Guarantees

**Stable (v1.x):**

- ? All tool names
- ? All required parameters
- ? Response data structures
- ? Authentication mechanism

**Unstable (may change):**

- ?? Internal implementation details
- ?? Performance characteristics
- ?? Exact error messages (codes are stable)

## Experimental Features

Features marked \[EXPERIMENTAL]\ are subject to change without MAJOR version bump:

\\\	ypescript
server.addTool({
  name: "experimental_feature",
  description: "[EXPERIMENTAL] This API may change without notice",
  // ...
});
\\\

Use experimental features at your own risk in production.

## Support Policy

| Version | Support Status | End of Life |
| ------- | -------------- | ----------- |
| 1.x     | ? Active      | TBD         |
| 0.x     | ? Unsupported | 2026-06-02  |

**Support Includes:**

- Security updates
- Critical bug fixes
- Compatibility patches

## Version Metadata

Available in all responses:

\\\	ypescript
{
  version: "1.0.0",
  apiVersion: "1",
  protocol: "mcp",
  protocolVersion: "0.1.0"
}
\\\

## References

- [Semantic Versioning](https://semver.org/)
- [Model Context Protocol Spec](https://modelcontextprotocol.io)
