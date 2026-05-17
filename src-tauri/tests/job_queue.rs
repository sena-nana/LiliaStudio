use ameya_lib::{
    db::{connection::open_memory_database, migrations::run_migrations},
    services::jobs::{
        append_job_log, cancel_job, complete_job_success, create_queued_job, current_running_job,
        fail_job, list_job_logs, mark_job_running, retry_job, AiJobDraft,
    },
};

#[test]
fn job_queue_tracks_status_lifecycle_and_current_running_job() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");
    let job = create_queued_job(&connection, draft()).expect("queued job");

    assert_eq!(job.status, "queued");
    assert!(job.started_at.is_none());

    let running = mark_job_running(&connection, &job.id).expect("running job");
    assert_eq!(running.status, "running");
    assert!(running.started_at.is_some());
    assert_eq!(
        current_running_job(&connection)
            .expect("current job")
            .expect("running")
            .id,
        job.id
    );

    let completed =
        complete_job_success(&connection, &job.id, "结构化结果".into()).expect("completed job");
    assert_eq!(completed.status, "succeeded");
    assert_eq!(completed.output_text, "结构化结果");
    assert!(completed.finished_at.is_some());
    assert!(current_running_job(&connection)
        .expect("current job")
        .is_none());
}

#[test]
fn job_logs_redact_secrets_from_messages() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");
    let job = create_queued_job(&connection, draft()).expect("queued job");

    append_job_log(
        &connection,
        &job.id,
        "error",
        "Authorization: Bearer sk-live-secret-1234",
    )
    .expect("log append");

    let logs = list_job_logs(&connection, &job.id).expect("logs");
    let combined = logs
        .iter()
        .map(|log| log.message.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!combined.contains("sk-live-secret-1234"));
    assert!(combined.contains("[redacted]"));
}

#[test]
fn job_queue_can_cancel_running_jobs_and_retry_from_previous_input() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");
    let job = create_queued_job(&connection, draft()).expect("queued job");
    mark_job_running(&connection, &job.id).expect("running job");

    let cancelled = cancel_job(&connection, &job.id).expect("cancelled job");
    assert_eq!(cancelled.status, "cancelled");
    assert!(cancelled.cancel_requested_at.is_some());
    assert!(cancelled.finished_at.is_some());

    let retry = retry_job(&connection, &job.id).expect("retry job");
    assert_ne!(retry.id, job.id);
    assert_eq!(retry.status, "queued");
    assert_eq!(retry.retry_of_job_id.as_deref(), Some(job.id.as_str()));
    assert_eq!(retry.provider_kind, job.provider_kind);
    assert_eq!(retry.input_summary, job.input_summary);
}

#[test]
fn failed_jobs_store_sanitized_error_message() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");
    let job = create_queued_job(&connection, draft()).expect("queued job");

    let failed =
        fail_job(&connection, &job.id, "api_key=sk-live-secret-1234".into()).expect("failed job");

    assert_eq!(failed.status, "failed");
    let message = failed.error_message.expect("error message");
    assert!(!message.contains("sk-live-secret-1234"));
    assert!(message.contains("[redacted]"));
}

#[test]
fn unknown_jobs_return_errors_without_panicking() {
    let mut connection = open_memory_database().expect("memory db");
    run_migrations(&mut connection).expect("migrations");

    assert!(mark_job_running(&connection, "job_missing").is_err());
    assert!(retry_job(&connection, "job_missing").is_err());
}

fn draft() -> AiJobDraft {
    AiJobDraft {
        project_id: Some("project_1".into()),
        provider_kind: "codexCli".into(),
        job_type: "logicAudit".into(),
        input_summary: "审计角色行为".into(),
    }
}
