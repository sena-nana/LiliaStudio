use serde_json::Value;

pub fn normalize_entry_rich_text(value: &str) -> String {
    if value.is_empty() {
        return empty_entry_document();
    }

    let trimmed = value.trim();
    if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
        if is_doc(&parsed) {
            return serde_json::to_string(&parsed).unwrap_or_else(|_| empty_entry_document());
        }
    }

    plain_text_to_entry_document(value)
}

pub fn entry_rich_text_to_plain_text(value: &str) -> String {
    let Ok(parsed) = serde_json::from_str::<Value>(value) else {
        return value.to_string();
    };
    if !is_doc(&parsed) {
        return value.to_string();
    }

    collect_plain_text_blocks(&parsed)
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn entry_searchable_text(title: &str, summary: &str, body: &str, tags: &[String]) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        title,
        entry_rich_text_to_plain_text(summary),
        entry_rich_text_to_plain_text(body),
        tags.join(" ")
    )
}

fn empty_entry_document() -> String {
    r#"{"type":"doc","content":[{"type":"paragraph"}]}"#.to_string()
}

fn is_doc(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("doc")
        && value
            .get("content")
            .is_none_or(|content| content.is_array())
}

fn plain_text_to_entry_document(value: &str) -> String {
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    let content = normalized
        .split('\n')
        .map(|line| {
            if line.is_empty() {
                serde_json::json!({ "type": "paragraph" })
            } else {
                serde_json::json!({
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": line }]
                })
            }
        })
        .collect::<Vec<_>>();
    serde_json::json!({ "type": "doc", "content": content }).to_string()
}

fn collect_plain_text_blocks(value: &Value) -> Vec<String> {
    let node_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let Some(children) = value.get("content").and_then(Value::as_array) else {
        return Vec::new();
    };

    if is_text_block_node(node_type) {
        let inline_text = collect_inline_text(children);
        return if inline_text.trim().is_empty() {
            Vec::new()
        } else {
            vec![inline_text]
        };
    }

    children
        .iter()
        .flat_map(collect_plain_text_blocks)
        .collect()
}

fn is_text_block_node(node_type: &str) -> bool {
    matches!(node_type, "paragraph" | "heading" | "codeBlock")
}

fn collect_inline_text(children: &[Value]) -> String {
    let mut text = String::new();
    for child in children {
        match child.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(value) = child.get("text").and_then(Value::as_str) {
                    text.push_str(value);
                }
            }
            Some("hardBreak") => text.push('\n'),
            _ => {
                if let Some(nested) = child.get("content").and_then(Value::as_array) {
                    text.push_str(&collect_inline_text(nested));
                }
            }
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::{entry_rich_text_to_plain_text, normalize_entry_rich_text};

    #[test]
    fn normalizes_empty_and_plain_text_values() {
        assert_eq!(
            normalize_entry_rich_text(""),
            r#"{"type":"doc","content":[{"type":"paragraph"}]}"#
        );

        let value = normalize_entry_rich_text("  第一行\n第二行\n");

        assert_eq!(entry_rich_text_to_plain_text(&value), "  第一行\n第二行");
    }

    #[test]
    fn keeps_doc_json_and_wraps_other_json_as_plain_text() {
        let document = r#"{"type":"doc","content":[{"type":"heading","attrs":{"level":2},"content":[{"type":"text","marks":[{"type":"bold"}],"text":"已存在"}]}]}"#;

        let normalized = normalize_entry_rich_text(document);
        assert!(normalized.contains(r#""type":"doc""#));
        assert_eq!(entry_rich_text_to_plain_text(&normalized), "已存在");

        for raw in [
            r#"{"type":"notDoc","content":"说明"}"#,
            r#"{"type":"doc","content":"说明"}"#,
        ] {
            let value = normalize_entry_rich_text(raw);

            assert_eq!(entry_rich_text_to_plain_text(&value), raw);
        }
    }

    #[test]
    fn falls_back_to_plain_text_for_invalid_json() {
        assert_eq!(entry_rich_text_to_plain_text("不是 JSON"), "不是 JSON");
    }

    #[test]
    fn extracts_plain_text_from_known_blocks() {
        let value = r#"{
            "type":"doc",
            "content":[
                {"type":"heading","content":[{"type":"text","text":"标题"}]},
                {"type":"bulletList","content":[{
                    "type":"listItem",
                    "content":[
                        {"type":"paragraph","content":[{"type":"text","text":"外层"}]},
                        {"type":"orderedList","content":[{
                            "type":"listItem",
                            "content":[{"type":"paragraph","content":[{"type":"text","text":"内层"}]}]
                        }]}
                    ]
                }]},
                {"type":"codeBlock","content":[{"type":"text","text":"moon_gate();"}]},
                {"type":"paragraph","content":[
                    {"type":"text","text":"第一行"},
                    {"type":"hardBreak"},
                    {"type":"text","text":"第二行"}
                ]}
            ]
        }"#;

        assert_eq!(
            entry_rich_text_to_plain_text(value),
            "标题\n外层\n内层\nmoon_gate();\n第一行\n第二行"
        );
    }
}
