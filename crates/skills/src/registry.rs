use std::collections::HashMap;

use crate::skill::Skill;

/// Registry of loaded skills with context-based matching.
///
/// Skills are registered by name and can be matched against a context
/// string using keyword overlap with the skill's trigger field.
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Register a skill. Overwrites any existing skill with the same name.
    pub fn register(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Remove a skill by name.
    pub fn remove(&mut self, name: &str) -> Option<Skill> {
        self.skills.remove(name)
    }

    /// Get a skill by name.
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// List all registered skill names (sorted).
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.skills.keys().cloned().collect();
        names.sort();
        names
    }

    /// Return the number of registered skills.
    pub fn count(&self) -> usize {
        self.skills.len()
    }

    /// Clear all registered skills.
    pub fn clear(&mut self) {
        self.skills.clear();
    }

    /// Match a skill to a context string using keyword overlap.
    ///
    /// Returns the skill with the most keyword matches in the trigger
    /// field compared to the context. Returns None if no skill matches
    /// with at least one keyword.
    pub fn match_skill(&self, context: &str) -> Option<&Skill> {
        let context_lower = context.to_lowercase();
        let context_words: Vec<&str> = context_lower.split_whitespace().collect();

        let mut best_match: Option<(&Skill, usize)> = None;

        for skill in self.skills.values() {
            let keywords = skill.trigger_keywords();
            let match_count = keywords
                .iter()
                .filter(|kw| context_words.iter().any(|cw| cw.contains(kw.as_str())))
                .count();

            if match_count > 0 {
                if let Some((_, best_count)) = best_match {
                    if match_count > best_count {
                        best_match = Some((skill, match_count));
                    }
                } else {
                    best_match = Some((skill, match_count));
                }
            }
        }

        best_match.map(|(skill, _)| skill)
    }

    /// Find all skills that match the context (with at least one keyword).
    pub fn match_all(&self, context: &str) -> Vec<(&Skill, usize)> {
        let context_lower = context.to_lowercase();
        let context_words: Vec<&str> = context_lower.split_whitespace().collect();

        let mut matches: Vec<(&Skill, usize)> = self
            .skills
            .values()
            .filter_map(|skill| {
                let keywords = skill.trigger_keywords();
                let match_count = keywords
                    .iter()
                    .filter(|kw| context_words.iter().any(|cw| cw.contains(kw.as_str())))
                    .count();

                if match_count > 0 {
                    Some((skill, match_count))
                } else {
                    None
                }
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str, trigger: &str, tools: &[&str], steps: &[&str]) -> Skill {
        Skill {
            name: name.to_string(),
            trigger: trigger.to_string(),
            tools_required: tools.iter().map(|s| s.to_string()).collect(),
            procedure_steps: steps.iter().map(|s| s.to_string()).collect(),
            source_path: None,
        }
    }

    #[test]
    fn test_new_registry_empty() {
        let reg = SkillRegistry::new();
        assert_eq!(reg.count(), 0);
        assert!(reg.names().is_empty());
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = SkillRegistry::new();
        let skill = make_skill("Greet", "human contact first time", &["human_message"], &["Say hi"]);
        reg.register(skill);

        assert_eq!(reg.count(), 1);
        let retrieved = reg.get("Greet").unwrap();
        assert_eq!(retrieved.name, "Greet");
    }

    #[test]
    fn test_register_overwrites() {
        let mut reg = SkillRegistry::new();
        let s1 = make_skill("Test", "trigger one", &[], &["Step 1"]);
        let s2 = make_skill("Test", "trigger two", &[], &["Step 2"]);

        reg.register(s1);
        reg.register(s2);

        assert_eq!(reg.count(), 1);
        assert_eq!(reg.get("Test").unwrap().trigger, "trigger two");
    }

    #[test]
    fn test_remove() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("A", "trigger", &[], &[]));
        assert_eq!(reg.count(), 1);

        let removed = reg.remove("A");
        assert!(removed.is_some());
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut reg = SkillRegistry::new();
        assert!(reg.remove("nope").is_none());
    }

    #[test]
    fn test_names_sorted() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("Charlie", "c", &[], &[]));
        reg.register(make_skill("Alpha", "a", &[], &[]));
        reg.register(make_skill("Bravo", "b", &[], &[]));

        assert_eq!(reg.names(), vec!["Alpha", "Bravo", "Charlie"]);
    }

    #[test]
    fn test_clear() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("A", "a", &[], &[]));
        reg.register(make_skill("B", "b", &[], &[]));
        assert_eq!(reg.count(), 2);

        reg.clear();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn test_match_skill_single_keyword() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill(
            "Greet",
            "When a human initiates contact for the first time",
            &[],
            &[],
        ));

        let matched = reg.match_skill("A human just said hello for the first time");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().name, "Greet");
    }

    #[test]
    fn test_match_skill_no_match() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill(
            "Greet",
            "When a human initiates contact",
            &[],
            &[],
        ));

        let matched = reg.match_skill("The weather is nice today");
        assert!(matched.is_none());
    }

    #[test]
    fn test_match_skill_best_match() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill(
            "Research",
            "research explore learn knowledge topic",
            &[],
            &[],
        ));
        reg.register(make_skill(
            "Greet",
            "human contact greeting hello",
            &[],
            &[],
        ));

        // "research a new topic" should match Research more than Greet
        let matched = reg.match_skill("I want to research a new topic and learn about it");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().name, "Research");
    }

    #[test]
    fn test_match_all() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("A", "human learn explore", &[], &[]));
        reg.register(make_skill("B", "human message contact", &[], &[]));
        reg.register(make_skill("C", "weather forecast rain", &[], &[]));

        let matches = reg.match_all("A human wants to learn and explore");
        // A and B should match (both have "human"), C should not
        assert_eq!(matches.len(), 2);
        // A should be first (more keywords match: human, learn, explore)
        assert_eq!(matches[0].0.name, "A");
        assert!(matches[0].1 > matches[1].1);
    }

    #[test]
    fn test_match_case_insensitive() {
        let mut reg = SkillRegistry::new();
        reg.register(make_skill("Test", "Research and Explore", &[], &[]));

        let matched = reg.match_skill("I want to RESEARCH something");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().name, "Test");
    }
}
