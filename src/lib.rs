/*!
agent-conversation-state: track state and phase transitions for LLM agents.

This crate provides a small, dependency-light state machine for modelling the
lifecycle of an LLM agent's turn. A [`ConversationState`] tracks the current
[`Phase`], a full history of [`Transition`]s, and an arbitrary
JSON [context map](ConversationState::set_ctx).

# Example

```rust
use agent_conversation_state::{ConversationState, Phase};
use serde_json::json;

let mut state = ConversationState::new();
assert_eq!(state.phase(), &Phase::Idle);

state.transition(Phase::GatheringInfo);
assert_eq!(state.phase(), &Phase::GatheringInfo);

// Attach metadata to a transition and store context for later.
state.transition_with(Phase::CallingTools, json!({ "tool": "search" }));
state.set_ctx("user_id", "u123");

assert_eq!(state.get_ctx("user_id").and_then(|v| v.as_str()), Some("u123"));
assert_eq!(state.transition_count(), 2);

// Terminal phases stop the conversation.
state.transition(Phase::Finished);
assert!(state.is_terminal());
```
*/

use serde_json::Value;
use std::fmt;

/// Phase in the agent conversation lifecycle.
///
/// Phases form a loose lifecycle: a turn typically starts in [`Phase::Idle`],
/// moves through information gathering, reasoning, and tool calls, and ends in a
/// terminal phase ([`Phase::Finished`] or [`Phase::Error`]). The type does not
/// enforce a strict ordering — callers are free to transition between any phases.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Phase {
    /// No active work; the initial phase of a fresh [`ConversationState`].
    Idle,
    /// Collecting information needed to act (e.g. clarifying questions).
    GatheringInfo,
    /// Reasoning about how to respond.
    Thinking,
    /// Invoking external tools or functions.
    CallingTools,
    /// Producing the response to the user.
    Responding,
    /// The conversation turn completed successfully (terminal).
    Finished,
    /// The conversation turn failed; carries an error message (terminal).
    Error(String),
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Phase::Idle => write!(f, "idle"),
            Phase::GatheringInfo => write!(f, "gathering_info"),
            Phase::Thinking => write!(f, "thinking"),
            Phase::CallingTools => write!(f, "calling_tools"),
            Phase::Responding => write!(f, "responding"),
            Phase::Finished => write!(f, "finished"),
            Phase::Error(msg) => write!(f, "error: {}", msg),
        }
    }
}

impl Phase {
    /// Returns `true` if the phase is terminal (the turn has ended).
    ///
    /// Terminal phases are [`Phase::Finished`] and [`Phase::Error`].
    ///
    /// ```
    /// use agent_conversation_state::Phase;
    /// assert!(Phase::Finished.is_terminal());
    /// assert!(Phase::Error("boom".into()).is_terminal());
    /// assert!(!Phase::Thinking.is_terminal());
    /// ```
    pub fn is_terminal(&self) -> bool {
        matches!(self, Phase::Finished | Phase::Error(_))
    }

    /// Returns `true` if the phase represents an error.
    ///
    /// ```
    /// use agent_conversation_state::Phase;
    /// assert!(Phase::Error("boom".into()).is_error());
    /// assert!(!Phase::Finished.is_error());
    /// ```
    pub fn is_error(&self) -> bool {
        matches!(self, Phase::Error(_))
    }
}

/// A record of a single phase transition.
///
/// Stored in [`ConversationState::history`](ConversationState::history) in the
/// order transitions occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    /// Phase the state was in before the transition.
    pub from: Phase,
    /// Phase the state moved to.
    pub to: Phase,
    /// Optional JSON metadata attached to the transition.
    pub metadata: Option<Value>,
}

/// Tracks the current conversation [`Phase`], the full transition history, and
/// an arbitrary JSON context map.
///
/// Construct one with [`ConversationState::new`] (or `Default`), drive it with
/// [`transition`](ConversationState::transition), and inspect it with the
/// accessors below.
pub struct ConversationState {
    phase: Phase,
    history: Vec<Transition>,
    context: std::collections::HashMap<String, Value>,
}

