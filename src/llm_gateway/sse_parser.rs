use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    pub done: bool,
    pub content_delta: Option<String>,
    pub completion_tokens: Option<usize>,
}

pub fn parse_event_line(line: &str) -> Result<Option<SseEvent>> {
    let line = line.trim();
    if line.is_empty() || line.starts_with(':') {
        return Ok(None);
    }
    let Some(data) = line.strip_prefix("data:") else {
        return Ok(None);
    };
    let data = data.trim();
    if data == "[DONE]" {
        return Ok(Some(SseEvent {
            done: true,
            content_delta: None,
            completion_tokens: None,
        }));
    }
    let value: Value = serde_json::from_str(data).context("parse SSE data JSON")?;
    Ok(Some(SseEvent {
        done: false,
        content_delta: content_delta(&value),
        completion_tokens: value
            .pointer("/usage/completion_tokens")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
    }))
}

pub fn parse_chunk(chunk: &str) -> Result<Vec<SseEvent>> {
    chunk
        .lines()
        .filter_map(|line| parse_event_line(line).transpose())
        .collect()
}

fn content_delta(value: &Value) -> Option<String> {
    value
        .pointer("/choices/0/delta/content")
        .and_then(Value::as_str)
        .or_else(|| {
            value
                .pointer("/choices/0/message/content")
                .and_then(Value::as_str)
        })
        .or_else(|| value.pointer("/choices/0/text").and_then(Value::as_str))
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openai_compatible_sse_frames() -> Result<()> {
        let events = parse_chunk(
            "event: ignored\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"hel\"}}]}\n\n\
             data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}],\"usage\":{\"completion_tokens\":2}}\n\
             data: [DONE]\n",
        )?;

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].content_delta.as_deref(), Some("hel"));
        assert_eq!(events[1].content_delta.as_deref(), Some("lo"));
        assert_eq!(events[1].completion_tokens, Some(2));
        assert!(events[2].done);
        Ok(())
    }
}
