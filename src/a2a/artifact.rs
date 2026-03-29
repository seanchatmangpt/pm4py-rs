/// Build A2A Artifact values from tool output strings.
use crate::a2a::protocol::{Artifact, ArtifactPart};

/// Wrap a tool result string in an A2A Artifact.
///
/// If the result is valid JSON, it becomes a `data` part.
/// Otherwise it becomes a `text` part.
pub fn build_artifact(result: String) -> Artifact {
    let part = if let Ok(data) = serde_json::from_str::<serde_json::Value>(&result) {
        ArtifactPart {
            part_type: "data".to_string(),
            text: None,
            data: Some(data),
        }
    } else {
        ArtifactPart {
            part_type: "text".to_string(),
            text: Some(result),
            data: None,
        }
    };
    Artifact { parts: vec![part] }
}
