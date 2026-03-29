/// Context compression — summarizing or truncating conversation history to fit context window.
///
/// Span: `span.conversation.compress`
/// Kind: `internal`
/// Stability: `development`
pub const CONVERSATION_COMPRESS_SPAN: &str = "conversation.compress";
/// Conversation session initialization — first turn, context loaded.
///
/// Span: `span.conversation.start`
/// Kind: `internal`
/// Stability: `development`
pub const CONVERSATION_START_SPAN: &str = "conversation.start";
/// Single conversation turn — user message received, assistant response generated.
///
/// Span: `span.conversation.turn`
/// Kind: `internal`
/// Stability: `development`
pub const CONVERSATION_TURN_SPAN: &str = "conversation.turn";
