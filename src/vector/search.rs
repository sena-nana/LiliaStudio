pub fn cosine_similarity(left: &[f32], right: &[f32]) -> Result<f32, String> {
    if left.len() != right.len() {
        return Err("vectors must have the same dimension".into());
    }
    if left.is_empty() {
        return Err("vectors must not be empty".into());
    }

    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;
    for (left_value, right_value) in left.iter().zip(right.iter()) {
        dot += left_value * right_value;
        left_norm += left_value * left_value;
        right_norm += right_value * right_value;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        return Ok(0.0);
    }

    Ok(dot / (left_norm.sqrt() * right_norm.sqrt()))
}
