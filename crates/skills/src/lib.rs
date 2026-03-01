pub mod loader;
pub mod registry;
pub mod skill;
pub mod watcher;

pub use loader::{load_skill_file, load_skills_dir, parse_skill, LoadError};
pub use registry::SkillRegistry;
pub use skill::Skill;
pub use watcher::SkillWatcher;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
