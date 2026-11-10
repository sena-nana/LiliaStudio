//! TipTap JSON ⇄ NanaUI RichText 双向转换器（纯函数，无状态依赖）。
//!
//! 存量契约来自 Tauri 版 `src/components/richTextContent.ts`：合法内容是
//! TipTap 文档 JSON `{"type":"doc","content":[...]}`；空串与非 doc JSON 按
//! 纯文本包裹规则归一为段落（CRLF 归一、按 `\n` 分段）。RichText 侧 API 见
//! `plans/audit/nanaui-api-reference.md` §4：spans 为字节区间，字符边界对齐
//! 由框架保证，本模块只负责拆分文本段。

use std::collections::BTreeSet;
use std::ops::Range;

use nana_ui::runtime::rich::{RichSpanStyle, RichText};

/// 与存量 `EMPTY_DOCUMENT` 常量一致。
pub const EMPTY_DOCUMENT: &str = r#"{"type":"doc","content":[{"type":"paragraph"}]}"#;

/// TipTap 文档 JSON → NanaUI RichText。非 JSON 或非 doc 结构的输入按
/// `richTextContent.ts` 的纯文本包裹规则转换。
pub fn tiptap_to_rich(value: &str) -> RichText {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return RichText::new("");
    }
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(blocks) = doc_blocks(&parsed) {
            let mut chunks = Vec::new();
            collect_blocks(blocks, &mut chunks);
            return chunks_to_rich(chunks);
        }
    }
    plain_text_to_rich(value)
}

/// NanaUI RichText → TipTap 文档 JSON 字符串（与存量数据格式兼容；
/// marks 只还原 bold/italic/strike，分段按 `\n`）。
pub fn rich_to_tiptap(rich: &RichText) -> String {
    let text = rich.text();
    let runs: Vec<(Range<usize>, &RichSpanStyle)> = rich.spans().iter().collect();
    let mut paragraphs = Vec::new();
    let mut offset = 0usize;
    for line in text.split('\n') {
        let line_range = offset..offset + line.len();
        offset = line_range.end + 1;
        paragraphs.push(paragraph_json(line, &line_range, &runs));
    }
    format!(
        "{{\"type\":\"doc\",\"content\":[{}]}}",
        paragraphs.join(",")
    )
}

// ---- tiptap → rich ----

struct Chunk {
    text: String,
    style: Option<RichSpanStyle>,
}

fn plain_text_to_rich(value: &str) -> RichText {
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    RichText::new(normalized)
}

static EMPTY_BLOCKS: Vec<serde_json::Value> = Vec::new();

fn doc_blocks(value: &serde_json::Value) -> Option<&[serde_json::Value]> {
    let object = value.as_object()?;
    if object.get("type").and_then(|v| v.as_str()) != Some("doc") {
        return None;
    }
    match object.get("content") {
        None => Some(&EMPTY_BLOCKS),
        Some(serde_json::Value::Array(items)) => Some(items),
        _ => None,
    }
}

/// heading level → 相对字号/字重。用 `size_em` 保持与继承字号无关。
fn heading_style(level: u64) -> RichSpanStyle {
    let em = match level {
        1 => 2.0,
        2 => 1.5,
        3 => 1.25,
        4 => 1.1,
        5 => 1.0,
        _ => 0.9,
    };
    RichSpanStyle::new().size_em(em).weight(700)
}

fn apply_marks(mut style: RichSpanStyle, marks: &[serde_json::Value]) -> RichSpanStyle {
    for mark in marks {
        match mark.get("type").and_then(|v| v.as_str()) {
            Some("bold") => style = style.bold(),
            Some("italic") => style = style.italic(true),
            Some("strike") => style = style.line_through(),
            _ => {}
        }
    }
    style
}

fn line_break() -> Chunk {
    Chunk {
        text: "\n".to_string(),
        style: None,
    }
}

