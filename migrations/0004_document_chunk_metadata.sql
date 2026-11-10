ALTER TABLE document_chunks
ADD COLUMN estimated_tokens INTEGER NOT NULL DEFAULT 0;
