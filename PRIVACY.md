# Workbase Privacy Policy

Last updated: 2026-06-23

Workbase is a **local-first** desktop application.

## Current State (v0.1.x)

- Workbase does **not** upload your project files, terminal sessions, command history, or workspace data to any server.
- All project data, session state, settings, and terminal output are stored locally on your device.
- The built-in update checker contacts GitHub Releases (`github.com/ZhangLiHua/workbase`) to check for new versions. This is a standard HTTPS request that does not include personal or project data.

## Future Changes

We may introduce optional features in the future that involve network communication, such as:

- Cloud sync of settings, sessions, or workspace state
- Account systems or license key validation
- Telemetry or usage analytics (always opt-in)
- Pro or Cloud features that require server-side processing

If and when such features are added:

- They will be **clearly documented** in this policy and in product release notes.
- Features that transmit data will be **opt-in** by default where technically feasible.
- We will describe what data is transmitted, why, and how it is stored.

## Third-Party Tools

Workbase runs user-installed command-line tools such as Claude Code, Codex, or custom scripts. These tools operate within your local environment and are governed by their own terms and privacy policies. Workbase does not intercept, redirect, or collect the data exchanged between you and third-party CLI tools.

## Contact

For privacy-related questions, contact: ZhangLiHua