fn collect_blocks(blocks: &[serde_json::Value], out: &mut Vec<Chunk>) {
    for (index, block) in blocks.iter().enumerate() {
        if index > 0 {
            out.push(line_break());
        }
        collect_block(block, out);
    }
}

fn collect_block(node: &serde_json::Value, out: &mut Vec<Chunk>) {
    let kind = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match kind {
        "paragraph" => collect_inline(node, RichSpanStyle::new(), out),
        "heading" => {
            let level = node.get("level").and_then(|v| v.as_u64()).unwrap_or(1);
            collect_inline(node, heading_style(level), out);
        }
        "blockquote" => {
            if let Some(blocks) = node.get("content").and_then(|v| v.as_array()) {
                for (index, block) in blocks.iter().enumerate() {
                    if index > 0 {
                        out.push(line_break());
                    }
                    out.push(Chunk {
                        text: "❝ ".to_string(),
                        style: None,
                    });
                    collect_block(block, out);
                }
            }
        }
        // 列表项转纯文本行（不带序号/圆点前缀，语义由展示层处理）。
        "bulletList" | "orderedList" => {
            if let Some(items) = node.get("content").and_then(|v| v.as_array()) {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        out.push(line_break());
                    }
                    collect_block(item, out);
                }
            }
        }
        "listItem" | "doc" => {
            if let Some(blocks) = node.get("content").and_then(|v| v.as_array()) {
                collect_blocks(blocks, out);
            }
        }
        // 未知节点降级：容器按块递归抽取文本，叶子带 text 字段则取其文本。
        _ => {
            if let Some(blocks) = node.get("content").and_then(|v| v.as_array()) {
                if !blocks.is_empty() {
                    collect_blocks(blocks, out);
                    return;
                }
            }
            collect_inline(node, RichSpanStyle::new(), out);
        }
    }
}

fn collect_inline(node: &serde_json::Value, base: RichSpanStyle, out: &mut Vec<Chunk>) {
    let kind = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match kind {
        "text" => {
            let text = node.get("text").and_then(|v| v.as_str()).unwrap_or("");
            if text.is_empty() {
                return;
            }
            let style = match node.get("marks").and_then(|v| v.as_array()) {
                Some(marks) => apply_marks(base.clone(), marks),
                None => base.clone(),
            };
            out.push(Chunk {
                text: text.to_string(),
                style: (!style.is_empty()).then_some(style),
            });
        }
        "hardBreak" => out.push(line_break()),
        _ => {
            if let Some(children) = node.get("content").and_then(|v| v.as_array()) {
                for child in children {
                    collect_inline(child, base.clone(), out);
                }
            } else if let Some(text) = node.get("text").and_then(|v| v.as_str()) {
                if !text.is_empty() {
                    out.push(Chunk {
                        text: text.to_string(),
                        style: (!base.is_empty()).then_some(base),
                    });
                }
            }
        }
    }
}

fn chunks_to_rich(chunks: Vec<Chunk>) -> RichText {
    if chunks.is_empty() {
        return RichText::new("");
    }
    let mut builder = RichText::builder();
    for chunk in chunks {
        if chunk.text.is_empty() {
            continue;
        }
        match chunk.style {
            Some(style) if !style.is_empty() => {
                builder.push(&chunk.text, style);
            }
            _ => {
                builder.plain(&chunk.text);
            }
        }
    }
    builder.build()
}

// ---- rich → tiptap ----

fn marks_of(style: &RichSpanStyle) -> Vec<&'static str> {
    let mut marks = Vec::new();
    if style.shape.weight.map_or(false, |weight| weight >= 600) {
        marks.push("bold");
    }
    if style.shape.italic == Some(true) {
        marks.push("italic");
    }
    if style
        .paint
        .decoration
        .map_or(false, |decoration| decoration.line_through)
    {
        marks.push("strike");
    }
    marks
}

