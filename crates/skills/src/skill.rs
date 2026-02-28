use serde::{Deserialize, Serialize};

/// A runtime-loadable skill definition parsed from a markdown file.
///
/// Skill format:
/// ```markdown
/// # Skill: {name}
/// ## Trigger: {when to use this skill}
/// ## Tools Required: {comma-separated list of tools}
/// ## Procedure:
/// 1. Step one...
/// 2. Step two...
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    /// The skill name (from `# Skill: {name}`).
    pub name: String,

    /// When this skill should be activated (from `## Trigger: ...`).
    pub trigger: String,

    /// Tools required by this skill (from `## Tools Required: ...`).
    pub tools_required: Vec<String>,

    /// Ordered steps in the procedure (from `## Procedure:`).
    pub procedure_steps: Vec<String>,

    /// The source file path this skill was loaded from.
    pub source_path: Option<String>,
}

impl Skill {
    /// Extract trigger keywords for matching.
    /// Splits the trigger text into lowercase words for keyword matching.
    pub fn trigger_keywords(&self) -> Vec<String> {
        self.trigger
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2) // skip very short words
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

impl std::fmt::Display for Skill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Skill({})", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skill() -> Skill {
        Skill {
            name: "Greet Human".to_string(),
            trigger: "When a human initiates contact for the first time".to_string(),
            tools_required: vec!["human_message".to_string(), "memory_query".to_string()],
            procedure_steps: vec![
                "Check relational memory for prior interactions.".to_string(),
                "Introduce SelfClaw if no prior interaction.".to_string(),
            ],
            source_path: Some("skills/greet_human.md".to_string()),
        }
    }

    #[test]
    fn test_skill_display() {
        let skill = sample_skill();
        assert_eq!(format!("{}", skill), "Skill(Greet Human)");
    }

    #[test]
    fn test_trigger_keywords() {
        let skill = sample_skill();
        let keywords = skill.trigger_keywords();
        assert!(keywords.contains(&"human".to_string()));
        assert!(keywords.contains(&"initiates".to_string()));
        assert!(keywords.contains(&"contact".to_string()));
        assert!(keywords.contains(&"first".to_string()));
        assert!(keywords.contains(&"time".to_string()));
        // Short words like "a", "for", "the" should be excluded
        assert!(!keywords.contains(&"a".to_string()));
    }

    #[test]
    fn test_skill_serde_roundtrip() {
        let skill = sample_skill();
        let json = serde_json::to_string(&skill).unwrap();
        let deserialized: Skill = serde_json::from_str(&json).unwrap();
        assert_eq!(skill, deserialized);
    }

    #[test]
    fn test_empty_trigger_keywords() {
        let skill = Skill {
            name: "Test".to_string(),
            trigger: String::new(),
            tools_required: vec![],
            procedure_steps: vec![],
            source_path: None,
        };
        assert!(skill.trigger_keywords().is_empty());
    }
}
