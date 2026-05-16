use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::domain::shared::{new_id, now};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub template: String,
    pub built_in: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub fn ensure_builtin_prompts(connection: &Connection) -> rusqlite::Result<()> {
    let count: i64 = connection.query_row(
        "SELECT COUNT(*) FROM prompt_templates WHERE built_in = 1",
        [],
        |row| row.get(0),
    )?;
    if count > 0 {
        return Ok(());
    }
    for (name, purpose, template) in [
        (
            "逻辑审计",
            "logic_audit",
            "基于以下上下文审计设定逻辑：\n{{project_context}}\n问题：{{question}}",
        ),
        (
            "角色行为校验",
            "behavior_audit",
            "判断角色行为是否符合画像和世界观：\n{{project_context}}\n行为：{{question}}",
        ),
        (
            "叙事模拟",
            "simulation",
            "基于世界观和角色画像推演情景：\n{{project_context}}\n情景：{{question}}",
        ),
    ] {
        let timestamp = now();
        connection.execute(
            "INSERT INTO prompt_templates
             (id, name, purpose, template, built_in, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5)",
            params![new_id("prompt"), name, purpose, template, timestamp],
        )?;
    }
    Ok(())
}

pub fn list_prompt_templates(connection: &Connection) -> rusqlite::Result<Vec<PromptTemplate>> {
    ensure_builtin_prompts(connection)?;
    let mut statement = connection.prepare(
        "SELECT id, name, purpose, template, built_in, created_at, updated_at
         FROM prompt_templates
         ORDER BY built_in DESC, name ASC",
    )?;
    let prompts = statement
        .query_map([], |row| {
            Ok(PromptTemplate {
                id: row.get(0)?,
                name: row.get(1)?,
                purpose: row.get(2)?,
                template: row.get(3)?,
                built_in: row.get::<_, i64>(4)? != 0,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?
        .collect();
    prompts
}
