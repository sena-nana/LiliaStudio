use crate::vector::chunking::{chunk_text, TextChunk};

#[tauri::command]
pub fn preview_chunks(text: String, max_chars: usize) -> Vec<TextChunk> {
    chunk_text(&text, max_chars)
}
