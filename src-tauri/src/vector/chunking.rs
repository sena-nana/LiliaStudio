use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TextChunk {
    pub ordinal: usize,
    pub text: String,
    pub content_hash: String,
}

pub fn chunk_text(text: &str, max_chars: usize) -> Vec<TextChunk> {
    let mut chunks = Vec::new();
    let mut ordinal = 0;

    for paragraph in text.split("\n\n").map(str::trim).filter(|part| !part.is_empty()) {
        if paragraph.chars().count() <= max_chars {
            chunks.push(make_chunk(ordinal, paragraph));
            ordinal += 1;
            continue;
        }

        let mut buffer = String::new();
        for ch in paragraph.chars() {
            buffer.push(ch);
            if buffer.chars().count() >= max_chars {
                chunks.push(make_chunk(ordinal, buffer.trim()));
                ordinal += 1;
                buffer.clear();
            }
        }
        if !buffer.trim().is_empty() {
            chunks.push(make_chunk(ordinal, buffer.trim()));
            ordinal += 1;
        }
    }

    chunks
}

fn make_chunk(ordinal: usize, text: &str) -> TextChunk {
    TextChunk {
        ordinal,
        text: text.to_string(),
        content_hash: format!("{:016x}", stable_hash(text)),
    }
}

fn stable_hash(text: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in text.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
