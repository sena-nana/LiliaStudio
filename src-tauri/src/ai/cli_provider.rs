use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CliTemplateError {
    #[error("unterminated quote in command template")]
    UnterminatedQuote,
}

pub fn render_command_template(
    template: &str,
    workspace: &str,
    prompt: &str,
    max_turns: u32,
    output_format: &str,
) -> Result<String, CliTemplateError> {
    Ok(template
        .replace("{workspace}", workspace)
        .replace("{prompt}", prompt)
        .replace("{max_turns}", &max_turns.to_string())
        .replace("{output_format}", output_format))
}

pub fn split_command_line(command: &str) -> Result<Vec<String>, CliTemplateError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = command.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => in_quotes = !in_quotes,
            '\\' if chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            ch if ch.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    parts.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if in_quotes {
        return Err(CliTemplateError::UnterminatedQuote);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    Ok(parts)
}
