# KeyStar: Security Policy (SECURITY)

This document establishes the security guidelines, architectural privilege boundaries, and vulnerability disclosure procedures for `KeyStar`.

---

## 1. Security Architecture & Privilege Boundaries

### 1.1 Process Execution & Shell Detach
- `KeyStar` is designed to run under standard user privileges (non-elevated).
- Launching shortcuts and scripts relies on `open::that_detached`, which invokes standard Windows shell associations (`ShellExecuteEx`) in a detached process context.
- The launcher never auto-executes unrecognized network commands or arbitrary remote payloads. It activates only items explicitly placed by the user in designated folders or explicitly confirmed via the `Enter` key.

### 1.2 Mathematical Evaluation Safety (`evalexpr`)
- Real-time arithmetic evaluation relies on `evalexpr`, which parses pure mathematical expressions inside a constrained memory space.
- The engine lacks OS shell execution (`system()`/`exec()`), filesystem mutation, or network communication capabilities, preventing arbitrary code execution.
- Edge cases such as division by zero, nested parenthesis overflows, and parsing errors are handled defensively, safely returning `None` without panicking.

---

## 2. Recommendations for Directory Integrity

- Place only trusted executables, shortcuts, and scripts in the monitored directory (`%USERPROFILE%\commands`).
- On multi-user or shared workstations, ensure the monitored directories are protected against unauthorized write access by other accounts.

---

## 3. Reporting Security Vulnerabilities

If you discover a security vulnerability or critical issue in `KeyStar`, please do not open a public issue.
Instead, privately notify the repository maintainers. We will investigate promptly and release remediation patches.
