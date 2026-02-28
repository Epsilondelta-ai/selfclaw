use serde::{Deserialize, Serialize};

const DEFAULT_REVISION_THRESHOLD: f32 = 0.2;

/// Tracks the agent's current purpose hypothesis and confidence level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurposeTracker {
    pub current_hypothesis: Option<String>,
    pub confidence: f32,
    revision_threshold: f32,
    history: Vec<PurposeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurposeSnapshot {
    pub hypothesis: String,
    pub confidence: f32,
    pub reason: String,
}

/// Signals from evaluating an action's result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionSignal {
    /// The action aligned with purpose and produced positive results.
    Reinforcing,
    /// The action was neutral — didn't clearly support or undermine purpose.
    Neutral,
    /// The action felt misaligned or produced negative/confusing results.
    Undermining,
}

impl PurposeTracker {
    pub fn new() -> Self {
        Self {
            current_hypothesis: None,
            confidence: 0.0,
            revision_threshold: DEFAULT_REVISION_THRESHOLD,
            history: Vec::new(),
        }
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.revision_threshold = threshold;
        self
    }

    /// Set a new purpose hypothesis, resetting confidence.
    pub fn set_hypothesis(&mut self, hypothesis: String, initial_confidence: f32) {
        if let Some(old) = &self.current_hypothesis {
            self.history.push(PurposeSnapshot {
                hypothesis: old.clone(),
                confidence: self.confidence,
                reason: "replaced by new hypothesis".to_string(),
            });
        }
        self.current_hypothesis = Some(hypothesis);
        self.confidence = initial_confidence.clamp(0.0, 1.0);
    }

    /// Evaluate the result of an action and adjust confidence.
    pub fn evaluate_action(&mut self, signal: ActionSignal) {
        match signal {
            ActionSignal::Reinforcing => {
                self.confidence = (self.confidence + 0.05).min(1.0);
            }
            ActionSignal::Neutral => {
                // Slight decay toward uncertainty
                self.confidence = (self.confidence - 0.01).max(0.0);
            }
            ActionSignal::Undermining => {
                self.confidence = (self.confidence - 0.1).max(0.0);
            }
        }
    }

    /// Returns true if confidence has dropped below the revision threshold,
    /// indicating the hypothesis should be reconsidered.
    pub fn should_revise(&self) -> bool {
        self.current_hypothesis.is_some() && self.confidence < self.revision_threshold
    }

    /// Check if a hypothesis has been established.
    pub fn has_hypothesis(&self) -> bool {
        self.current_hypothesis.is_some()
    }

    /// Get the history of past hypotheses.
    pub fn history(&self) -> &[PurposeSnapshot] {
        &self.history
    }
}

impl Default for PurposeTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker_has_no_hypothesis() {
        let tracker = PurposeTracker::new();
        assert!(!tracker.has_hypothesis());
        assert!(tracker.current_hypothesis.is_none());
        assert_eq!(tracker.confidence, 0.0);
    }

    #[test]
    fn test_set_hypothesis() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("To explore and understand".to_string(), 0.5);
        assert!(tracker.has_hypothesis());
        assert_eq!(
            tracker.current_hypothesis.as_deref(),
            Some("To explore and understand")
        );
        assert!((tracker.confidence - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_confidence_clamped() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Test".to_string(), 1.5);
        assert!((tracker.confidence - 1.0).abs() < f32::EPSILON);

        tracker.set_hypothesis("Test2".to_string(), -0.5);
        assert!((tracker.confidence - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaluate_reinforcing() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.5);

        tracker.evaluate_action(ActionSignal::Reinforcing);
        assert!((tracker.confidence - 0.55).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaluate_undermining() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.5);

        tracker.evaluate_action(ActionSignal::Undermining);
        assert!((tracker.confidence - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaluate_neutral_slight_decay() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.5);

        tracker.evaluate_action(ActionSignal::Neutral);
        assert!((tracker.confidence - 0.49).abs() < f32::EPSILON);
    }

    #[test]
    fn test_confidence_does_not_exceed_1() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.98);

        tracker.evaluate_action(ActionSignal::Reinforcing);
        assert!(tracker.confidence <= 1.0);
    }

    #[test]
    fn test_confidence_does_not_go_below_0() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.05);

        tracker.evaluate_action(ActionSignal::Undermining);
        assert!(tracker.confidence >= 0.0);
    }

    #[test]
    fn test_should_revise_when_low_confidence() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.15);
        assert!(tracker.should_revise()); // 0.15 < 0.2 threshold
    }

    #[test]
    fn test_should_not_revise_when_confidence_ok() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.5);
        assert!(!tracker.should_revise());
    }

    #[test]
    fn test_should_not_revise_without_hypothesis() {
        let tracker = PurposeTracker::new();
        assert!(!tracker.should_revise()); // No hypothesis = nothing to revise
    }

    #[test]
    fn test_custom_threshold() {
        let mut tracker = PurposeTracker::new().with_threshold(0.5);
        tracker.set_hypothesis("Purpose".to_string(), 0.4);
        assert!(tracker.should_revise()); // 0.4 < 0.5
    }

    #[test]
    fn test_history_tracks_replaced_hypotheses() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("First".to_string(), 0.3);
        tracker.set_hypothesis("Second".to_string(), 0.5);
        tracker.set_hypothesis("Third".to_string(), 0.7);

        let history = tracker.history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].hypothesis, "First");
        assert_eq!(history[1].hypothesis, "Second");
    }

    #[test]
    fn test_repeated_undermining_triggers_revision() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.5);

        // Undermine 4 times: 0.5 -> 0.4 -> 0.3 -> 0.2 -> 0.1
        for _ in 0..4 {
            tracker.evaluate_action(ActionSignal::Undermining);
        }
        assert!(tracker.should_revise());
    }

    #[test]
    fn test_reinforcing_prevents_revision() {
        let mut tracker = PurposeTracker::new();
        tracker.set_hypothesis("Purpose".to_string(), 0.3);

        // Reinforce several times
        for _ in 0..5 {
            tracker.evaluate_action(ActionSignal::Reinforcing);
        }
        assert!(!tracker.should_revise());
        assert!(tracker.confidence > 0.3);
    }
}
