UPDATE entries
SET summary = normalize_entry_rich_text(summary),
    body = normalize_entry_rich_text(body);
