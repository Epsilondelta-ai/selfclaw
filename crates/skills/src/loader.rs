use std::path::Path;

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use thiserror::Error;

use crate::skill::Skill;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("missing required field: {0}")]
    MissingField(String),
}

/// State machine section tracker for the markdown parser.
#[derive(Debug, PartialEq)]
enum Section {
    None,
    SkillName,
    Trigger,
    ToolsRequired,
    Procedure,
}

/// Flush accumulated text into the appropriate field.
fn flush_section(
    section: &Section,
    text: &str,
    trigger: &mut Option<String>,
    tools: &mut Vec<String>,
) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    match section {
        Section::Trigger if trigger.is_none() => {
            *trigger = Some(trimmed.to_string());
        }
        Section::ToolsRequired if tools.is_empty() => {
            *tools = trimmed
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        _ => {}
    }
}

/// Parse a single skill markdown string into a Skill struct.
pub fn parse_skill(content: &str, source_path: Option<&str>) -> Result<Skill, LoadError> {
    let parser = Parser::new(content);

    let mut name: Option<String> = None;
    let mut trigger: Option<String> = None;
    let mut tools_required: Vec<String> = Vec::new();
    let mut procedure_steps: Vec<String> = Vec::new();

    let mut current_section = Section::None;
    let mut in_heading = false;
    let mut heading_level = HeadingLevel::H1;
    let mut heading_text = String::new();
    let mut collecting_text = false;
    let mut text_buffer = String::new();
    let mut in_list_item = false;
    let mut list_item_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush_section(
                    &current_section,
                    &text_buffer,
                    &mut trigger,
                    &mut tools_required,
                );
                text_buffer.clear();

                in_heading = true;
                heading_level = level;
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                let h = heading_text.trim().to_string();

                if heading_level == HeadingLevel::H1 && h.starts_with("Skill:") {
                    name = Some(h.strip_prefix("Skill:").unwrap().trim().to_string());
                    current_section = Section::SkillName;
                    collecting_text = false;
                } else if heading_level == HeadingLevel::H2 {
                    if h.starts_with("Trigger:") {
                        let inline = h.strip_prefix("Trigger:").unwrap().trim().to_string();
                        if !inline.is_empty() {
                            trigger = Some(inline);
                            current_section = Section::Trigger;
                            collecting_text = false;
                        } else {
                            current_section = Section::Trigger;
                            collecting_text = true;
                        }
                    } else if h.starts_with("Tools Required:") {
                        let inline = h
                            .strip_prefix("Tools Required:")
                            .unwrap()
                            .trim()
                            .to_string();
                        if !inline.is_empty() {
                            tools_required = inline
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            current_section = Section::ToolsRequired;
                            collecting_text = false;
                        } else {
                            current_section = Section::ToolsRequired;
                            collecting_text = true;
                        }
                    } else if h.starts_with("Procedure:") || h == "Procedure" {
                        current_section = Section::Procedure;
                        collecting_text = false;
                    } else {
                        current_section = Section::None;
                        collecting_text = false;
                    }
                }
            }
            Event::Text(text) => {
                if in_heading {
                    heading_text.push_str(&text);
                } else if collecting_text {
                    text_buffer.push_str(&text);
                } else if in_list_item && current_section == Section::Procedure {
                    list_item_text.push_str(&text);
                }
            }
            Event::Start(Tag::Item) => {
                in_list_item = true;
                list_item_text.clear();
            }
            Event::End(TagEnd::Item) => {
                in_list_item = false;
                if current_section == Section::Procedure {
                    let step = list_item_text.trim().to_string();
                    if !step.is_empty() {
                        procedure_steps.push(step);
                    }
                }
                list_item_text.clear();
            }
            _ => {}
        }
    }

    // Flush final section
    flush_section(
        &current_section,
        &text_buffer,
        &mut trigger,
        &mut tools_required,
    );

    let name = name.ok_or_else(|| LoadError::MissingField("name (# Skill: ...)".to_string()))?;
    let trigger =
        trigger.ok_or_else(|| LoadError::MissingField("trigger (## Trigger: ...)".to_string()))?;

    Ok(Skill {
        name,
        trigger,
        tools_required,
        procedure_steps,
        source_path: source_path.map(|s| s.to_string()),
    })
}

/// Load a skill from a markdown file.
pub fn load_skill_file(path: &Path) -> Result<Skill, LoadError> {
    let content = std::fs::read_to_string(path)?;
    let source = path.to_string_lossy().to_string();
    parse_skill(&content, Some(&source))
}

/// Load all skills from a directory.
pub fn load_skills_dir(dir: &Path) -> Result<Vec<Skill>, LoadError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            match load_skill_file(&path) {
                Ok(skill) => {
                    tracing::info!(skill = %skill.name, path = %path.display(), "Loaded skill");
                    skills.push(skill);
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "Failed to load skill");
                }
            }
        }
    }

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    const SAMPLE_SKILL_MD: &str = r#"# Skill: Greet Human

