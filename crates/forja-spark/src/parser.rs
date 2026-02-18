use crate::events::ChatEvent;

/// Parse a single line of `claude --output-format stream-json` output
/// into a `ChatEvent`.
///
/// Returns `None` for lines that don't map to a known event type
/// (e.g. heartbeats, unknown fields, blank lines).
///
/// # Implementation note
///
/// The exact stream-json format needs to be captured from a real
/// `claude --output-format stream-json --print -- "hello"` invocation
/// during Phase 4. This skeleton provides the API contract.
pub fn parse_line(line: &str) -> Option<ChatEvent> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let value: serde_json::Value = serde_json::from_str(line).ok()?;

    // TODO: Map actual claude stream-json event types to ChatEvent variants.
    // The field names below are provisional — replace with real ones after
    // investigating the actual output format.
    let event_type = value.get("type")?.as_str()?;

    match event_type {
        "message_start" => Some(ChatEvent::MessageStart {
            id: json_str(&value, "id"),
            role: json_str(&value, "role"),
        }),
        "content_block_delta" => {
            let delta = value.get("delta")?;
            Some(ChatEvent::TextDelta {
                index: value.get("index")?.as_u64()? as usize,
                text: delta.get("text")?.as_str()?.to_string(),
            })
        }
        "content_block_stop" => Some(ChatEvent::ContentBlockStop {
            index: value.get("index")?.as_u64()? as usize,
        }),
        "message_stop" => Some(ChatEvent::MessageStop),
        "error" => Some(ChatEvent::Error {
            message: json_str(&value, "message"),
        }),
        _ => None,
    }
}

fn json_str(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_line_returns_none() {
        assert!(parse_line("").is_none());
        assert!(parse_line("   ").is_none());
    }

    #[test]
    fn invalid_json_returns_none() {
        assert!(parse_line("not json").is_none());
    }

    #[test]
    fn unknown_event_type_returns_none() {
        assert!(parse_line(r#"{"type": "unknown_thing"}"#).is_none());
    }

    #[test]
    fn parse_message_stop() {
        let event = parse_line(r#"{"type": "message_stop"}"#);
        assert!(matches!(event, Some(ChatEvent::MessageStop)));
    }

    #[test]
    fn parse_error_event() {
        let event = parse_line(r#"{"type": "error", "message": "rate limited"}"#);
        match event {
            Some(ChatEvent::Error { message }) => assert_eq!(message, "rate limited"),
            other => panic!("expected Error, got {:?}", other),
        }
    }
}
