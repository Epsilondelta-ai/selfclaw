pub mod skill;
pub mod loader;
pub mod registry;
pub mod watcher;

pub use skill::Skill;
pub use loader::{parse_skill, load_skill_file, load_skills_dir, LoadError};
pub use registry::SkillRegistry;
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
