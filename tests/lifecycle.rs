//! Integration tests exercising a realistic agent turn through the public API.

use agent_conversation_state::{ConversationState, Phase};
use serde_json::json;

#[test]
fn full_happy_path_lifecycle() {
    let mut state = ConversationState::new();
    assert_eq!(state.phase(), &Phase::Idle);
    assert!(!state.is_terminal());

    state.transition(Phase::GatheringInfo);
    state.set_ctx("question", "what is the weather?");

    state.transition(Phase::Thinking);
    state.transition_with(Phase::CallingTools, json!({ "tool": "weather_api" }));
    state.transition(Phase::Responding);
    state.transition(Phase::Finished);

    assert!(state.is_terminal());
    assert_eq!(state.transition_count(), 5);
    assert_eq!(state.previous_phase(), Some(&Phase::Responding));

    // The tool call transition carried metadata.
    let tool_step = &state.history()[2];
    assert_eq!(tool_step.to, Phase::CallingTools);
    assert_eq!(
        tool_step.metadata.as_ref().and_then(|m| m.get("tool")),
        Some(&json!("weather_api"))
    );

    // Context survives transitions.
    assert_eq!(
        state.get_ctx("question").and_then(|v| v.as_str()),
        Some("what is the weather?")
    );
}

#[test]
fn error_path_is_terminal_and_reportable() {
    let mut state = ConversationState::new();
    state.transition(Phase::CallingTools);
    state.transition(Phase::Error("tool timed out".into()));

    assert!(state.is_terminal());
    assert!(state.phase().is_error());
    assert!(state.phase().to_string().contains("tool timed out"));
}

#[test]
fn reset_returns_to_initial_state() {
    let mut state = ConversationState::new();
    state.transition(Phase::Thinking);
    state.set_ctx("k", json!(1));

    state.reset();

    assert_eq!(state.phase(), &Phase::Idle);
    assert_eq!(state.transition_count(), 0);
    assert!(state.previous_phase().is_none());
    assert!(state.get_ctx("k").is_none());
    assert!(!state.is_terminal());
}