## Trigger: When a human initiates contact for the first time

## Tools Required: human_message, memory_query

## Procedure:
1. Check relational memory for prior interactions with this human.
2. If no prior interaction exists, introduce SelfClaw and express genuine curiosity about the human.
3. If prior interaction exists, reference something from the previous conversation to demonstrate continuity.
4. Log the interaction in the appropriate relational memory file.
"#;

    #[test]
    fn test_parse_sample_skill() {
        let skill = parse_skill(SAMPLE_SKILL_MD, Some("test.md")).unwrap();

        assert_eq!(skill.name, "Greet Human");
        assert_eq!(
            skill.trigger,
            "When a human initiates contact for the first time"
        );
        assert_eq!(skill.tools_required, vec!["human_message", "memory_query"]);
        assert_eq!(skill.procedure_steps.len(), 4);
        assert!(skill.procedure_steps[0].contains("Check relational memory"));
        assert!(skill.procedure_steps[3].contains("Log the interaction"));
        assert_eq!(skill.source_path.as_deref(), Some("test.md"));
    }

    #[test]
    fn test_parse_minimal_skill() {
        let md = r#"# Skill: Minimal
## Trigger: always
## Tools Required: none
## Procedure:
1. Do nothing.
"#;
        let skill = parse_skill(md, None).unwrap();
        assert_eq!(skill.name, "Minimal");
        assert_eq!(skill.trigger, "always");
        assert_eq!(skill.tools_required, vec!["none"]);
        assert_eq!(skill.procedure_steps.len(), 1);
        assert!(skill.source_path.is_none());
    }

    #[test]
    fn test_parse_missing_name() {
        let md = "## Trigger: something\n## Tools Required: x\n## Procedure:\n1. Do it.\n";
        let result = parse_skill(md, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_parse_missing_trigger() {
        let md = "# Skill: Test\n## Tools Required: x\n## Procedure:\n1. Do it.\n";
        let result = parse_skill(md, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("trigger"));
    }

    #[test]
    fn test_parse_empty_tools() {
        let md = "# Skill: Test\n## Trigger: test trigger\n## Procedure:\n1. Step one.\n";
        let skill = parse_skill(md, None).unwrap();
        assert_eq!(skill.name, "Test");
        assert!(skill.tools_required.is_empty());
    }

    #[test]
    fn test_parse_multiple_tools() {
        let md = "# Skill: Multi\n## Trigger: test\n## Tools Required: file_read, shell_exec, llm_call\n## Procedure:\n1. Do.\n";
        let skill = parse_skill(md, None).unwrap();
        assert_eq!(
            skill.tools_required,
            vec!["file_read", "shell_exec", "llm_call"]
        );
    }

    #[test]
    fn test_load_skill_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test_skill.md");
        let mut file = std::fs::File::create(&path).unwrap();
        write!(file, "{}", SAMPLE_SKILL_MD).unwrap();

        let skill = load_skill_file(&path).unwrap();
        assert_eq!(skill.name, "Greet Human");
    }

    #[test]
    fn test_load_skill_file_nonexistent() {
        let result = load_skill_file(Path::new("/nonexistent/skill.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_skills_dir() {
        let dir = TempDir::new().unwrap();

        let mut f1 = std::fs::File::create(dir.path().join("skill1.md")).unwrap();
        write!(
            f1,
            "# Skill: Alpha\n## Trigger: test alpha\n## Procedure:\n1. Step.\n"
        )
        .unwrap();

        let mut f2 = std::fs::File::create(dir.path().join("skill2.md")).unwrap();
        write!(
            f2,
            "# Skill: Beta\n## Trigger: test beta\n## Procedure:\n1. Step.\n"
        )
        .unwrap();

        // Non-md file should be ignored
        let mut f3 = std::fs::File::create(dir.path().join("readme.txt")).unwrap();
        write!(f3, "not a skill").unwrap();

        let skills = load_skills_dir(dir.path()).unwrap();
        assert_eq!(skills.len(), 2);

        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Alpha"));
        assert!(names.contains(&"Beta"));
    }

    #[test]
    fn test_load_skills_dir_nonexistent() {
        let skills = load_skills_dir(Path::new("/nonexistent/dir")).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn test_load_skills_dir_with_bad_file() {
        let dir = TempDir::new().unwrap();

        let mut f1 = std::fs::File::create(dir.path().join("good.md")).unwrap();
        write!(
            f1,
            "# Skill: Good\n## Trigger: test\n## Procedure:\n1. Yes.\n"
        )
        .unwrap();

        let mut f2 = std::fs::File::create(dir.path().join("bad.md")).unwrap();
        write!(f2, "This is not a valid skill file at all.").unwrap();

        let skills = load_skills_dir(dir.path()).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "Good");
    }
}
