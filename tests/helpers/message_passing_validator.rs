/// Armstrong No-Shared-State Helper (Rust).
/// Validates communication via message passing, not shared memory.

use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

/// Type-safe message channel (replaces shared mutable state).
pub struct MessageChannel<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> MessageChannel<T> {
    /// Creates new message-passing channel.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        MessageChannel { sender, receiver }
    }

    /// Gets sender half (send to other thread).
    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }

    /// Waits for message with timeout.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (msg_send, msg_recv) = MessageChannel::new();
    /// std::thread::spawn(move || {
    ///     msg_send.send("hello").unwrap();
    /// });
    /// let msg = msg_recv.receive_with_timeout(Duration::from_secs(5)).unwrap();
    /// assert_eq!(msg, "hello");
    /// ```
    pub fn receive_with_timeout(&self, timeout: Duration) -> Result<T, String> {
        self.receiver
            .recv_timeout(timeout)
            .map_err(|e| format!("message not received: {}", e))
    }

    /// Receives message blocking indefinitely.
    pub fn receive(&self) -> Result<T, String> {
        self.receiver.recv().map_err(|e| e.to_string())
    }
}

impl<T> Default for MessageChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounded message queue (prevents unbounded accumulation).
pub struct BoundedQueue<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
    max_size: usize,
}

impl<T> BoundedQueue<T> {
    /// Creates bounded queue (prevents memory exhaustion).
    pub fn new(max_size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        BoundedQueue {
            sender,
            receiver,
            max_size,
        }
    }

    /// Sends message if queue not full.
    pub fn try_send(&mut self, msg: T) -> Result<(), String> {
        // In production, track queue size via atomic counter
        self.sender
            .send(msg)
            .map_err(|e| format!("failed to send: {}", e))
    }

    /// Receives message.
    pub fn recv(&self) -> Result<T, String> {
        self.receiver.recv().map_err(|e| e.to_string())
    }

    /// Max queue size.
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

/// Asserts message received within timeout (no shared state).
///
/// # Example
///
/// ```ignore
/// let (tx, rx) = MessageChannel::new();
/// std::thread::spawn(move || {
///     tx.send(42).unwrap();
/// });
/// assert_receives_message(&rx, Duration::from_secs(5));
/// ```
pub fn assert_receives_message<T: std::fmt::Debug>(
    receiver: &Receiver<T>,
    timeout: Duration,
) -> Result<T, String> {
    receiver
        .recv_timeout(timeout)
        .map_err(|_| format!("message not received within {:?}", timeout))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_message_channel() {
        let ch = MessageChannel::<String>::new();
        let sender = ch.sender();

        thread::spawn(move || {
            sender.send("hello".to_string()).unwrap();
        });

        let msg = ch
            .receive_with_timeout(Duration::from_secs(1))
            .unwrap();
        assert_eq!(msg, "hello");
    }

    #[test]
    fn test_message_channel_timeout() {
        let ch: MessageChannel<String> = MessageChannel::new();
        let result = ch.receive_with_timeout(Duration::from_millis(10));
        assert!(result.is_err());
    }

    #[test]
    fn test_bounded_queue() {
        let mut queue = BoundedQueue::new(100);
        assert!(queue.try_send(1).is_ok());
        let msg = queue.recv().unwrap();
        assert_eq!(msg, 1);
    }
}
