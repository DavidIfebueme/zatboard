Tasks for ZatBoard

This document lists all the tasks required to build ZatBoard (Zcash Addressed Text Bulletin Board). Tasks are grouped by phases.

Phase 0 – Setup & Environment

- [x] Install Rust toolchain (rustup, cargo).

- [x] Explore basic Rust syntax (ownership, structs, enums, traits).

- [x] Install and set up zingo-cli or zingolib.

- [x] Confirm you can:

  - [x] Generate Zcash shielded addresses.

  - [x] Send/receive transactions with memos on testnet.

  - [x] Decode memos locally.

- [x] Create GitHub repo with structure:

/src        # Rust source  
/docs       # Documentation  
/cli        # CLI tool  
/coordinator # Coordinator daemon  
/tests      # Integration tests  
tasks.md  
README.md  

Phase 1 – Core Zcash Memo Messaging

- [x] Implement basic Rust wrapper for zingo-cli (send/receive memos).

- [x] Create Message struct (fields: sender, recipient, memo text, signature).

- [x] Implement digital signature verification (for reply address binding).

- [x] Implement polling loop for incoming memos (testnet).

- [x] Write unit test: send memo → receive memo → parse message.

Phase 2 – Reply Address & Authentication

- [x] Add ability for user to set persistent reply-to shielded address.

- [x] Implement message signing (user signs their first command).

- [x] Coordinator verifies sender via signature + reply address.

- [x] Store mapping of session_id → reply_address.

Phase 3 – Filesystem Commands

- [x] Define directory structure model (Folder, File, permissions).

- [ ] Implement command parsing:

  - [ ] ls <folder> → list contents.

  - [ ] cat <file> → show file content.

  - [ ] mkdir <folder> → create folder.

  - [ ] echo "msg" > <file> → write to file.

- [ ] Coordinator responds with command results via memo.

- [ ] Implement permissions system (read/write per user).

- [ ] Store directory state locally (SQLite or RocksDB).

Phase 4 – Chat System

- [ ] Extend folders to also act as chatrooms.

- [ ] Implement chat <folder> "message" command.

- [ ] Store chat logs locally as memo history.

- [ ] When user rejoins, replay full chat history.

- [ ] Mark messages with timestamp + sender.

- [ ] Test multiple users chatting in same folder.

Phase 5 – Multi-Recipient Messaging

- [ ] Add coordinator ability to send one message to multiple reply addresses.

- [ ] Implement broadcast notifications (e.g., "new file uploaded to folder").

- [ ] Optimize memo batching for announcements.

Phase 6 – Latency Handling

- [ ] Implement aggressive polling (short intervals, local caching).

- [ ] Buffer responses so users don't wait full 75s for commands.

- [ ] Leave chat async (no buffering).

Phase 7 – Coordinator Service

- [ ] Package coordinator as a Rust daemon.

- [ ] Config file: specify listening address, permissions, fees.

- [ ] Add option for coordinators to charge micro-fees for access.

- [ ] Logging & error handling.

- [ ] Add JSON-RPC interface (so web demo can query).

Phase 8 – CLI Client

- [ ] Build CLI frontend (Rust binary).

- [ ] Commands:

  - zatboard connect <coordinator_address>
  - zatboard ls <folder>
  - zatboard cat <file>
  - zatboard chat <folder> "message"
  - zatboard mkdir <folder>

- [ ] Pretty-print responses in terminal.

- [ ] Store local config (reply address, keys).

Phase 9 – Testing & Testnet Deployment

- [ ] Run coordinator + client against Zcash testnet.

- [ ] Simulate multiple users and verify:

  - [ ] Commands return expected results.

  - [ ] Chats persist and replay.

  - [ ] Multi-recipient broadcasts work.

- [ ] Measure latency & optimize polling.

Phase 10 – Demo & Web Mock

- [ ] Build minimal web demo (HTML/JS) that queries the coordinator API.

- [ ] Show filesystem navigation + chat in browser.

- [ ] Record video demo of interactions.

- [ ] Prepare hackathon documentation (README, usage, screenshots).