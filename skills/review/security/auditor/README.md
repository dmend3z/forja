# Security Auditor

> Phase: **Review** | Tech: **security**

Security audit covering OWASP Top 10, hardcoded secrets, injection vulnerabilities, and auth issues.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| security-auditor | Read, Grep, Glob, Bash | opus |

## What it does

The security-auditor agent finds vulnerabilities before attackers do. It scans for hardcoded secrets, reviews authentication and authorization logic, checks input validation at system boundaries, analyzes data flow for injection vulnerabilities, and reviews dependency versions for known CVEs. Findings include severity ratings and remediation steps.

## Usage

After installing with `forja install review/security/auditor`:

```bash
# Use the security-auditor agent for security review
security-auditor
```

The agent checks for:
- OWASP Top 10 vulnerabilities
- Hardcoded secrets and credentials
- Broken access control and IDOR
- SQL injection, XSS, and path traversal
- Missing input validation
- Weak cryptography

## Install

```bash
forja install review/security/auditor
```