impl ConversationState {
    /// Creates a new state starting in [`Phase::Idle`] with empty history and
    /// context.
    pub fn new() -> Self {
        Self {
            phase: Phase::Idle,
            history: Vec::new(),
            context: std::collections::HashMap::new(),
        }
    }

    /// Returns the current phase.
    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// Transitions to a new phase, recording the change in [`history`].
    ///
    /// [`history`]: ConversationState::history
    pub fn transition(&mut self, to: Phase) {
        self.record_transition(to, None);
    }

    /// Transitions to a new phase, attaching JSON `metadata` to the recorded
    /// [`Transition`].
    pub fn transition_with(&mut self, to: Phase, metadata: Value) {
        self.record_transition(to, Some(metadata));
    }

    /// Shared implementation for the public `transition*` methods.
    fn record_transition(&mut self, to: Phase, metadata: Option<Value>) {
        let from = std::mem::replace(&mut self.phase, to.clone());
        self.history.push(Transition { from, to, metadata });
    }

    /// Sets a context value, overwriting any existing value for `key`.
    pub fn set_ctx<V: Into<Value>>(&mut self, key: &str, value: V) {
        self.context.insert(key.to_string(), value.into());
    }

    /// Returns the context value for `key`, if present.
    pub fn get_ctx(&self, key: &str) -> Option<&Value> {
        self.context.get(key)
    }

    /// Removes and returns the context value for `key`, if present.
    pub fn remove_ctx(&mut self, key: &str) -> Option<Value> {
        self.context.remove(key)
    }

    /// Returns the number of recorded transitions.
    pub fn transition_count(&self) -> usize {
        self.history.len()
    }

    /// Returns the full transition history, oldest first.
    pub fn history(&self) -> &[Transition] {
        &self.history
    }

    /// Returns the most recent transition, if any have occurred.
    pub fn last_transition(&self) -> Option<&Transition> {
        self.history.last()
    }

    /// Returns `true` if the current phase is terminal.
    ///
    /// See [`Phase::is_terminal`].
    pub fn is_terminal(&self) -> bool {
        self.phase.is_terminal()
    }

    /// Returns the phase the state was in before the most recent transition.
    ///
    /// Returns `None` if no transitions have occurred yet.
    pub fn previous_phase(&self) -> Option<&Phase> {
        self.history.last().map(|t| &t.from)
    }

    /// Resets the state back to [`Phase::Idle`], clearing history and context.
    pub fn reset(&mut self) {
        self.phase = Phase::Idle;
        self.history.clear();
        self.context.clear();
    }
}

