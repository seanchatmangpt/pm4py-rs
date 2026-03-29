/// Streaming event processing module
///
/// Provides traits and implementations for processing events in real-time
/// from pluggable sources with support for windowing and filtering.

use crate::log::Event;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

/// Result type for stream operations
pub type StreamResult<T> = Result<T, StreamError>;

/// Error types for streaming operations
#[derive(Debug, Clone)]
pub enum StreamError {
    /// Buffer overflow or capacity exceeded
    BufferFull(String),
    /// Stream ended prematurely
    StreamEnded,
    /// Invalid window configuration
    InvalidWindow(String),
    /// Filter operation failed
    FilterError(String),
    /// Custom error message
    Custom(String),
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamError::BufferFull(msg) => write!(f, "Buffer full: {}", msg),
            StreamError::StreamEnded => write!(f, "Stream ended"),
            StreamError::InvalidWindow(msg) => write!(f, "Invalid window: {}", msg),
            StreamError::FilterError(msg) => write!(f, "Filter error: {}", msg),
            StreamError::Custom(msg) => write!(f, "Stream error: {}", msg),
        }
    }
}

impl Error for StreamError {}

/// Window type for stream operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowType {
    /// Time-based window (duration in milliseconds)
    TimeBased(u64),
    /// Count-based window (number of events)
    CountBased(usize),
    /// Sliding window with both time and count constraints
    Sliding { count: usize, time_ms: u64 },
}

/// Statistics about the stream buffer
#[derive(Debug, Clone)]
pub struct BufferStats {
    /// Current number of events in buffer
    pub current_size: usize,
    /// Maximum capacity of buffer
    pub max_capacity: usize,
    /// Total events processed
    pub total_processed: u64,
    /// Total events dropped due to overflow
    pub total_dropped: u64,
}

/// Trait for pluggable event sources
pub trait EventSource: Send {
    /// Retrieve the next event from the source
    fn next(&mut self) -> StreamResult<Option<Event>>;

    /// Check if source has more events
    fn has_next(&self) -> bool;

    /// Reset the source to beginning (if supported)
    fn reset(&mut self) -> StreamResult<()> {
        Err(StreamError::Custom("Reset not supported".to_string()))
    }
}

/// In-memory event stream with buffer management
pub struct EventStream {
    /// Internal event buffer
    buffer: VecDeque<Event>,
    /// Maximum buffer capacity
    max_capacity: usize,
    /// Current window configuration
    window: Option<WindowType>,
    /// Statistics tracking
    total_processed: u64,
    total_dropped: u64,
    /// Optional event source
    source: Option<Box<dyn EventSource>>,
}

impl EventStream {
    /// Create a new event stream with specified buffer capacity
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Buffer capacity must be greater than 0");

