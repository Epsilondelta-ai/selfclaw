use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Tool, ToolError, ToolOutput};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledAction {
    pub id: u64,
    pub execute_at: DateTime<Utc>,
    pub action: String,
    pub payload: serde_json::Value,
}

pub struct SchedulerTool {
    queue: Mutex<Vec<ScheduledAction>>,
    next_id: Mutex<u64>,
}

impl SchedulerTool {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }

    /// Get all pending scheduled actions.
    pub fn pending(&self) -> Vec<ScheduledAction> {
        self.queue.lock().unwrap().clone()
    }

    /// Get actions that are due (execute_at <= now).
    pub fn due_actions(&self) -> Vec<ScheduledAction> {
        let now = Utc::now();
        self.queue
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.execute_at <= now)
            .cloned()
            .collect()
    }

    /// Remove a scheduled action by ID.
    pub fn remove(&self, id: u64) -> bool {
        let mut queue = self.queue.lock().unwrap();
        let len_before = queue.len();
        queue.retain(|a| a.id != id);
        queue.len() < len_before
    }

    /// Get the count of scheduled actions.
    pub fn count(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}

impl Default for SchedulerTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for SchedulerTool {
    fn name(&self) -> &str {
        "schedule"
    }

    fn description(&self) -> &str {
        "Schedule a future action"
    }

    fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let action = input
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("action".to_string()))?;

        let execute_at_str = input
            .get("execute_at")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::MissingField("execute_at".to_string()))?;

        let execute_at: DateTime<Utc> = execute_at_str
            .parse()
            .map_err(|e: chrono::ParseError| {
                ToolError::InvalidInput(format!("invalid datetime: {}", e))
            })?;

        let payload = input
            .get("payload")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let scheduled = ScheduledAction {
            id,
            execute_at,
            action: action.to_string(),
            payload,
        };

        self.queue.lock().unwrap().push(scheduled);

        Ok(ToolOutput::ok(serde_json::json!({
            "scheduled_id": id,
            "execute_at": execute_at_str,
        })))
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_action() {
        let tool = SchedulerTool::new();
        let result = tool
            .execute(serde_json::json!({
                "action": "reflect",
                "execute_at": "2026-03-01T12:00:00Z",
                "payload": { "depth": "deep" }
            }))
            .unwrap();

        assert!(result.success);
        assert_eq!(result.data["scheduled_id"], 1);
        assert_eq!(tool.count(), 1);
    }

    #[test]
    fn test_schedule_multiple() {
        let tool = SchedulerTool::new();

        tool.execute(serde_json::json!({
            "action": "reflect",
            "execute_at": "2026-03-01T12:00:00Z"
        }))
        .unwrap();
        tool.execute(serde_json::json!({
            "action": "consolidate",
            "execute_at": "2026-03-01T13:00:00Z"
        }))
        .unwrap();

        assert_eq!(tool.count(), 2);
        let pending = tool.pending();
        assert_eq!(pending[0].action, "reflect");
        assert_eq!(pending[1].action, "consolidate");
    }

    #[test]
    fn test_schedule_ids_increment() {
        let tool = SchedulerTool::new();

        let r1 = tool
            .execute(serde_json::json!({
                "action": "a",
                "execute_at": "2026-03-01T12:00:00Z"
            }))
            .unwrap();
        let r2 = tool
            .execute(serde_json::json!({
                "action": "b",
                "execute_at": "2026-03-01T12:00:00Z"
            }))
            .unwrap();

        assert_eq!(r1.data["scheduled_id"], 1);
        assert_eq!(r2.data["scheduled_id"], 2);
    }

    #[test]
    fn test_schedule_missing_action() {
        let tool = SchedulerTool::new();
        let result = tool.execute(serde_json::json!({
            "execute_at": "2026-03-01T12:00:00Z"
        }));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("action"));
    }

    #[test]
    fn test_schedule_missing_execute_at() {
        let tool = SchedulerTool::new();
        let result = tool.execute(serde_json::json!({
            "action": "reflect"
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_schedule_invalid_datetime() {
        let tool = SchedulerTool::new();
        let result = tool.execute(serde_json::json!({
            "action": "reflect",
            "execute_at": "not-a-date"
        }));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("datetime"));
    }

    #[test]
    fn test_remove_action() {
        let tool = SchedulerTool::new();
        tool.execute(serde_json::json!({
            "action": "a",
            "execute_at": "2026-03-01T12:00:00Z"
        }))
        .unwrap();

        assert!(tool.remove(1));
        assert_eq!(tool.count(), 0);
    }

    #[test]
    fn test_remove_nonexistent() {
        let tool = SchedulerTool::new();
        assert!(!tool.remove(999));
    }

    #[test]
    fn test_due_actions_past() {
        let tool = SchedulerTool::new();
        // Schedule something in the past
        tool.execute(serde_json::json!({
            "action": "overdue",
            "execute_at": "2020-01-01T00:00:00Z"
        }))
        .unwrap();

        let due = tool.due_actions();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].action, "overdue");
    }

    #[test]
    fn test_due_actions_future() {
        let tool = SchedulerTool::new();
        // Schedule something far in the future
        tool.execute(serde_json::json!({
            "action": "future",
            "execute_at": "2099-01-01T00:00:00Z"
        }))
        .unwrap();

        let due = tool.due_actions();
        assert!(due.is_empty());
    }

    #[test]
    fn test_default_payload_is_null() {
        let tool = SchedulerTool::new();
        tool.execute(serde_json::json!({
            "action": "test",
            "execute_at": "2026-03-01T12:00:00Z"
        }))
        .unwrap();

        let pending = tool.pending();
        assert!(pending[0].payload.is_null());
    }

    #[test]
    fn test_scheduler_name_and_description() {
        let tool = SchedulerTool::new();
        assert_eq!(tool.name(), "schedule");
        assert!(!tool.description().is_empty());
    }
}