impl Default for ConversationState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn starts_idle() {
        let s = ConversationState::new();
        assert_eq!(s.phase(), &Phase::Idle);
    }

    #[test]
    fn transition_changes_phase() {
        let mut s = ConversationState::new();
        s.transition(Phase::Thinking);
        assert_eq!(s.phase(), &Phase::Thinking);
    }

    #[test]
    fn history_records_transitions() {
        let mut s = ConversationState::new();
        s.transition(Phase::GatheringInfo);
        s.transition(Phase::Thinking);
        assert_eq!(s.transition_count(), 2);
        assert_eq!(s.history()[0].from, Phase::Idle);
        assert_eq!(s.history()[0].to, Phase::GatheringInfo);
    }

    #[test]
    fn previous_phase() {
        let mut s = ConversationState::new();
        s.transition(Phase::Thinking);
        assert_eq!(s.previous_phase(), Some(&Phase::Idle));
    }

    #[test]
    fn terminal_phases() {
        assert!(Phase::Finished.is_terminal());
        assert!(Phase::Error("oops".into()).is_terminal());
        assert!(!Phase::Thinking.is_terminal());
    }

    #[test]
    fn is_terminal_on_state() {
        let mut s = ConversationState::new();
        s.transition(Phase::Finished);
        assert!(s.is_terminal());
    }

    #[test]
    fn context_store() {
        let mut s = ConversationState::new();
        s.set_ctx("user_id", json!("u123"));
        assert_eq!(s.get_ctx("user_id").unwrap(), "u123");
    }

    #[test]
    fn context_missing_key() {
        let s = ConversationState::new();
        assert!(s.get_ctx("nope").is_none());
    }

    #[test]
    fn transition_with_metadata() {
        let mut s = ConversationState::new();
        s.transition_with(Phase::CallingTools, json!({"tool": "search"}));
        assert!(s.history()[0].metadata.is_some());
    }

    #[test]
    fn reset() {
        let mut s = ConversationState::new();
        s.transition(Phase::Thinking);
        s.set_ctx("key", json!(1));
        s.reset();
        assert_eq!(s.phase(), &Phase::Idle);
        assert_eq!(s.transition_count(), 0);
        assert!(s.get_ctx("key").is_none());
    }

    #[test]
    fn phase_display() {
        assert_eq!(Phase::Idle.to_string(), "idle");
        assert_eq!(Phase::GatheringInfo.to_string(), "gathering_info");
        assert!(Phase::Error("bad".into()).to_string().contains("bad"));
    }

    #[test]
    fn multiple_transitions() {
        let mut s = ConversationState::new();
        s.transition(Phase::GatheringInfo);
        s.transition(Phase::Thinking);
        s.transition(Phase::CallingTools);
        s.transition(Phase::Responding);
        s.transition(Phase::Finished);
        assert_eq!(s.transition_count(), 5);
        assert!(s.is_terminal());
    }

    #[test]
    fn is_error_distinguishes_terminal_phases() {
        assert!(Phase::Error("boom".into()).is_error());
        assert!(!Phase::Finished.is_error());
        assert!(!Phase::Idle.is_error());
    }

    #[test]
    fn previous_phase_is_none_initially() {
        let s = ConversationState::new();
        assert!(s.previous_phase().is_none());
    }

    #[test]
    fn last_transition_returns_most_recent() {
        let mut s = ConversationState::new();
        assert!(s.last_transition().is_none());
        s.transition(Phase::GatheringInfo);
        s.transition(Phase::Thinking);
        let last = s.last_transition().expect("a transition was recorded");
        assert_eq!(last.from, Phase::GatheringInfo);
        assert_eq!(last.to, Phase::Thinking);
        assert!(last.metadata.is_none());
    }

    #[test]
    fn transition_with_records_from_and_metadata() {
        let mut s = ConversationState::new();
        s.transition(Phase::Thinking);
        s.transition_with(Phase::CallingTools, json!({ "tool": "search" }));
        let last = s.last_transition().unwrap();
        assert_eq!(last.from, Phase::Thinking);
        assert_eq!(last.to, Phase::CallingTools);
        assert_eq!(last.metadata, Some(json!({ "tool": "search" })));
    }

    #[test]
    fn remove_ctx_returns_and_deletes() {
        let mut s = ConversationState::new();
        s.set_ctx("key", json!(42));
        assert_eq!(s.remove_ctx("key"), Some(json!(42)));
        assert!(s.get_ctx("key").is_none());
        assert!(s.remove_ctx("key").is_none());
    }

    #[test]
    fn phase_is_usable_as_hashmap_key() {
        // Requires the `Eq` + `Hash` derives on `Phase`.
        let mut counts: std::collections::HashMap<Phase, usize> = std::collections::HashMap::new();
        *counts.entry(Phase::Thinking).or_insert(0) += 1;
        *counts.entry(Phase::Thinking).or_insert(0) += 1;
        assert_eq!(counts.get(&Phase::Thinking), Some(&2));
    }

    #[test]
    fn transition_equality() {
        // Requires the `PartialEq` derive on `Transition`.
        let a = Transition {
            from: Phase::Idle,
            to: Phase::Thinking,
            metadata: None,
        };
        let b = Transition {
            from: Phase::Idle,
            to: Phase::Thinking,
            metadata: None,
        };
        assert_eq!(a, b);
    }
}