fn paragraph_json(
    line: &str,
    range: &Range<usize>,
    runs: &[(Range<usize>, &RichSpanStyle)],
) -> String {
    let mut bounds: BTreeSet<usize> = [range.start, range.end].into_iter().collect();
    for (run, _) in runs {
        if run.start > range.start && run.start < range.end {
            bounds.insert(run.start);
        }
        if run.end > range.start && run.end < range.end {
            bounds.insert(run.end);
        }
    }
    let points: Vec<usize> = bounds.into_iter().collect();
    let mut parts = Vec::new();
    for window in points.windows(2) {
        let (start, end) = (window[0], window[1]);
        let segment = &line[start - range.start..end - range.start];
        if segment.is_empty() {
            continue;
        }
        let marks = runs
            .iter()
            .find(|(run, _)| run.start <= start && start < run.end)
            .map_or_else(Vec::new, |(_, style)| marks_of(style));
        parts.push(text_node_json(segment, &marks));
    }
    if parts.is_empty() {
        return r#"{"type":"paragraph"}"#.to_string();
    }
    format!(r#"{{"type":"paragraph","content":[{}]}}"#, parts.join(","))
}

fn text_node_json(text: &str, marks: &[&str]) -> String {
    let encoded = serde_json::to_string(text).unwrap_or_default();
    if marks.is_empty() {
        return format!(r#"{{"type":"text","text":{encoded}}}"#);
    }
    let list = marks
        .iter()
        .map(|mark| format!(r#"{{"type":"{mark}"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"type":"text","text":{encoded},"marks":[{list}]}}"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marks_at(rich: &RichText, byte: usize) -> Vec<&'static str> {
        rich.spans().at(byte).map_or_else(Vec::new, marks_of)
    }

    #[test]
    fn roundtrip_preserves_text_and_marks() {
        let mut builder = RichText::builder();
        builder.plain("今天也");
        builder.push("辛苦了", RichSpanStyle::new().bold().italic(true));
        builder.plain("，明天见。");
        let rich = builder.build();

        let tiptap = rich_to_tiptap(&rich);
        let back = tiptap_to_rich(&tiptap);

        assert_eq!(back.text(), "今天也辛苦了，明天见。");
        let hard = rich.text().find("辛苦了").unwrap();
        for offset in 0..rich.text().len() {
            assert_eq!(
                marks_at(&back, offset),
                marks_at(&rich, offset),
                "offset {offset}"
            );
        }
        assert!(hard > 0);
    }

    #[test]
    fn roundtrip_multiline_with_strike() {
        let mut builder = RichText::builder();
        builder.plain("第一行");
        builder.push("重点", RichSpanStyle::new().line_through());
        builder.plain("\n第二行");
        let rich = builder.build();

        let back = tiptap_to_rich(&rich_to_tiptap(&rich));
        assert_eq!(back.text(), "第一行重点\n第二行");
        let hit = rich.text().find("重点").unwrap();
        assert_eq!(marks_at(&back, hit), vec!["strike"]);
        let plain = rich.text().find("第二行").unwrap();
        assert!(marks_at(&back, plain).is_empty());
    }

    #[test]
    fn plain_text_is_wrapped_like_legacy_contract() {
        let rich = tiptap_to_rich("第一行\n第二行\r\n第三行");
        assert_eq!(rich.text(), "第一行\n第二行\n第三行");
        assert!(rich.spans().is_empty());

        let tiptap = rich_to_tiptap(&rich);
        assert_eq!(
            tiptap,
            concat!(
                r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"第一行"}]},"#,
                r#"{"type":"paragraph","content":[{"type":"text","text":"第二行"}]},"#,
                r#"{"type":"paragraph","content":[{"type":"text","text":"第三行"}]}]}"#
            )
        );
    }

    #[test]
    fn non_doc_json_falls_back_to_plain_text() {
        let rich = tiptap_to_rich(r#"{"type":"paragraph","content":[{"type":"text","text":"x"}]}"#);
        assert_eq!(
            rich.text(),
            r#"{"type":"paragraph","content":[{"type":"text","text":"x"}]}"#
        );
    }

    #[test]
    fn blocks_and_marks_convert() {
        let doc = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "heading", "level": 2, "content": [{"type": "text", "text": "标题"}]},
                {"type": "paragraph", "content": [
                    {"type": "text", "text": "加粗", "marks": [{"type": "bold"}]},
                    {"type": "text", "text": "斜体", "marks": [{"type": "italic"}]}
                ]},
                {"type": "paragraph", "content": [
                    {"type": "text", "text": "旧文", "marks": [{"type": "strike"}]}
                ]},
                {"type": "blockquote", "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "引用"}]}
                ]},
                {"type": "bulletList", "content": [
                    {"type": "listItem", "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "条目"}]}
                    ]}
                ]},
                {"type": "paragraph", "content": [
                    {"type": "text", "text": "a"},
                    {"type": "hardBreak"},
                    {"type": "text", "text": "b"}
                ]}
            ]
        });
        let rich = tiptap_to_rich(&doc.to_string());
        assert_eq!(rich.text(), "标题\n加粗斜体\n旧文\n❝ 引用\n条目\na\nb");

        let title = rich.text().find("标题").unwrap();
        let title_span = rich.spans().at(title).expect("heading styled");
        assert_eq!(title_span.shape.weight, Some(700));
        assert_eq!(title_span.shape.size_em, Some(1.5));

        let bold = rich.text().find("加粗").unwrap();
        assert_eq!(marks_at(&rich, bold), vec!["bold"]);
        let italic = rich.text().find("斜体").unwrap();
        assert_eq!(marks_at(&rich, italic), vec!["italic"]);
        let strike = rich.text().find("旧文").unwrap();
        assert_eq!(marks_at(&rich, strike), vec!["strike"]);
        let quote = rich.text().find("引用").unwrap();
        assert!(marks_at(&rich, quote).is_empty());
    }

    #[test]
    fn unknown_nodes_degrade_to_plain_text() {
        let doc = serde_json::json!({
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "前"}]},
                {"type": "fancyCard", "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "卡片"}]}
                ]},
                {"type": "paragraph", "content": [{"type": "text", "text": "后"}]}
            ]
        });
        let rich = tiptap_to_rich(&doc.to_string());
        assert_eq!(rich.text(), "前\n卡片\n后");
    }

    #[test]
    fn empty_inputs_match_legacy_empty_document() {
        assert_eq!(rich_to_tiptap(&RichText::new("")), EMPTY_DOCUMENT);
        assert_eq!(tiptap_to_rich("").text(), "");
        assert!(tiptap_to_rich("").spans().is_empty());
        assert_eq!(tiptap_to_rich(EMPTY_DOCUMENT).text(), "");
        assert!(tiptap_to_rich("   \n  ").text().is_empty());
    }

    #[test]
    fn legacy_storage_shape_is_stable() {
        let mut builder = RichText::builder();
        builder.push("粗", RichSpanStyle::new().bold());
        let tiptap = rich_to_tiptap(&builder.build());
        assert_eq!(
            tiptap,
            r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"粗","marks":[{"type":"bold"}]}]}]}"#
        );
        assert!(serde_json::from_str::<serde_json::Value>(&tiptap).is_ok());
    }

    #[test]
    fn text_is_json_escaped() {
        let rich = RichText::new("说\"引号\"\n与\\斜杠");
        let tiptap = rich_to_tiptap(&rich);
        assert!(tiptap.contains(r#"\"引号\""#));
        assert!(tiptap.contains(r#"\\斜杠"#));
        let back = tiptap_to_rich(&tiptap);
        assert_eq!(back.text(), "说\"引号\"\n与\\斜杠");
    }
}
