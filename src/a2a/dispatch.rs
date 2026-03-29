/// A2A message → tool dispatch.
///
/// Extracts the tool name and argument payload from an A2A message's parts array.
/// Part::Data is checked first; Part::Text is accepted as a JSON fallback.
use serde_json::Value;

/// Parse an A2A message object and return `(tool_name, args)`.
///
/// Expected message shape:
/// ```json
/// {
///   "role": "user",
///   "parts": [{ "type": "data", "data": { "tool": "pm4py_statistics", "args": {...} } }]
/// }
/// ```
///
/// Returns `None` if no usable part is found.
pub fn parse_message_command(message: &Value) -> Option<(String, Value)> {
    let parts = message.get("parts")?.as_array()?;

    // Prefer Part::Data — structured invocation
    for part in parts {
        if part.get("type").and_then(|t| t.as_str()) == Some("data") {
            let data = part.get("data")?;
            let tool = data.get("tool")?.as_str()?.to_string();
            let args = data
                .get("args")
                .cloned()
                .unwrap_or(Value::Object(Default::default()));
            return Some((tool, args));
        }
    }

    // Fallback: Part::Text — JSON-encoded invocation
    for part in parts {
        if part.get("type").and_then(|t| t.as_str()) == Some("text") {
            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                if let Ok(parsed) = serde_json::from_str::<Value>(text) {
                    if let Some(tool) = parsed.get("tool").and_then(|t| t.as_str()) {
                        let args = parsed
                            .get("args")
                            .cloned()
                            .unwrap_or(Value::Object(Default::default()));
                        return Some((tool.to_string(), args));
                    }
                }
            }
        }
    }

    None
}
