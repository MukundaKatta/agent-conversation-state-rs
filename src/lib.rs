/*!
agent-conversation-state: track state and phase transitions for LLM agents.

```rust
use agent_conversation_state::{ConversationState, Phase};

let mut state = ConversationState::new();
assert_eq!(state.phase(), Phase::Idle);
state.transition(Phase::GatheringInfo);
assert_eq!(state.phase(), Phase::GatheringInfo);
```
*/

use serde_json::Value;
use std::fmt;

/// Phase in the agent conversation lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Idle,
    GatheringInfo,
    Thinking,
    CallingTools,
    Responding,
    Finished,
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
    pub fn is_terminal(&self) -> bool { matches!(self, Phase::Finished | Phase::Error(_)) }
}

/// A state transition record.
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: Phase,
    pub to: Phase,
    pub metadata: Option<Value>,
}

/// Tracks conversation phase and context data.
pub struct ConversationState {
    phase: Phase,
    history: Vec<Transition>,
    context: std::collections::HashMap<String, Value>,
}

impl ConversationState {
    pub fn new() -> Self {
        Self { phase: Phase::Idle, history: Vec::new(), context: std::collections::HashMap::new() }
    }

    pub fn phase(&self) -> &Phase { &self.phase }

    /// Transition to a new phase.
    pub fn transition(&mut self, to: Phase) {
        let from = self.phase.clone();
        self.history.push(Transition { from, to: to.clone(), metadata: None });
        self.phase = to;
    }

    /// Transition with metadata.
    pub fn transition_with(&mut self, to: Phase, metadata: Value) {
        let from = self.phase.clone();
        self.history.push(Transition { from, to: to.clone(), metadata: Some(metadata) });
        self.phase = to;
    }

    /// Set a context value.
    pub fn set_ctx<V: Into<Value>>(&mut self, key: &str, value: V) {
        self.context.insert(key.to_string(), value.into());
    }

    pub fn get_ctx(&self, key: &str) -> Option<&Value> { self.context.get(key) }

    pub fn transition_count(&self) -> usize { self.history.len() }
    pub fn history(&self) -> &[Transition] { &self.history }

    pub fn is_terminal(&self) -> bool { self.phase.is_terminal() }

    /// Previous phase (before last transition).
    pub fn previous_phase(&self) -> Option<&Phase> {
        self.history.last().map(|t| &t.from)
    }

    /// Reset to Idle.
    pub fn reset(&mut self) {
        self.phase = Phase::Idle;
        self.history.clear();
        self.context.clear();
    }
}

impl Default for ConversationState {
    fn default() -> Self { Self::new() }
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
}
