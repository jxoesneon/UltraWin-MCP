# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | ✅ |
| < 1.0   | ❌                |

## Reporting a Vulnerability

**Please DO NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them responsibly:

### Reporting Process

1. **GitHub Security Advisories**: https://github.com/jxoesneon/ultrawin-mcp/security/advisories/new
2. **Provide**:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if available)

### What to Expect

- **Acknowledgment**: Within 48 hours
- **Assessment**: Within 7 days
- **Fix Timeline**: Critical issues within 14 days, others within 30 days
- **Disclosure**: Coordinated disclosure after fix is released

### Rewards

Currently no bug bounty program, but:

- Public acknowledgment in CHANGELOG
- Listed in SECURITY.md contributors
- Our eternal gratitude 🙏

## Security Measures

### Already Implemented ✅

1. **Authentication**

   - API key validation
   - Rate limiting (10 req/sec)
   - Client identification

2. **Input Sanitization**

   - Command whitelist
   - Windows path validation (preventing directory traversal)
   - Shell metacharacter removal for PowerShell/CMD

3. **Audit Logging**

   - All tool invocations logged
   - Security events tracked
   - 30-day retention

4. **Dependency Security**

   - Automated Dependabot scans
   - GitHub Actions security checks
   - Zero known vulnerabilities (`npm audit` clean)

5. **Windows Integration Security**
   - C# Bridge utilizes strong-named assemblies (where applicable)
   - Limited scope for UI Automation queries
   - UAC awareness and elevation checks

### Known Limitations ⚠️

1. **Local Access Required**

   - Server must run on the machine being automated
   - No remote access protections (by design)

2. **Screenshot Security**

   - Screenshots saved to `%TEMP%` or a secure user-specific folder
   - Recommendation: Use BitLocker to encrypt the system drive

3. **Action History**

   - Encrypted at rest using AES-256-GCM
   - Stored in secure `%USERPROFILE%\.ultrawin-mcp\history` directory
   - 7-day automated retention policy

4. **No Built-in HTTPS**
   - HTTP transport not encrypted by default
   - Recommendation: Use reverse proxy (IIS, nginx, Caddy)

## Security Best Practices

### For Administrators

1. **API Keys**

   - Generate strong keys: `openssl rand -hex 32` or PowerShell `[Convert]::ToBase64String((1..32 | % { [byte](Get-Random -Minimum 0 -Maximum 255) }))`
   - Rotate every 90 days
   - Never commit to version control
   - Use environment variables

2. **Filesystem**

   - Enable BitLocker (Windows)
   - Restrict directory permissions for the `.ultrawin-mcp` folder
   - Regularly review audit logs

3. **Network**

   - Use HTTPS reverse proxy
   - Windows Firewall rules (allow only necessary IPs/Ports)
   - VPN for remote access

4. **Monitoring**

   - Set up Prometheus alerts
   - Monitor audit logs daily
   - Track authentication failures

5. **Updates**
   - Apply Windows Updates promptly
   - Review security advisories weekly
   - Apply UltraWin MCP patches immediately

### For Developers

1. **Code Review**

   - All PRs require review
   - Security-sensitive changes (C# Bridge, path handling) need extra scrutiny
   - Run `npm audit` before committing

2. **Testing**

   - Maintain 95%+ test coverage
   - Include security tests (injection attempts, path traversal)
   - Test input sanitization across Windows-specific shell environments

3. **Dependencies**
   - Minimize dependencies
   - Review new dependencies
   - Monitor for vulnerabilities

## Security Audits

### Internal Audits

- **Frequency**: Quarterly
- **Scope**: Code review, dependency check, penetration testing
- **Last Audit**: 2025-12-22

### External Audits

- Not yet conducted
- Open to security researchers
- Contact us for collaboration

## Vulnerability Disclosure Policy

### Disclosure Timeline

1. **T+0**: Vulnerability reported
2. **T+48h**: Acknowledged
3. **T+7d**: Assessed and prioritized
4. **T+14-30d**: Fix developed and tested
5. **T+Release**: Security patch released
6. **T+Release+7d**: Public disclosure

## Security Checklist for Deployments

- [ ] Strong API key configured
- [ ] BitLocker encryption enabled
- [ ] HTTPS reverse proxy (if using HTTP transport)
- [ ] Windows Firewall rules configured
- [ ] Monitoring and alerting set up
- [ ] Audit logs reviewed regularly
- [ ] Dependencies up to date
- [ ] Incident response plan in place

## Incident Response

### In Case of Security Incident

1. **Contain**: Stop the service immediately
2. **Assess**: Review audit logs and Windows Event Logs
3. **Notify**: Report to administrators
4. **Remediate**: Apply fixes
5. **Learn**: Update security measures

## Compliance

### Standards

- ✅ OWASP Top 10 (2021)
- ✅ CWE/SANS Top 25
- ✅ NIST Cybersecurity Framework (Core)

## Contact

- **Security Issues**: GitHub Security Advisories
- **General Security Questions**: GitHub Issues (tag: security)

---

**Last Updated**: 2025-12-22
**Policy Version**: 1.0.0
