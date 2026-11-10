ALTER TABLE ai_jobs ADD COLUMN started_at TEXT;
ALTER TABLE ai_jobs ADD COLUMN finished_at TEXT;
ALTER TABLE ai_jobs ADD COLUMN cancel_requested_at TEXT;
ALTER TABLE ai_jobs ADD COLUMN retry_of_job_id TEXT;

CREATE TABLE IF NOT EXISTS ai_job_logs (
  id TEXT PRIMARY KEY,
  job_id TEXT NOT NULL,
  level TEXT NOT NULL,
  message TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (job_id) REFERENCES ai_jobs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ai_job_logs_job
ON ai_job_logs(job_id, created_at);

CREATE INDEX IF NOT EXISTS idx_ai_jobs_status_updated
ON ai_jobs(status, updated_at);
