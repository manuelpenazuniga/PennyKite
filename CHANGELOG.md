# Changelog

All notable changes to PennyKite are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Public repo bootstrap: `.gitignore`, `README.md`, `backlog.yaml`.
- MIT licence.
- Agent operating manual (`docs/AGENT_CONTEXT.md`, private).
- README: "Dashboard control plane" section documenting live feed, session detail, kill-switch behaviour, and on-chain revocation status.
- README: "Local demo" section with step-by-step commands for proxy, target API, dashboard, session pause, and expected 402 response.
- `documentation/demo-runbook.md`: step-by-step local smoke runbook covering approve flow, deny_loop scenario, kill-switch pause, and cleanup.

## [0.1.0] — 2026-05-25 (planned)

- PennyKite reverse proxy (Rust, axum) with x402 intercept, atomic budget
  enforcement, loop detection, and Kite Passport session validation.
- `PennyKiteAttestor` Solidity contract deployed on Kite testnet.
- Next.js 16 control plane with live spend feed and kill-switch.
- Demo: runaway agent scenario (Express target API + Python agent).
- Python SDK convenience wrapper.
- Kite AI Global Hackathon 2026 submission.

[Unreleased]: https://github.com/manuelpenazuniga/PennyKite/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/manuelpenazuniga/PennyKite/releases/tag/v0.1.0
