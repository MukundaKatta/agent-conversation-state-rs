# agent-conversation-state

A small, dependency-light Rust library for tracking the **state and phase transitions** of an LLM agent's conversation lifecycle.

It models a conversation as a simple finite state machine: the agent moves through phases such as gathering information, thinking, calling tools, and responding, while keeping a full history of every transition and an attached key/value context store.

## Why

LLM agents typically cycle through distinct phases on every turn. Keeping that lifecycle explicit makes it easier to log, debug, gate behavior, and reason about where an agent is at any moment. This crate provides a minimal primitive for exactly that — no async runtime, no framework lock-in, just a tracked state machine.

## Phases

The `Phase` enum captures the conversation lifecycle:

| Phase | Meaning |
|-------|---------|
| `Idle` | No active work (initial state) |
| `GatheringInfo` | Collecting input or context |
| `Thinking` | Reasoning / planning |
| `CallingTools` | Invoking external tools |
| `Responding` | Producing the answer |
| `Finished` | Terminal: completed successfully |
| `Error(String)` | Terminal: failed with a message |

`Finished` and `Error` are terminal phases (`Phase::is_terminal()`).

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
agent-conversation-state = "0.1"
```

## Usage

```rust
use agent_conversation_state::{ConversationState, Phase};
use serde_json::json;

let mut state = ConversationState::new();
assert_eq!(state.phase(), &Phase::Idle);

// Drive the conversation through its phases.
state.transition(Phase::GatheringInfo);
state.transition(Phase::Thinking);

// Attach metadata to a transition.
state.transition_with(Phase::CallingTools, json!({ "tool": "search" }));

// Store arbitrary context alongside the state.
state.set_ctx("user_id", json!("u123"));
assert_eq!(state.get_ctx("user_id").unwrap(), "u123");

// Inspect history.
assert_eq!(state.transition_count(), 3);
assert_eq!(state.previous_phase(), Some(&Phase::Thinking));

state.transition(Phase::Finished);
assert!(state.is_terminal());
```

## API overview

- `ConversationState::new()` / `Default` — create a state starting at `Idle`.
- `phase()` — borrow the current `Phase`.
- `transition(to)` — move to a new phase, recording the change.
- `transition_with(to, metadata)` — transition with attached `serde_json::Value` metadata.
- `set_ctx(key, value)` / `get_ctx(key)` — store and retrieve context values.
- `history()` / `transition_count()` — inspect the full list of `Transition` records.
- `previous_phase()` — the phase prior to the last transition.
- `is_terminal()` — whether the current phase is `Finished` or `Error`.
- `reset()` — return to `Idle` and clear history and context.

## Tech stack

- **Language:** Rust (edition 2021)
- **Dependencies:** [`serde_json`](https://crates.io/crates/serde_json) for context and metadata values
- **License:** MIT

## Development

```bash
cargo build      # compile the library
cargo test       # run the unit test suite
cargo clippy     # lint
```

## License

MIT
