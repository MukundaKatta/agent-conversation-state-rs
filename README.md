# agent-conversation-state

[![CI](https://github.com/MukundaKatta/agent-conversation-state-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/MukundaKatta/agent-conversation-state-rs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/agent-conversation-state.svg)](https://crates.io/crates/agent-conversation-state)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A small, dependency-light Rust library for tracking the **state and phase
transitions** of an LLM agent's conversation turn.

It models an agent turn as a simple lifecycle — idle, gathering information,
thinking, calling tools, responding, and finishing (or erroring) — and keeps a
full, inspectable history of every transition plus an arbitrary JSON context
map. It is intentionally unopinionated: it records *what happened* without
forcing a strict state-machine ordering on you.

## Features

- **`Phase` enum** covering the common stages of an agent turn, including a
  terminal `Error(String)` variant.
- **Full transition history** — every phase change is recorded as a
  [`Transition`] (`from`, `to`, optional JSON `metadata`).
- **Context map** — attach arbitrary `serde_json::Value` data keyed by string.
- **Terminal detection** — `is_terminal()` / `is_error()` to know when a turn
  has ended.
- Only one dependency (`serde_json`); no async runtime, no macros.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
agent-conversation-state = "0.1"
```

Or with cargo:

```sh
cargo add agent-conversation-state
```

## Usage

```rust
use agent_conversation_state::{ConversationState, Phase};
use serde_json::json;

let mut state = ConversationState::new();
assert_eq!(state.phase(), &Phase::Idle);

// Drive the conversation through its phases.
state.transition(Phase::GatheringInfo);
state.set_ctx("user_id", "u123");

state.transition(Phase::Thinking);

// Attach structured metadata to a transition (e.g. which tool was invoked).
state.transition_with(Phase::CallingTools, json!({ "tool": "search" }));

state.transition(Phase::Responding);
state.transition(Phase::Finished);

// Inspect what happened.
assert!(state.is_terminal());
assert_eq!(state.transition_count(), 5);
assert_eq!(state.previous_phase(), Some(&Phase::Responding));

let tool_step = &state.history()[2];
assert_eq!(tool_step.to, Phase::CallingTools);
assert_eq!(
    tool_step.metadata.as_ref().and_then(|m| m.get("tool")),
    Some(&json!("search"))
);

// Read context back later.
assert_eq!(state.get_ctx("user_id").and_then(|v| v.as_str()), Some("u123"));
```

### Handling errors

The `Error` phase is terminal and carries a message:

```rust
use agent_conversation_state::{ConversationState, Phase};

let mut state = ConversationState::new();
state.transition(Phase::CallingTools);
state.transition(Phase::Error("tool timed out".into()));

assert!(state.is_terminal());
assert!(state.phase().is_error());
assert_eq!(state.phase().to_string(), "error: tool timed out");
```

## API overview

### `Phase`

An enum of the conversation phases: `Idle`, `GatheringInfo`, `Thinking`,
`CallingTools`, `Responding`, `Finished`, and `Error(String)`.
Derives `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` (so it can be used as a
`HashMap` key) and implements `Display`.

| Method | Description |
| --- | --- |
| `is_terminal()` | `true` for `Finished` and `Error`. |
| `is_error()` | `true` only for `Error`. |

### `Transition`

A record of one phase change. Public fields: `from: Phase`, `to: Phase`,
`metadata: Option<serde_json::Value>`. Derives `Debug`, `Clone`, `PartialEq`,
`Eq`.

### `ConversationState`

| Method | Description |
| --- | --- |
| `new()` / `default()` | Create a state starting in `Phase::Idle`. |
| `phase()` | Current `&Phase`. |
| `transition(to)` | Move to a new phase, recording the change. |
| `transition_with(to, metadata)` | Transition and attach JSON metadata. |
| `set_ctx(key, value)` | Store a context value (`impl Into<Value>`). |
| `get_ctx(key)` | Look up a context value. |
| `remove_ctx(key)` | Remove and return a context value. |
| `transition_count()` | Number of recorded transitions. |
| `history()` | All transitions, oldest first. |
| `last_transition()` | The most recent transition, if any. |
| `previous_phase()` | The phase before the last transition. |
| `is_terminal()` | Whether the current phase is terminal. |
| `reset()` | Return to `Idle`, clearing history and context. |

## Development

```sh
cargo build
cargo test          # unit, integration, and doc tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## License

Licensed under the [MIT License](LICENSE).
