use std::collections::HashMap;

use crate::Tool;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.tools.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ToolError, ToolOutput};

    struct DummyTool {
        tool_name: String,
    }

    impl DummyTool {
        fn new(name: &str) -> Self {
            Self {
                tool_name: name.to_string(),
            }
        }
    }

    impl Tool for DummyTool {
        fn name(&self) -> &str {
            &self.tool_name
        }
        fn description(&self) -> &str {
            "A dummy tool for testing"
        }
        fn execute(&self, _input: serde_json::Value) -> Result<ToolOutput, ToolError> {
            Ok(ToolOutput::ok(serde_json::json!("dummy")))
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(DummyTool::new("test_tool")));

        let tool = reg.get("test_tool");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "test_tool");
    }

    #[test]
    fn test_get_nonexistent() {
        let reg = ToolRegistry::new();
        assert!(reg.get("missing").is_none());
    }

    #[test]
    fn test_names() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(DummyTool::new("bravo")));
        reg.register(Box::new(DummyTool::new("alpha")));

        let names = reg.names();
        assert_eq!(names, vec!["alpha", "bravo"]);
    }

    #[test]
    fn test_count() {
        let mut reg = ToolRegistry::new();
        assert_eq!(reg.count(), 0);

        reg.register(Box::new(DummyTool::new("a")));
        reg.register(Box::new(DummyTool::new("b")));
        assert_eq!(reg.count(), 2);
    }

    #[test]
    fn test_register_overwrites_same_name() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(DummyTool::new("tool")));
        reg.register(Box::new(DummyTool::new("tool")));
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn test_execute_through_registry() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(DummyTool::new("exec_test")));

        let tool = reg.get("exec_test").unwrap();
        let result = tool.execute(serde_json::json!({})).unwrap();
        assert!(result.success);
    }
}
