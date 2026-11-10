use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    db::app_state::AppState,
    domain::{
        axiom::{AxiomDraft, AxiomRepository},
        character::{CharacterDraft, CharacterRepository},
        entry::{EntryDraft, EntryRepository},
        project::{ProjectDraft, ProjectRepository},
        relation::{RelationDraft, RelationRepository},
    },
    services::library::load_project_snapshot,
};

use crate::secrets;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HealthInfo {
    pub app_version: String,
    pub platform: String,
    pub app_data_dir: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenAiProviderTestResult {
    ok: bool,
    message: String,
    error: Option<ameya_core::ai::openai_compatible::ProviderError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CliProviderTestResult {
    ok: bool,
    message: String,
    error: Option<ameya_core::ai::cli_provider::CliProviderError>,
    output: Option<String>,
}

fn openai_error_result(
    error: ameya_core::ai::openai_compatible::ProviderError,
) -> OpenAiProviderTestResult {
    OpenAiProviderTestResult {
        ok: false,
        message: error.message.clone(),
        error: Some(error),
    }
}

fn cli_test_ok(message: &str, output: String) -> CliProviderTestResult {
    CliProviderTestResult {
        ok: true,
        message: message.into(),
        error: None,
        output: Some(output),
    }
}

fn cli_test_err(error: ameya_core::ai::cli_provider::CliProviderError) -> CliProviderTestResult {
    CliProviderTestResult {
        ok: false,
        message: error.message.clone(),
        error: Some(error),
        output: None,
    }
}

fn semantic_response(
    status: crate::services::search::SemanticSearchStatus,
    message: String,
    model: String,
    items: Vec<crate::services::search::SearchResult>,
) -> Result<Value, String> {
    serde_json::to_value(crate::services::search::SemanticSearchResponse {
        status,
        message,
        model,
        items,
    })
    .map_err(|error| error.to_string())
}

/// Nana's command boundary. It intentionally keeps the frontend command names
/// stable while exposing them through the native host API.
#[derive(Clone)]
pub struct NativeCommandHost {
    state: Arc<AppState>,
    data_dir: PathBuf,
    config_dir: Option<PathBuf>,
    secrets: secrets::SharedSecretStore,
}

impl NativeCommandHost {
    pub fn initialize(data_dir: PathBuf) -> Result<Self, String> {
        Self::initialize_with_config(data_dir, None)
    }

    /// Initializes the host with an explicit config directory for AI provider
    /// settings (`ai-provider-settings.json` + secret store). Passing `None`
    /// keeps settings next to the data directory (useful for tests).
    pub fn initialize_with_config(
        data_dir: PathBuf,
        config_dir: Option<PathBuf>,
    ) -> Result<Self, String> {
        let state = AppState::initialize(data_dir.clone()).map_err(|error| error.to_string())?;
        Ok(Self {
            state: Arc::new(state),
            data_dir,
            config_dir,
            secrets: secrets::SharedSecretStore::new(secrets::SystemCredentialStore),
        })
    }

    fn config_dir_path(&self) -> &std::path::Path {
        self.config_dir.as_deref().unwrap_or(&self.data_dir)
    }

    fn load_openai_settings(
        &self,
    ) -> Result<
        (
            Vec<ameya_core::ai::settings::AiProviderSettingsView>,
            Option<String>,
        ),
        String,
    > {
        ameya_core::ai::provider_resolver::load_openai_settings(
            self.config_dir_path(),
            &self.secrets,
        )
    }

    fn embedding_status(
        &self,
        project_id: &str,
        model: &str,
    ) -> Result<crate::services::rag::RagIndexStatus, String> {
        self.state.with_database(|connection| {
            crate::services::rag::embedding_index_status(connection, project_id, Some(model))
        })
    }

    fn cli_command_template(
        &self,
        kind: ameya_core::ai::settings::AiProviderKind,
        label: &str,
    ) -> Result<String, String> {
        use ameya_core::ai::settings::{AiProviderConfig, AiProviderKind, AiProviderSettingsView};

        let settings = ameya_core::ai::settings::load_provider_settings(
            self.config_dir_path(),
            &self.secrets,
        )?;
        let provider = settings
            .into_iter()
            .find(|provider: &AiProviderSettingsView| provider.kind == kind)
            .ok_or_else(|| format!("未找到 {label} Provider 设置"))?;
        let default = match kind {
            AiProviderKind::CodexCli => AiProviderConfig::codex_cli().command_template,
            _ => AiProviderConfig::claude_cli().command_template,
        };
        provider
            .command_template
            .or(default)
            .ok_or_else(|| format!("未找到 {label} 命令模板"))
    }

    pub fn invoke(&self, command: &str, args: Value) -> Result<Value, String> {
        match command {
            "health_check" => Ok(serde_json::to_value(HealthInfo {
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                platform: std::env::consts::OS.to_string(),
                app_data_dir: self.data_dir.to_string_lossy().to_string(),
            })
            .map_err(|error| error.to_string())?),
            "project_list" => self.state.with_database(|connection| {
                serde_json::to_value(ProjectRepository::new(connection).list_active()?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
            }),
            "project_create" => {
                let draft: ProjectDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid project draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(ProjectRepository::new(connection).create(draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "project_update" => {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                let draft: ProjectDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid project draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(ProjectRepository::new(connection).update(id, draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "project_archive" => {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                self.state.with_database(|connection| {
                    ProjectRepository::new(connection).archive(id)?;
                    Ok(Value::Null)
                })
            }
            "library_create_entry" => {
                let draft: EntryDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid entry draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(EntryRepository::new(connection).create(draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_update_entry" => {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or("missing entry id")?;
                let draft: EntryDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid entry draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(EntryRepository::new(connection).update(id, draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_delete_entry" => self.state.with_database(|connection| {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
                EntryRepository::new(connection).soft_delete(id)?;
                Ok(Value::Null)
            }),
            "library_create_character" => {
                let draft: CharacterDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid character draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(CharacterRepository::new(connection).create(draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_update_character" => {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or("missing character id")?;
                let draft: CharacterDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid character draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(CharacterRepository::new(connection).update(id, draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_delete_character" => self.state.with_database(|connection| {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
                CharacterRepository::new(connection).soft_delete(id)?;
                Ok(Value::Null)
            }),
            "library_create_axiom" => {
                let draft: AxiomDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid axiom draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(AxiomRepository::new(connection).create(draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_update_axiom" => {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or("missing axiom id")?;
                let draft: AxiomDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid axiom draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(AxiomRepository::new(connection).update(id, draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_delete_axiom" => self.state.with_database(|connection| {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
                AxiomRepository::new(connection).soft_delete(id)?;
                Ok(Value::Null)
            }),
            "library_create_relation" => {
                let draft: RelationDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid relation draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(RelationRepository::new(connection).create(draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_update_relation" => {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or("missing relation id")?;
                let draft: RelationDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid relation draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(RelationRepository::new(connection).update(id, draft)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_delete_relation" => self.state.with_database(|connection| {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
                RelationRepository::new(connection).soft_delete(id)?;
                Ok(Value::Null)
            }),
            "library_project_snapshot" => {
                let id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                self.state.with_database(|connection| {
                    serde_json::to_value(load_project_snapshot(connection, id)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "diagnostics_summary" => Ok(json!({
                "appVersion": env!("CARGO_PKG_VERSION"),
                "platform": std::env::consts::OS,
                "database": "sqlite",
            })),
            "vector_preview_chunks" => {
                let text = args
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or("missing text")?;
                let max_chars =
                    args.get("maxChars").and_then(Value::as_u64).unwrap_or(1200) as usize;
                serde_json::to_value(crate::vector::chunking::chunk_text(text, max_chars))
                    .map_err(|error| error.to_string())
            }
            "search_entities" => {
                let filter: crate::services::search::SearchFilter =
                    serde_json::from_value(args.get("filter").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid search filter: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::search::search_project(
                        connection, filter,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "logic_audit_facts" => {
                let facts: Vec<crate::logic::conflict::Fact> =
                    serde_json::from_value(args.get("facts").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid facts: {error}"))?;
                serde_json::to_value(crate::logic::conflict::detect_conflicts(&facts))
                    .map_err(|error| error.to_string())
            }
            "logic_repair_suggestions" => {
                let conflict: crate::logic::conflict::LogicConflict =
                    serde_json::from_value(args.get("conflict").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid conflict: {error}"))?;
                serde_json::to_value(crate::logic::repair::suggest_repairs(&conflict))
                    .map_err(|error| error.to_string())
            }
            "library_relation_type_presets" => {
                serde_json::to_value(crate::services::relation_workspace::relation_type_presets())
                    .map_err(|error| error.to_string())
            }
            "library_list_backlinks" => {
                let target: crate::domain::relation::EntityRef =
                    serde_json::from_value(args.get("target").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid relation target: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(
                        crate::domain::relation::RelationRepository::new(connection)
                            .list_backlinks(&target)?,
                    )
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_relation_neighborhood" => {
                let project_id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                let target: crate::domain::relation::EntityRef =
                    serde_json::from_value(args.get("target").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid relation target: {error}"))?;
                let depth = args.get("depth").and_then(Value::as_u64).unwrap_or(1) as usize;
                self.state.with_database(|connection| {
                    serde_json::to_value(
                        crate::services::relation_workspace::relation_neighborhood(
                            connection, project_id, target, depth,
                        )?,
                    )
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_create_event" | "library_update_event" => {
                let draft: crate::domain::event::EventDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid event draft: {error}"))?;
                let participants: Vec<crate::domain::event::EventParticipantDraft> =
                    serde_json::from_value(args.get("participants").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid event participants: {error}"))?;
                let id = args.get("id").and_then(Value::as_str).map(str::to_owned);
                self.state.with_database(|connection| {
                    let repository = crate::domain::event::EventRepository::new(connection);
                    let event = if command == "library_create_event" {
                        repository.create(draft, participants)?
                    } else {
                        let id = id.as_deref().ok_or(rusqlite::Error::QueryReturnedNoRows)?;
                        repository.update(id, draft, participants)?
                    };
                    serde_json::to_value(event)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "library_delete_event" => self.state.with_database(|connection| {
                let id = args
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
                crate::domain::event::EventRepository::new(connection).soft_delete(id)?;
                Ok(Value::Null)
            }),
            "job_create" => {
                let draft: crate::services::jobs::AiJobDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid job draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::jobs::create_queued_job(
                        connection, draft,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "job_list" => self.state.with_database(|connection| {
                serde_json::to_value(crate::services::jobs::list_jobs(connection)?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
            }),
            "job_current" => self.state.with_database(|connection| {
                serde_json::to_value(crate::services::jobs::current_running_job(connection)?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
            }),
            "job_logs" => {
                let id = args
                    .get("jobId")
                    .and_then(Value::as_str)
                    .ok_or("missing job id")?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::jobs::list_job_logs(connection, id)?)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "job_cancel" | "job_retry" => {
                let id = args
                    .get("jobId")
                    .and_then(Value::as_str)
                    .ok_or("missing job id")?;
                self.state.with_database(|connection| {
                    let job = if command == "job_cancel" {
                        crate::services::jobs::cancel_job(connection, id)?
                    } else {
                        crate::services::jobs::retry_job(connection, id)?
                    };
                    serde_json::to_value(job)
                        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "prompt_list_templates" => self.state.with_database(|connection| {
                serde_json::to_value(crate::services::prompts::list_prompt_templates(connection)?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
            }),
            "prompt_copy_template" => {
                let id = args
                    .get("templateId")
                    .and_then(Value::as_str)
                    .ok_or("missing template id")?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::prompts::copy_prompt_template(
                        connection, id,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "prompt_save_template" => {
                let draft: crate::services::prompts::PromptTemplateDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid prompt draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::prompts::save_prompt_template(
                        connection, draft,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "prompt_reset_builtin_templates" => self.state.with_database(|connection| {
                serde_json::to_value(crate::services::prompts::reset_builtin_prompt_templates(
                    connection,
                )?)
                .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
            }),
            "prompt_preview_template" => {
                let request: crate::services::prompts::PromptTemplatePreviewRequest =
                    serde_json::from_value(args.get("request").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid prompt preview request: {error}"))?;
                serde_json::to_value(crate::services::prompts::preview_prompt_template(request))
                    .map_err(|error| error.to_string())
            }
            "simulation_run" => {
                let project_id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                let scenario = args
                    .get("scenario")
                    .and_then(Value::as_str)
                    .ok_or("missing scenario")?;
                let referenced_entities: Vec<String> = serde_json::from_value(
                    args.get("referencedEntities").cloned().unwrap_or_default(),
                )
                .map_err(|error| format!("invalid referenced entities: {error}"))?;
                serde_json::to_value(crate::services::simulation::simulate_scenario(
                    project_id,
                    scenario,
                    referenced_entities,
                ))
                .map_err(|error| error.to_string())
            }
            "character_growth_preview_trait_delta" => {
                let mut state: crate::services::character_growth::CharacterTraitState =
                    serde_json::from_value(args.get("state").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid trait state: {error}"))?;
                let delta: crate::services::character_growth::TraitDelta =
                    serde_json::from_value(args.get("delta").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid trait delta: {error}"))?;
                crate::services::character_growth::apply_trait_delta(&mut state, delta);
                serde_json::to_value(state).map_err(|error| error.to_string())
            }
            "character_growth_workspace" => {
                let project_id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::character_growth::list_growth_workspace(
                        connection, project_id,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "character_growth_create_record" => {
                let draft: crate::domain::character_trait_delta::CharacterTraitDeltaRecordDraft =
                    serde_json::from_value(args.get("draft").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid trait record draft: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(
                        crate::services::character_growth::create_trait_delta_record(
                            connection, draft,
                        )?,
                    )
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "archive_export_project" => {
                let project_id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::import_export::export_project(
                        connection, project_id,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "archive_import_project" => {
                let archive: crate::services::import_export::ProjectArchive =
                    serde_json::from_value(args.get("archive").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid project archive: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::import_export::import_project(
                        connection, archive,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "rag_index_chunks" => {
                let project_id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                let max_chars =
                    args.get("maxChars").and_then(Value::as_u64).unwrap_or(1200) as usize;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::rag::index_project_chunks(
                        connection, project_id, max_chars,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "rag_index_status" => {
                let project_id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                let (settings, api_key) = self.load_openai_settings()?;
                let model =
                    ameya_core::ai::provider_resolver::resolve_optional_openai_embedding_model(
                        settings, api_key,
                    );
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::rag::embedding_index_status(
                        connection,
                        project_id,
                        model.as_deref(),
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "rag_index_embeddings" => {
                let project_id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                let max_chars =
                    args.get("maxChars").and_then(Value::as_u64).unwrap_or(1200) as usize;
                self.state.with_database(|connection| {
                    crate::services::rag::index_project_chunks(connection, project_id, max_chars)
                })?;
                let (settings, api_key) = self.load_openai_settings()?;
                let provider =
                    ameya_core::ai::provider_resolver::resolve_openai_embedding_provider(
                        settings,
                        api_key,
                        ameya_core::ai::provider_resolver::OPENAI_RAG_MESSAGES,
                    )?;
                let chunks = self.state.with_database(|connection| {
                    crate::services::rag::chunks_requiring_embedding(
                        connection,
                        project_id,
                        &provider.model,
                    )
                })?;
                if chunks.is_empty() {
                    let status = self.embedding_status(project_id, &provider.model)?;
                    return Ok(json!({
                        "chunkCount": status.chunk_count,
                        "embeddingCount": status.embedding_count,
                        "model": status.model,
                    }));
                }
                let client = ameya_core::ai::openai_compatible::OpenAiCompatibleClient::new(
                    provider.base_url.clone(),
                    provider.api_key.clone(),
                    ameya_core::ai::openai_compatible::UreqOpenAiTransport,
                );
                let result = client
                    .embeddings(
                        &provider.model,
                        chunks.iter().map(|chunk| chunk.text.clone()).collect(),
                    )
                    .map_err(|error| error.message)?;
                if result.vectors.len() != chunks.len() {
                    return Err("嵌入模型返回的向量数量与切片数量不一致".into());
                }
                self.state.with_database(|connection| {
                    for (chunk, vector) in chunks.iter().zip(result.vectors) {
                        crate::services::rag::upsert_embedding(
                            connection,
                            &chunk.id,
                            &provider.model,
                            vector,
                        )?;
                    }
                    Ok(())
                })?;
                let status = self.embedding_status(project_id, &provider.model)?;
                Ok(json!({
                    "chunkCount": status.chunk_count,
                    "embeddingCount": status.embedding_count,
                    "model": status.model,
                }))
            }
            "rag_preview_context_pack" => {
                let project_id = args
                    .get("projectId")
                    .and_then(Value::as_str)
                    .ok_or("missing project id")?;
                let query = args
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let query_vector: Vec<f32> =
                    serde_json::from_value(args.get("queryVector").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid query vector: {error}"))?;
                self.state.with_database(|connection| {
                    serde_json::to_value(crate::services::rag::build_context_pack(
                        connection,
                        project_id,
                        query,
                        query_vector,
                    )?)
                    .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
                })
            }
            "search_semantic" => {
                let request: crate::services::search::SemanticSearchRequest =
                    serde_json::from_value(args.get("request").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid semantic search request: {error}"))?;
                if request.query.trim().is_empty() {
                    return semantic_response(
                        crate::services::search::SemanticSearchStatus::Ready,
                        String::new(),
                        String::new(),
                        Vec::new(),
                    );
                }
                let (settings, api_key) = self.load_openai_settings()?;
                let provider =
                    match ameya_core::ai::provider_resolver::resolve_openai_embedding_provider(
                        settings,
                        api_key,
                        ameya_core::ai::provider_resolver::OPENAI_SEARCH_MESSAGES,
                    ) {
                        Ok(provider) => provider,
                        Err(message) => {
                            return semantic_response(
                                crate::services::search::SemanticSearchStatus::Degraded,
                                message,
                                String::new(),
                                Vec::new(),
                            );
                        }
                    };
                let index_status = self.state.with_database(|connection| {
                    crate::services::rag::embedding_index_status(
                        connection,
                        &request.project_id,
                        Some(&provider.model),
                    )
                })?;
                if index_status.status != "ready" {
                    return semantic_response(
                        crate::services::search::SemanticSearchStatus::Degraded,
                        index_status.message,
                        provider.model.clone(),
                        Vec::new(),
                    );
                }
                let client = ameya_core::ai::openai_compatible::OpenAiCompatibleClient::new(
                    provider.base_url.clone(),
                    provider.api_key.clone(),
                    ameya_core::ai::openai_compatible::UreqOpenAiTransport,
                );
                let query_vector =
                    match client.embeddings(&provider.model, vec![request.query.clone()]) {
                        Ok(result) => result.vectors.into_iter().next().unwrap_or_default(),
                        Err(error) => {
                            return semantic_response(
                                crate::services::search::SemanticSearchStatus::Degraded,
                                error.message,
                                provider.model.clone(),
                                Vec::new(),
                            );
                        }
                    };
                let candidates = self.state.with_database(|connection| {
                    crate::services::rag::load_vector_candidates_for_model(
                        connection,
                        &request.project_id,
                        &provider.model,
                    )
                })?;
                if candidates.is_empty() {
                    return semantic_response(
                        crate::services::search::SemanticSearchStatus::Degraded,
                        "当前项目还没有可用的 embedding 索引。".into(),
                        provider.model.clone(),
                        Vec::new(),
                    );
                }
                let matches = crate::services::rag::rank_vector_candidates(
                    candidates,
                    query_vector,
                    request.limit,
                );
                let items = self.state.with_database(|connection| {
                    crate::services::search::semantic_search_results(
                        connection,
                        matches,
                        request.limit,
                    )
                })?;
                semantic_response(
                    crate::services::search::SemanticSearchStatus::Ready,
                    String::new(),
                    provider.model,
                    items,
                )
            }
            // Tauri 契约：默认提供方列表只列 CLI 项；OpenAI 兼容项由
            // load 侧的首选默认（含禁用占位）单独提供。
            "ai_default_providers" => serde_json::to_value([
                ameya_core::ai::settings::AiProviderConfig::codex_cli(),
                ameya_core::ai::settings::AiProviderConfig::claude_cli(),
            ])
            .map_err(|error| error.to_string()),
            "ai_agent_ask" => {
                let request: ameya_core::ai::agent_ask::AgentAskRequest =
                    serde_json::from_value(args.get("request").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid agent ask request: {error}"))?;
                ameya_core::ai::agent_ask::validate_question(&request.question)?;
                let (settings, api_key) = self.load_openai_settings()?;
                let provider = ameya_core::ai::provider_resolver::resolve_openai_chat_provider(
                    settings,
                    api_key,
                    ameya_core::ai::provider_resolver::OPENAI_AGENT_MESSAGES,
                )?;
                let client = ameya_core::ai::openai_compatible::OpenAiCompatibleClient::new(
                    provider.base_url.clone(),
                    provider.api_key.clone(),
                    ameya_core::ai::openai_compatible::UreqOpenAiTransport,
                );
                serde_json::to_value(ameya_core::ai::agent_ask::run_agent_ask(
                    request, provider, &client,
                )?)
                .map_err(|error| error.to_string())
            }
            "ai_test_openai_provider" => {
                let (settings, api_key) = self.load_openai_settings()?;
                let provider = match ameya_core::ai::provider_resolver::resolve_openai_chat_provider(
                    settings,
                    api_key,
                    ameya_core::ai::provider_resolver::OPENAI_TEST_MESSAGES,
                ) {
                    Ok(provider) => provider,
                    Err(message) => {
                        return serde_json::to_value(openai_error_result(
                            ameya_core::ai::openai_compatible::ProviderError::config_missing(
                                message,
                            ),
                        ))
                        .map_err(|error| error.to_string())
                    }
                };
                let client = ameya_core::ai::openai_compatible::OpenAiCompatibleClient::new(
                    provider.base_url.clone(),
                    provider.api_key.clone(),
                    ameya_core::ai::openai_compatible::UreqOpenAiTransport,
                );
                let result = match client.chat(
                    &provider.model,
                    vec![ameya_core::ai::openai_compatible::ChatMessage {
                        role: "user".into(),
                        content: "Return the word ok.".into(),
                    }],
                    0.0,
                ) {
                    Ok(content) => OpenAiProviderTestResult {
                        ok: true,
                        message: if content.trim().is_empty() {
                            "Provider 已响应，但返回内容为空".into()
                        } else {
                            "Provider 测试调用成功".into()
                        },
                        error: None,
                    },
                    Err(error) => openai_error_result(error),
                };
                serde_json::to_value(result).map_err(|error| error.to_string())
            }
            "ai_test_codex_cli_provider" => {
                // The test spawns a real subprocess with a 60s timeout; require an
                // explicit (possibly empty) object so headless dispatch checks do
                // not trigger it accidentally.
                if !args.is_object() {
                    return Err(format!(
                        "{command} requires an explicit JSON object argument"
                    ));
                }
                let command_template = self.cli_command_template(
                    ameya_core::ai::settings::AiProviderKind::CodexCli,
                    "Codex CLI",
                )?;
                std::fs::create_dir_all(&self.data_dir)
                    .map_err(|error| format!("创建 Codex CLI 工作目录失败：{error}"))?;
                let config = ameya_core::ai::codex_cli::CodexCliConfig {
                    command_template,
                    workspace: self.data_dir.to_string_lossy().to_string(),
                    max_turns: 1,
                    output_format: ameya_core::ai::cli_provider::DEFAULT_CLI_OUTPUT_FORMAT
                        .to_string(),
                    timeout: std::time::Duration::from_secs(60),
                };
                let runner = ameya_core::ai::process_runner::StdProcessRunner;
                let provider = ameya_core::ai::codex_cli::CodexCliProvider::new(config, &runner);
                let result = match provider.test_prompt() {
                    Ok(output) => cli_test_ok("Codex CLI 测试调用成功", output),
                    Err(error) => cli_test_err(error),
                };
                serde_json::to_value(result).map_err(|error| error.to_string())
            }
            "ai_test_claude_cli_provider" => {
                // See the Codex arm: explicit arguments guard real subprocess runs.
                if !args.is_object() {
                    return Err(format!(
                        "{command} requires an explicit JSON object argument"
                    ));
                }
                let command_template = self.cli_command_template(
                    ameya_core::ai::settings::AiProviderKind::ClaudeCli,
                    "Claude CLI",
                )?;
                std::fs::create_dir_all(&self.data_dir)
                    .map_err(|error| format!("创建 Claude CLI 工作目录失败：{error}"))?;
                let config = ameya_core::ai::claude_cli::ClaudeCliConfig {
                    command_template,
                    workspace: self.data_dir.to_string_lossy().to_string(),
                    max_turns: 1,
                    output_format: ameya_core::ai::claude_cli::CLAUDE_OUTPUT_FORMAT.to_string(),
                    timeout: std::time::Duration::from_secs(60),
                };
                let runner = ameya_core::ai::process_runner::StdProcessRunner;
                let provider = ameya_core::ai::claude_cli::ClaudeCliProvider::new(config, &runner);
                let result = match provider.test_prompt() {
                    Ok(output) => cli_test_ok("Claude CLI 测试调用成功", output),
                    Err(error) => cli_test_err(error),
                };
                serde_json::to_value(result).map_err(|error| error.to_string())
            }
            "ai_load_provider_settings" => {
                serde_json::to_value(ameya_core::ai::settings::load_provider_settings(
                    self.config_dir_path(),
                    &self.secrets,
                )?)
                .map_err(|error| error.to_string())
            }
            "ai_save_provider_settings" => {
                let drafts: Vec<ameya_core::ai::settings::AiProviderSettingsDraft> =
                    serde_json::from_value(args.get("drafts").cloned().unwrap_or_default())
                        .map_err(|error| format!("invalid provider settings drafts: {error}"))?;
                serde_json::to_value(ameya_core::ai::settings::save_provider_settings(
                    self.config_dir_path(),
                    drafts,
                    &self.secrets,
                )?)
                .map_err(|error| error.to_string())
            }
            _ => Err(format!("Unknown Nana command: {command}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Route;

    fn test_host() -> (NativeCommandHost, PathBuf) {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("ameya-native-{}", uuid::Uuid::new_v4()));
        (
            NativeCommandHost::initialize(dir.clone()).expect("database initializes"),
            dir,
        )
    }

    #[test]
    fn health_uses_frontend_camel_case_contract() {
        let (host, dir) = test_host();
        let health = host
            .invoke("health_check", Value::Null)
            .expect("health works");
        assert_eq!(health["appVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(health["appDataDir"], dir.to_string_lossy().to_string());
    }

    #[test]
    fn dispatch_to_host_executes_route_commands() {
        let (host, _) = test_host();
        let mut app = crate::NativeApp::default();
        let result = app
            .dispatch_to_host(&host, "project_list", Value::Null)
            .expect("route command reaches host");
        assert!(result.is_array());
    }

    #[test]
    fn projects_create_list_and_snapshot_follow_the_native_boundary() {
        let (host, _) = test_host();
        let mut app = crate::NativeApp {
            route: Route::Projects,
            ..crate::NativeApp::default()
        };
        let created = app
            .dispatch_to_host(
                &host,
                "project_create",
                app.action_payload("project_create")
                    .expect("create payload"),
            )
            .expect("project create reaches sqlite");
        let project_id = created["id"]
            .as_str()
            .expect("created project id")
            .to_string();
        app.selected_project_id = Some(project_id.clone());
        let projects = host
            .invoke("project_list", Value::Null)
            .expect("list works");
        assert!(projects
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"] == project_id));
        let snapshot = app
            .dispatch_to_host(
                &host,
                "library_project_snapshot",
                app.action_payload("library_project_snapshot")
                    .expect("snapshot payload"),
            )
            .expect("snapshot reaches sqlite");
        assert_eq!(snapshot["projectId"], project_id);
        let missing_context =
            crate::NativeApp::default().action_payload("library_project_snapshot");
        assert_eq!(missing_context, Err("请先选择一个项目".to_string()));
    }

    #[test]
    fn project_commands_use_existing_sqlite_repository() {
        let (host, _) = test_host();
        let project = host
            .invoke(
                "project_create",
                json!({ "draft": { "name": "演示", "description": "Nana" } }),
            )
            .expect("project create works");
        assert_eq!(project["name"], "演示");
        let projects = host
            .invoke("project_list", Value::Null)
            .expect("project list works");
        assert_eq!(projects.as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn every_route_action_reaches_native_host_boundary() {
        let (host, _) = test_host();
        for route in Route::ALL {
            for (_, _, command) in route.actions() {
                let result = host.invoke(command, Value::Null);
                assert!(
                    !matches!(result, Err(ref error) if error == &format!("Unknown Nana command: {command}")),
                    "route action is not wired to NativeCommandHost: {route:?} -> {command}"
                );
            }
        }
    }

    #[test]
    fn route_specific_workspace_commands_have_native_branches() {
        let (host, _) = test_host();
        for command in [
            "archive_export_project",
            "archive_import_project",
            "diagnostics_summary",
            "job_current",
            "logic_audit_facts",
            "rag_index_embeddings",
            "search_entities",
            "ai_agent_ask",
            "ai_test_openai_provider",
        ] {
            let result = host.invoke(command, Value::Null);
            assert!(
                !matches!(result, Err(ref error) if error == &format!("Unknown Nana command: {command}")),
                "missing route command: {command}"
            );
        }
    }

    #[test]
    fn every_frontend_command_has_a_native_dispatch_branch() {
        let (host, _) = test_host();
        let commands = [
            "ai_agent_ask",
            "ai_default_providers",
            "ai_load_provider_settings",
            "ai_save_provider_settings",
            "ai_test_openai_provider",
            "ai_test_codex_cli_provider",
            "ai_test_claude_cli_provider",
            "archive_export_project",
            "archive_import_project",
            "diagnostics_summary",
            "health_check",
            "job_create",
            "job_list",
            "job_current",
            "job_logs",
            "job_cancel",
            "job_retry",
            "library_project_snapshot",
            "library_create_entry",
            "library_update_entry",
            "library_delete_entry",
            "library_create_character",
            "library_update_character",
            "library_delete_character",
            "library_create_event",
            "library_update_event",
            "library_delete_event",
            "library_create_axiom",
            "library_update_axiom",
            "library_delete_axiom",
            "library_list_backlinks",
            "library_relation_type_presets",
            "library_relation_neighborhood",
            "library_create_relation",
            "library_update_relation",
            "library_delete_relation",
            "logic_audit_facts",
            "logic_repair_suggestions",
            "project_list",
            "project_create",
            "project_update",
            "project_archive",
            "prompt_list_templates",
            "prompt_copy_template",
            "prompt_save_template",
            "prompt_reset_builtin_templates",
            "prompt_preview_template",
            "rag_index_chunks",
            "rag_index_status",
            "rag_index_embeddings",
            "rag_preview_context_pack",
            "search_entities",
            "search_semantic",
            "simulation_run",
            "vector_preview_chunks",
            "character_growth_preview_trait_delta",
            "character_growth_workspace",
            "character_growth_create_record",
        ];
        for command in commands {
            let result = host.invoke(command, Value::Null);
            assert!(
                !matches!(
                    result,
                    Err(ref error) if error == &format!("Unknown Nana command: {command}")
                ),
                "missing native dispatch branch for {command}"
            );
        }
    }

    fn isolated_host() -> (NativeCommandHost, tempfile::TempDir, tempfile::TempDir) {
        let data_dir = tempfile::tempdir().expect("data tempdir");
        let config_dir = tempfile::tempdir().expect("config tempdir");
        let mut host = NativeCommandHost::initialize_with_config(
            data_dir.path().to_path_buf(),
            Some(config_dir.path().to_path_buf()),
        )
        .expect("host initializes");
        host.secrets = secrets::SharedSecretStore::new(secrets::InMemorySecretStore::new());
        (host, data_dir, config_dir)
    }

    #[test]
    fn provider_settings_roundtrip_keeps_secrets_out_of_json() {
        let (host, _data_dir, config_dir) = isolated_host();

        let saved = host
            .invoke(
                "ai_save_provider_settings",
                json!({
                    "drafts": [{
                        "kind": "openAiCompatible",
                        "baseUrl": "https://llm.example/v1",
                        "apiKey": "sk-live-abcd1234",
                        "clearApiKey": false,
                        "chatModel": "story-chat",
                        "embeddingModel": "story-embed",
                        "commandTemplate": null,
                        "enabled": true
                    }]
                }),
            )
            .expect("settings save");
        let openai = saved
            .as_array()
            .expect("settings are a list")
            .iter()
            .find(|item| item["kind"] == "openAiCompatible")
            .expect("openai provider present");
        assert_eq!(openai["hasApiKey"], true);
        assert_eq!(openai["apiKeyPreview"], "sk-********1234");
        assert!(!saved.to_string().contains("sk-live-abcd1234"));

        let raw_json = std::fs::read_to_string(config_dir.path().join("ai-provider-settings.json"))
            .expect("settings file written");
        assert!(raw_json.contains("hasApiKey"));
        assert!(!raw_json.contains("sk-live-abcd1234"));

        let loaded = host
            .invoke("ai_load_provider_settings", Value::Null)
            .expect("settings load");
        let openai = loaded
            .as_array()
            .expect("settings are a list")
            .iter()
            .find(|item| item["kind"] == "openAiCompatible")
            .expect("openai provider present");
        assert_eq!(openai["hasApiKey"], true);
        assert_eq!(openai["apiKeyPreview"], "sk-********1234");
    }

    #[test]
    fn agent_ask_rejects_empty_question_before_provider_resolution() {
        let (host, _data_dir, _config_dir) = isolated_host();

        let error = host
            .invoke(
                "ai_agent_ask",
                json!({
                    "request": {
                        "projectId": "project_1",
                        "question": "   ",
                        "references": []
                    }
                }),
            )
            .expect_err("empty question fails");
        assert_eq!(error, "请输入问题");
    }

    #[test]
    fn diagnostics_summary_matches_frontend_contract() {
        let (host, _) = test_host();
        let summary = host
            .invoke("diagnostics_summary", Value::Null)
            .expect("diagnostics work");
        assert_eq!(summary["appVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(summary["platform"], std::env::consts::OS);
        assert_eq!(summary["database"], "sqlite");
    }

    #[test]
    fn semantic_search_degrades_without_configured_provider() {
        let (host, _data_dir, _config_dir) = isolated_host();

        let degraded = host
            .invoke(
                "search_semantic",
                json!({
                    "request": { "projectId": "project_1", "query": "月光", "limit": 5 }
                }),
            )
            .expect("semantic search never throws");
        assert_eq!(degraded["status"], "degraded");
        assert_eq!(degraded["message"], "请先启用 OpenAI 兼容接口用于语义搜索");
        assert_eq!(degraded["model"], "");
        assert_eq!(degraded["items"].as_array().map(Vec::len), Some(0));

        let empty = host
            .invoke(
                "search_semantic",
                json!({
                    "request": { "projectId": "project_1", "query": "   ", "limit": 5 }
                }),
            )
            .expect("empty query returns ready");
        assert_eq!(empty["status"], "ready");
        assert_eq!(empty["items"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn rag_index_embeddings_reports_missing_provider_before_network() {
        let (host, _data_dir, _config_dir) = isolated_host();

        let project = host
            .invoke(
                "project_create",
                json!({ "draft": { "name": "演示", "description": "" } }),
            )
            .expect("project create");
        let project_id = project["id"].as_str().expect("project id").to_string();
        host.invoke(
            "library_create_entry",
            json!({
                "draft": {
                    "projectId": project_id,
                    "entryType": "item",
                    "title": "月光阔剑",
                    "summary": "",
                    "body": "潮汐能武器，首次用于围城战。",
                    "tags": [],
                    "status": "draft"
                }
            }),
        )
        .expect("entry create");

        let error = host
            .invoke(
                "rag_index_embeddings",
                json!({ "projectId": project_id, "maxChars": 600 }),
            )
            .expect_err("unconfigured provider fails fast");
        assert_eq!(error, "请先启用 OpenAI 兼容接口用于向量索引");
    }

    #[test]
    fn rag_index_status_reports_missing_model_without_provider() {
        let (host, _data_dir, _config_dir) = isolated_host();

        let project = host
            .invoke(
                "project_create",
                json!({ "draft": { "name": "演示", "description": "" } }),
            )
            .expect("project create");
        let project_id = project["id"].as_str().expect("project id").to_string();

        let status = host
            .invoke("rag_index_status", json!({ "projectId": project_id }))
            .expect("status works");
        assert_eq!(status["status"], "degraded");
        assert_eq!(
            status["message"],
            "请先启用 OpenAI 兼容接口并填写嵌入模型。"
        );
        assert_eq!(status["model"], "");
    }

    #[test]
    fn default_providers_match_tauri_two_item_contract() {
        let dir = tempfile::tempdir().expect("temp dir");
        let host = super::NativeCommandHost::initialize(dir.path().to_path_buf())
            .expect("host initializes");
        let providers = host
            .invoke("ai_default_providers", json!({}))
            .expect("default providers");
        let items = providers.as_array().expect("provider list");
        assert_eq!(items.len(), 2, "Tauri 契约：默认提供方仅 CLI 两项");
        assert_eq!(items[0]["kind"], "codexCli");
        assert_eq!(items[1]["kind"], "claudeCli");
    }
}
