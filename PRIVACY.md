# Privacy Policy

**Last Updated**: 2025-12-22  
**Version**: 1.0.0

## Overview

UltraWin MCP ("the Service") is a desktop automation server that processes user commands to control Windows systems. This privacy policy explains what data is collected, how it's used, and your rights.

## Data Controller

jxoesneon/ultrawin-mcp  
GitHub: https://github.com/jxoesneon/ultrawin-mcp

## Data We Collect

### 1. Action History (Optional, Opt-In)

**What**: Record of automation commands executed
**Data includes**:

- Tool name (e.g., "mouseClick", "type")
- Timestamp
- Parameters (e.g., coordinates, text)
- Result status (success/failure)

**Storage**: Local filesystem (`%USERPROFILE%\.ultrawin-mcp\history\action-history.json`)  
**Retention**: Until manually deleted or 7 days (configurable)  
**Purpose**: Replay, debugging, audit trail

**Control**:

- Opt-in via configuration
- Can be disabled entirely
- Can be cleared at any time

### 2. Audit Logs

**What**: Security and operational logs
**Data includes**:

- Tool invocations with timestamps
- Authentication attempts
- Security events
- Error logs

**Storage**: Local filesystem (`%USERPROFILE%\.ultrawin-mcp\logs\`)  
**Retention**: 30 days (automatic rotation)  
**Purpose**: Security monitoring, compliance, troubleshooting

**Note**: Sensitive data (passwords, tokens) is automatically redacted

### 3. Metrics (Anonymous)

**What**: Prometheus metrics for monitoring
**Data includes**:

- Tool invocation counts
- Response times
- Error rates
- Cache sizes

**Storage**: In-memory only (not persisted)  
**Retention**: None (real-time only)  
**Purpose**: Performance monitoring, capacity planning

**Note**: No personally identifiable information (PII) is collected in metrics

### 4. API Keys

**What**: Authentication credentials
**Data includes**:

- API key hash (first 8 characters of SHA-256)
- Client name (if provided)
- Last used timestamp

**Storage**: In-memory only (not persisted to disk)
**Retention**: Until server restart
**Purpose**: Authentication and rate limiting

## Data We Do NOT Collect

- 🚫 Screen content (screenshots are only stored if explicitly requested)
- 🚫 Keyboard input content (only that input occurred, not what was typed by default)
- 🚫 OCR results (cached temporarily, automatically expire after 10 seconds)
- 🚫 Personal information (names, emails, etc.)
- 🚫 Usage analytics sent to third parties
- 🚫 Telemetry sent to external servers

## Data Sharing

**We DO NOT share your data with any third parties.**

All data is stored locally on your machine. No data is transmitted to external servers unless you explicitly configure integrations (e.g., Prometheus, Sentry).

## Your Rights (GDPR Compliance)

### Right to Access

Request a copy of all stored data:

```powershell
# View action history
Get-Content $HOME\.ultrawin-mcp\history\action-history.json

# View audit logs
Get-Content $HOME\.ultrawin-mcp\logs\audit.log
```

### Right to Erasure ("Right to be Forgotten")

Delete all stored data:

```powershell
# Clear action history
Remove-Item $HOME\.ultrawin-mcp\history\action-history.json

# Clear audit logs
Remove-Item -Recurse $HOME\.ultrawin-mcp\logs\
```

Or via API:

```typescript
// Clear action history
server.request("clear_action_history");
```

### Right to Data Portability

Export data in machine-readable format (JSON):

```powershell
Copy-Item $HOME\.ultrawin-mcp\history\action-history.json C:\path\to\export\
```

### Right to Rectification

Modify or update stored data directly:

```powershell
# Edit action history (JSON format)
notepad $HOME\.ultrawin-mcp\history\action-history.json
```

### Right to Restriction of Processing

Disable action logging entirely:

```powershell
# Set environment variable
$env:ULTRAWIN_MCP_DISABLE_LOGGING="true"
```

## Data Retention

| Data Type      | Default Retention     | Configurable      |
| -------------- | --------------------- | ----------------- |
| Action History | Until manual deletion | ✅ Yes            |
| Audit Logs     | 30 days               | ✅ Yes (maxFiles) |
| Metrics        | Real-time only        | 🚫 No             |
| API Keys       | Until restart         | 🚫 No             |
| OCR Cache      | 10 seconds            | ✅ Yes (TTL)      |

## Data Security

### Encryption

- **At Rest**: Action history and logs stored in plain text (filesystem permissions only)
- **In Transit**: HTTPS recommended for HTTP transport (not enforced by default)
- **In Memory**: API keys hashed, sensitive data redacted from logs

### Access Controls

- Filesystem permissions: User-only access recommended
- Container security: Non-root user (where applicable)
- Authentication: API key required

### Recommendations for Enhanced Security

1. Encrypt filesystem where data is stored
2. Use HTTPS reverse proxy for HTTP transport
3. Enable filesystem encryption (BitLocker on Windows)
4. Rotate API keys regularly (every 90 days)
5. Review audit logs periodically

## Third-Party Services (Optional)

If you configure external integrations, data may be shared:

| Service    | Data Shared       | Purpose      | Opt-In |
| ---------- | ----------------- | ------------ | ------ |
| Prometheus | Anonymous metrics | Monitoring   | Yes    |
| Sentry     | Error reports     | Bug tracking | Yes    |

**Important**: These integrations are opt-in and must be explicitly configured.

## Cookies

UltraWin MCP does not use cookies.

## Children's Privacy

This service is not intended for use by children under 13. We do not knowingly collect data from children.

## Changes to Privacy Policy

We will notify users of significant changes via:

- GitHub release notes
- CHANGELOG.md updates
- Version bump in privacy policy

## Contact & Data Protection Officer

For privacy concerns or data requests:

- **GitHub Issues**: https://github.com/jxoesneon/ultrawin-mcp/issues

## Compliance

This privacy policy complies with:

- ✅ GDPR (General Data Protection Regulation)
- ✅ CCPA (California Consumer Privacy Act) - where applicable
- ✅ Privacy by Design principles

## Open Source Transparency

UltraWin MCP is open source. You can:

- Review all code: https://github.com/jxoesneon/ultrawin-mcp
- Audit data collection
- Verify security claims
- Contribute improvements

## Legal Basis for Processing (GDPR)

- **Legitimate Interest**: Security logging, error tracking
- **Consent**: Action history (opt-in)
- **Contractual**: API authentication

## International Data Transfers

All data is stored locally on your machine. No international transfers unless you configure cloud deployments.

## Your Consent

By using UltraWin MCP, you consent to this privacy policy.

You can withdraw consent at any time by:

1. Stopping the service
2. Deleting all stored data
3. Uninstalling the software

---

**Questions?** Open an issue: https://github.com/jxoesneon/ultrawin-mcp/issues
