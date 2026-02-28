use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::message::InboundMessage;

/// A thread-safe queue for chat messages.
///
/// Human messages are pushed into the queue from the CLI or other channels.
/// The agent loop drains the queue at the start of each cycle.
#[derive(Clone)]
pub struct ChatQueue {
    inner: Arc<Mutex<VecDeque<InboundMessage>>>,
}

impl ChatQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Push a message into the queue.
    pub fn push(&self, message: InboundMessage) {
        let mut q = self.inner.lock().unwrap();
        q.push_back(message);
    }

    /// Drain all messages from the queue.
    pub fn drain(&self) -> Vec<InboundMessage> {
        let mut q = self.inner.lock().unwrap();
        q.drain(..).collect()
    }

    /// Peek at the next message without removing it.
    pub fn peek(&self) -> Option<InboundMessage> {
        let q = self.inner.lock().unwrap();
        q.front().cloned()
    }

    /// Return the number of queued messages.
    pub fn len(&self) -> usize {
        let q = self.inner.lock().unwrap();
        q.len()
    }

    /// Return whether the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for ChatQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{ChannelKind, MessageIntent, MessageMetadata};

    fn make_message(content: &str) -> InboundMessage {
        InboundMessage {
            id: format!("test-{}", content.len()),
            content: content.to_string(),
            metadata: MessageMetadata {
                timestamp: "2026-03-01T12:00:00Z".to_string(),
                sender: "human".to_string(),
                channel: ChannelKind::Cli,
                intent: MessageIntent::Chat,
                conversation_id: None,
            },
        }
    }

    #[test]
    fn test_new_queue_is_empty() {
        let q = ChatQueue::new();
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn test_push_and_len() {
        let q = ChatQueue::new();
        q.push(make_message("hello"));
        assert_eq!(q.len(), 1);
        assert!(!q.is_empty());

        q.push(make_message("world"));
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn test_drain_returns_all_in_order() {
        let q = ChatQueue::new();
        q.push(make_message("first"));
        q.push(make_message("second"));
        q.push(make_message("third"));

        let messages = q.drain();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].content, "first");
        assert_eq!(messages[1].content, "second");
        assert_eq!(messages[2].content, "third");

        // Queue is now empty
        assert!(q.is_empty());
    }

    #[test]
    fn test_drain_empty_queue() {
        let q = ChatQueue::new();
        let messages = q.drain();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_peek_returns_front() {
        let q = ChatQueue::new();
        assert!(q.peek().is_none());

        q.push(make_message("first"));
        q.push(make_message("second"));

        let peeked = q.peek().unwrap();
        assert_eq!(peeked.content, "first");

        // Peek doesn't consume
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn test_clone_shares_state() {
        let q1 = ChatQueue::new();
        let q2 = q1.clone();

        q1.push(make_message("from q1"));
        assert_eq!(q2.len(), 1);

        let messages = q2.drain();
        assert_eq!(messages[0].content, "from q1");
        assert!(q1.is_empty());
    }

    #[test]
    fn test_thread_safety() {
        let q = ChatQueue::new();
        let q_clone = q.clone();

        let handle = std::thread::spawn(move || {
            for i in 0..100 {
                q_clone.push(make_message(&format!("msg-{}", i)));
            }
        });

        handle.join().unwrap();
        assert_eq!(q.len(), 100);
    }
}