        Self {
            buffer: VecDeque::with_capacity(capacity),
            max_capacity: capacity,
            window: None,
            total_processed: 0,
            total_dropped: 0,
            source: None,
        }
    }

    /// Create event stream with default capacity (1000 events)
    pub fn with_default_capacity() -> Self {
        Self::new(1000)
    }

    /// Set the event source for this stream
    pub fn with_source(mut self, source: Box<dyn EventSource>) -> Self {
        self.source = Some(source);
        self
    }

    /// Set window configuration for the stream
    pub fn with_window(mut self, window: WindowType) -> StreamResult<Self> {
        match &window {
            WindowType::TimeBased(ms) if *ms == 0 => {
                return Err(StreamError::InvalidWindow(
                    "Time window must be > 0".to_string(),
                ))
            }
            WindowType::CountBased(count) if *count == 0 => {
                return Err(StreamError::InvalidWindow(
                    "Count window must be > 0".to_string(),
                ))
            }
            WindowType::Sliding { count, time_ms } => {
                if *count == 0 || *time_ms == 0 {
                    return Err(StreamError::InvalidWindow(
                        "Sliding window parameters must be > 0".to_string(),
                    ));
                }
            }
            _ => {}
        }

        self.window = Some(window);
        Ok(self)
    }

    /// Add an event to the stream buffer
    pub fn push(&mut self, event: Event) -> StreamResult<()> {
        if self.buffer.len() >= self.max_capacity {
            self.total_dropped += 1;
            return Err(StreamError::BufferFull(format!(
                "Buffer capacity {} exceeded",
                self.max_capacity
            )));
        }

        self.buffer.push_back(event);
        self.total_processed += 1;
        Ok(())
    }

    /// Get the next event from the buffer
    pub fn next(&mut self) -> Option<Event> {
        self.buffer.pop_front()
    }

    /// Peek at the next event without removing it
    pub fn peek(&self) -> Option<&Event> {
        self.buffer.front()
    }

    /// Get all events currently in buffer
    pub fn peek_all(&self) -> Vec<&Event> {
        self.buffer.iter().collect()
    }

    /// Apply a filter predicate to events in the buffer
    pub fn filter<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(&Event) -> bool,
    {
        let original_len = self.buffer.len();
        self.buffer.retain(|e| predicate(e));
        original_len - self.buffer.len()
    }

    /// Apply a transformation to events in the buffer
    pub fn transform<F>(&mut self, transformer: F)
    where
        F: Fn(&mut Event),
    {
        for event in self.buffer.iter_mut() {
            transformer(event);
        }
    }

    /// Get current buffer statistics
    pub fn stats(&self) -> BufferStats {
        BufferStats {
            current_size: self.buffer.len(),
            max_capacity: self.max_capacity,
            total_processed: self.total_processed,
            total_dropped: self.total_dropped,
        }
    }

    /// Clear all events from the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get current number of events in buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Get current buffer capacity
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }

    /// Drain all events from buffer into a vector
    pub fn drain_all(&mut self) -> Vec<Event> {
        self.buffer.drain(..).collect()
    }

    /// Get sliding window of events
    pub fn window(&self, window_size: usize) -> Vec<&Event> {
        let start = if self.buffer.len() > window_size {
            self.buffer.len() - window_size
        } else {
            0
        };

        self.buffer.iter().skip(start).collect()
    }

    /// Get the current window configuration
    pub fn get_window(&self) -> Option<&WindowType> {
        self.window.as_ref()
    }

    /// Pull next event from source if available
    pub fn pull_from_source(&mut self) -> StreamResult<Option<Event>> {
        match &mut self.source {
            Some(source) => {
                let event = source.next()?;
                if let Some(evt) = &event {
                    self.push(evt.clone())?;
                }
                Ok(event)
            }
            None => Err(StreamError::Custom("No event source configured".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_event(activity: &str) -> Event {
        Event::new(activity, Utc::now())
    }

    #[test]
    fn test_stream_creation() {
        let stream = EventStream::new(100);
        assert_eq!(stream.len(), 0);
        assert_eq!(stream.capacity(), 100);
        assert!(stream.is_empty());
    }

    #[test]
    fn test_push_and_next() {
        let mut stream = EventStream::new(10);
        let event = create_test_event("activity_a");

        assert!(stream.push(event.clone()).is_ok());
        assert_eq!(stream.len(), 1);

        let retrieved = stream.next();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().activity, "activity_a");
        assert!(stream.is_empty());
    }

    #[test]
    fn test_buffer_overflow() {
        let mut stream = EventStream::new(2);
        assert!(stream.push(create_test_event("a")).is_ok());
        assert!(stream.push(create_test_event("b")).is_ok());

        let result = stream.push(create_test_event("c"));
        assert!(result.is_err());
        assert_eq!(stream.stats().total_dropped, 1);
    }

    #[test]
    fn test_peek_operations() {
        let mut stream = EventStream::new(10);
        let event = create_test_event("activity_a");
        stream.push(event.clone()).unwrap();

        let peeked = stream.peek();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().activity, "activity_a");
        assert_eq!(stream.len(), 1);

        let all = stream.peek_all();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_filtering() {
        let mut stream = EventStream::new(10);
        stream.push(create_test_event("a")).unwrap();
        stream.push(create_test_event("b")).unwrap();
        stream.push(create_test_event("a")).unwrap();

        let removed = stream.filter(|e| e.activity != "a");
        assert_eq!(removed, 2);
        assert_eq!(stream.len(), 1);
        assert_eq!(stream.peek().unwrap().activity, "b");
    }

    #[test]
    fn test_window_configuration() {
        let stream = EventStream::new(100);
        let result = stream.with_window(WindowType::CountBased(5));
        assert!(result.is_ok());

        let stream = EventStream::new(100);
        let result = stream.with_window(WindowType::CountBased(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_sliding_window() {
        let mut stream = EventStream::new(10);
        for i in 0..10 {
            stream
                .push(create_test_event(&format!("activity_{}", i)))
                .unwrap();
        }

        let window = stream.window(3);
        assert_eq!(window.len(), 3);
        assert_eq!(window[0].activity, "activity_7");
        assert_eq!(window[2].activity, "activity_9");
    }

    #[test]
    fn test_drain_all() {
        let mut stream = EventStream::new(10);
        stream.push(create_test_event("a")).unwrap();
        stream.push(create_test_event("b")).unwrap();
        stream.push(create_test_event("c")).unwrap();

        let drained = stream.drain_all();
        assert_eq!(drained.len(), 3);
        assert!(stream.is_empty());
    }
}
