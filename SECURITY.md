# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Currently supported versions:

| Version | Supported          |
| ------- | ------------------ |
| 2.9.x   | :white_check_mark: |
| 2.8.x   | :white_check_mark: |
| < 2.8   | :x:                |

## Reporting a Vulnerability

We take the security of Hakanai seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### Please do NOT:
- Open a public GitHub issue
- Post about it on social media
- Exploit the vulnerability beyond necessary verification

### Please DO report via one of these channels:

#### Option 1: GitHub Security Advisory (Recommended)
- Go to the [Security tab](https://github.com/czerwonk/hakanai/security) in our repository
- Click "Report a vulnerability"
- Fill out the private security advisory form
- This creates a private discussion that only maintainers can see

#### Option 2: Direct Contact
- Contact the maintainer [@czerwonk](https://github.com/czerwonk) directly through GitHub
- Include "SECURITY" in your message title
- PGP key available at [https://github.com/czerwonk.keys](https://github.com/czerwonk.keys) for encrypted communication

### What to include in your report:
- Affected version(s)
- Steps to reproduce the issue
- Potential impact
- Suggested fix (if available)

### What to expect:
- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Timeline**: Typically within 30 days for critical issues

### After we receive your report:
1. We will confirm receipt and begin investigation
2. We will work to verify and reproduce the issue
3. We will prepare a fix and coordinate disclosure
4. We will credit you in the security advisory (unless you prefer to remain anonymous)

## Security Considerations

### Architecture Security
Hakanai implements a zero-knowledge architecture where:
- All encryption/decryption happens client-side
- Server never has access to plaintext data or encryption keys
- Secrets are automatically deleted after first access
- Mandatory content integrity verification via SHA-256 hashes

### Cryptographic Implementation
- **Algorithm**: AES-256-GCM (authenticated encryption)
- **Key Generation**: Cryptographically secure random generation
- **Nonce**: Unique per encryption operation
- **Hash Verification**: SHA-256 truncated to 128 bits (tradeoff between security and URL practicality)
  - 128-bit truncation provides 2^64 collision resistance
  - Balances strong tamper detection with manageable URL length and QR code scannability

### Deployment Security

#### Required Infrastructure Security (Your Responsibility)
- **TLS/HTTPS**: Must be terminated at reverse proxy
- **Rate Limiting**: Implement at reverse proxy level
- **DDoS Protection**: Use appropriate infrastructure protection
- **Firewall Rules**: Restrict Redis access to application only

#### Application Security (Built-in)
- **Authentication**: Token-based with SHA-256 hashing
- **Input Validation**: UUID format validation, size limits
- **CORS Policy**: Restrictive by default
- **Security Headers**: X-Frame-Options, X-Content-Type-Options, HSTS
- **Memory Safety**: Sensitive data zeroized after use

### Best Practices for Users

#### For Administrators
1. Always run behind a reverse proxy with TLS
2. Configure rate limiting based on your usage patterns
3. Regularly update to the latest version
4. Monitor logs for suspicious activity
5. Restrict Redis network access
6. Use strong admin tokens (if admin API enabled)
7. Set appropriate size limits for your use case

#### For End Users
1. Use the `--separate-key` option for highly sensitive data
2. Share URLs and keys through different communication channels
3. Verify the HTTPS certificate before entering secrets
4. Use appropriate TTL values (shorter is more secure)
5. Never share secret URLs in public forums or logs

### Known Security Properties

#### What Hakanai DOES protect against:
- Server compromise (zero-knowledge architecture)
- Data persistence (automatic deletion after access)
- Replay attacks (single-view design)
- Content tampering (mandatory hash verification)
- Unauthorized access (token authentication)

#### What Hakanai DOES NOT protect against:
- Compromised client devices
- Network eavesdropping without TLS
- Malicious reverse proxy
- Physical access to Redis storage
- Social engineering for URL/key disclosure
- Browser cache/history on client devices

## Security Audit History

- **2025-01-07**: Internal security audit completed (Grade: A-)
- Regular dependency updates via Dependabot

## Vulnerability Disclosure

We follow coordinated disclosure practices:
1. Security issues are fixed in private
2. New version is released with the fix
3. Security advisory is published after release
4. Credit given to reporters (with permission)

## Acknowledgments

We thank the following for their responsible disclosure and contributions to Hakanai's security:
- *Your name could be here*

---

*This security policy is based on industry best practices and follows the guidelines from [securitytxt.org](https://securitytxt.org/) and [GitHub's security policy templates](https://docs.github.com/en/code-security/getting-started/adding-a-security-policy-to-your-repository).*
