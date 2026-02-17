# Changelog

## 0.1.0 - 2026-02-17

### Added
- Functional `zatboard` CLI subcommands for connect, register, auth, command, and poll.
- Client local bootstrap persistence in `client_data/client_state.json`.
- Retry-aware polling behavior in the user CLI.
- Integration coverage for registration/authentication flow and conversation-id command flow.
- Smoke install tests for binary presence and CLI usage output.
- Message transaction parser (`Message::from_zingo_transaction`) with tests.

### Changed
- `zingo-cli` command execution now uses argument-safe parsing and structured argument passing.
- Memo parsing now returns explicit parsing errors instead of silently dropping malformed payloads.
- Coordinator registration/auth challenge flow now stores and verifies challenges.
- Coordinator logging paths now avoid panic-prone string slicing.
- Coordinator now supports configurable DB filename and cache TTL through runtime options.
- Coordinator session cleanup now synchronizes session mappings, verified users, and pending challenges.
- Coordinator runtime now bounds response cache growth and processed txid memory growth.

### Fixed
- Registration runtime panic from out-of-bounds string slicing.
- Send memo amount handling path to respect provided zatoshi values.
- Placeholder parser panic path in `message.rs`.
