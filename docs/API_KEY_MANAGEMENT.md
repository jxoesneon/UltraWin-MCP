# API Key Management

## Overview

The UltraWin MCP server uses API keys for authentication and rate limiting. Each client must provide a valid API key to access the server.

## Setting Up API Keys

### Environment Variable (Recommended for Production)

Set the `ULTRAWIN_MCP_API_KEY` environment variable:

```powershell
$env:ULTRAWIN_MCP_API_KEY="uwcp_your_secure_key_here"
```

### Development Mode

If no API key is set and `NODE_ENV` is not `production`, a development key will be automatically generated and printed to the console.

?? **Never use development keys in production!**

## Generating API Keys

To generate a new API key programmatically:

```typescript
import { generateApiKey } from "./src/auth";

const newKey = generateApiKey("client-name");
console.log(`New API Key: ${newKey}`);
```

API keys have the format: `uwcp_<64-character-hex-string>`

## Using API Keys

### HTTP Header (Recommended)

```powershell
curl.exe -H "X-API-Key: uwcp_your_key_here" http://localhost:3000/...
```

### Query Parameter

```powershell
curl.exe "http://localhost:3000/...?apiKey=uwcp_your_key_here"
```

## Rate Limiting

- Each API key is limited to **10 requests per second**
- Requests exceeding the limit will be rejected
- Rate limit resets every second (sliding window)

## Key Rotation

To rotate an API key:

1. Generate a new key
2. Update all clients to use the new key
3. Revoke the old key

```typescript
import { generateApiKey, revokeApiKey } from "./src/auth";

// Generate new key
const newKey = generateApiKey("client-name");

// After clients updated, revoke old key
revokeApiKey(oldKey);
```

## Security Best Practices

1. **Never commit API keys to version control**
2. **Use environment variables for production keys**
3. **Rotate keys regularly** (every 90 days recommended)
4. **Use unique keys per client/environment**
5. **Monitor audit logs** for suspicious activity
6. **Revoke compromised keys immediately**

## Audit Logging

All authentication events are logged:

- Invalid API key attempts
- Rate limit violations
- Key generation/revocation
- See `~/.ultrawin-mcp/logs/audit.log` (resolves to `%USERPROFILE%\.ultrawin-mcp\logs\audit.log`)

## Monitoring

Check active API keys:

```typescript
import { listApiKeys } from "./src/auth";

const keys = listApiKeys();
console.log(keys); // Shows all active keys with metadata
```

## Troubleshooting

### "Invalid API key" errors

1. Verify key is correct (no extra spaces/characters)
2. Check if key was revoked
3. Verify environment variable is loaded (`echo $env:ULTRAWIN_MCP_API_KEY`)
4. Check audit logs for details

### Rate limit exceeded

1. Reduce request frequency
2. Implement client-side rate limiting
3. Consider increasing server rate limit if justified
4. Check if multiple clients share same key (not recommended)

## Example: MCP Client Configuration

```json
{
  "mcpServers": {
    "automation": {
      "command": "bun",
      "args": ["run", "C:/path/to/ultrawin-mcp/index.ts"],
      "env": {
        "ULTRAWIN_MCP_API_KEY": "uwcp_your_key_here"
      }
    }
  }
}
```
