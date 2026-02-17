# ZatBoard

ZatBoard is a filesystem-style bulletin board and folder chat tool over Zcash memos.

## MVP Scope

- Coordinator daemon processes memo commands and persists filesystem/chat state.
- User CLI sends register/auth/command memos and polls for responses.
- Testnet-oriented workflow with `zingo-cli` integration.

## Compatibility

- Rust: stable toolchain (tested with `1.90.0`)
- Zcash client: `zingo-cli` on testnet

## Install

From crates.io:

```bash
cargo install zatboard
```

From source:

```bash
git clone https://github.com/DavidIfebueme/zatboard
cd zatboard
cargo build --release
```

## Coordinator Setup

Create `coordinator.toml` from the example:

```bash
cp coordinator.toml.example coordinator.toml
```

Start coordinator:

```bash
zatboard-coordinator
```

Or with Cargo during development:

```bash
cargo run --bin zatboard-coordinator
```

## User CLI Setup

Optional environment overrides:

```bash
export ZATBOARD_DATA_DIR=./client_data
export ZATBOARD_SERVER=http://127.0.0.1:9067
```

Commands:

```bash
zatboard connect <coordinator_address>
zatboard register <coordinator_address> <reply_address>
zatboard auth <coordinator_address> <challenge> <signature>
zatboard command <coordinator_address> "ls /"
zatboard poll
```

The CLI persists local state in `client_data/client_state.json`.

## Development and Tests

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Security Note

The current MVP authentication is lightweight and intended for iterative development. Treat this release as an MVP, not a hardened production security model.

## License

MIT
