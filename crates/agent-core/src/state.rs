use serde::{Deserialize, Serialize};

/// The agent's current phase within the loop cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Idle,
    Reflecting,
    Thinking,
    Planning,
    Acting,
    Observing,
    Updating,
}

impl AgentState {
    /// Return the next state in the loop cycle.
    /// Idle -> Reflecting -> Thinking -> Planning -> Acting -> Observing -> Updating -> Idle
    pub fn next(self) -> Self {
        match self {
            Self::Idle => Self::Reflecting,
            Self::Reflecting => Self::Thinking,
            Self::Thinking => Self::Planning,
            Self::Planning => Self::Acting,
            Self::Acting => Self::Observing,
            Self::Observing => Self::Updating,
            Self::Updating => Self::Idle,
        }
    }

    pub fn is_idle(self) -> bool {
        self == Self::Idle
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Reflecting => "REFLECT",
            Self::Thinking => "THINK",
            Self::Planning => "PLAN",
            Self::Acting => "ACT",
            Self::Observing => "OBSERVE",
            Self::Updating => "UPDATE",
        }
    }
}

impl std::fmt::Display for AgentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions_full_cycle() {
        let mut state = AgentState::Idle;
        let expected = [
            AgentState::Reflecting,
            AgentState::Thinking,
            AgentState::Planning,
            AgentState::Acting,
            AgentState::Observing,
            AgentState::Updating,
            AgentState::Idle,
        ];
        for expected_state in expected {
            state = state.next();
            assert_eq!(state, expected_state);
        }
    }

    #[test]
    fn test_idle_check() {
        assert!(AgentState::Idle.is_idle());
        assert!(!AgentState::Reflecting.is_idle());
        assert!(!AgentState::Acting.is_idle());
    }

    #[test]
    fn test_state_labels() {
        assert_eq!(AgentState::Idle.label(), "IDLE");
        assert_eq!(AgentState::Reflecting.label(), "REFLECT");
        assert_eq!(AgentState::Thinking.label(), "THINK");
        assert_eq!(AgentState::Planning.label(), "PLAN");
        assert_eq!(AgentState::Acting.label(), "ACT");
        assert_eq!(AgentState::Observing.label(), "OBSERVE");
        assert_eq!(AgentState::Updating.label(), "UPDATE");
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", AgentState::Thinking), "THINK");
    }

    #[test]
    fn test_cycle_returns_to_idle() {
        let mut state = AgentState::Idle;
        for _ in 0..7 {
            state = state.next();
        }
        assert!(state.is_idle());
    }

    #[test]
    fn test_multiple_cycles() {
        let mut state = AgentState::Idle;
        for _ in 0..21 {
            // 3 full cycles
            state = state.next();
        }
        assert!(state.is_idle());
    }
}
