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
    pub variables: Vec<PromptTemplateVariable>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplateVariable {
    pub name: String,
    pub description: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplateDraft {
    pub id: Option<String>,
    pub name: String,
    pub purpose: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplateVariableValue {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplatePreviewRequest {
    pub template: String,
    pub values: Vec<PromptTemplateVariableValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplatePreview {
    pub prompt: String,
    pub missing_variables: Vec<String>,
}

struct BuiltinPromptTemplate {
    name: &'static str,
    purpose: &'static str,
    template: &'static str,
}

pub fn ensure_builtin_prompts(connection: &Connection) -> rusqlite::Result<()> {
    for builtin in builtin_prompt_templates() {
        let count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM prompt_templates WHERE built_in = 1 AND purpose = ?1",
            params![builtin.purpose],
            |row| row.get(0),
        )?;
        if count == 0 {
            insert_builtin_prompt_template(connection, builtin)?;
        }
    }
    Ok(())
}

pub fn reset_builtin_prompt_templates(
    connection: &Connection,
) -> rusqlite::Result<Vec<PromptTemplate>> {
    connection.execute("DELETE FROM prompt_templates WHERE built_in = 1", [])?;
    for builtin in builtin_prompt_templates() {
        insert_builtin_prompt_template(connection, builtin)?;
    }
    list_builtin_prompt_templates(connection)
}

pub fn copy_prompt_template(
    connection: &Connection,
    template_id: &str,
) -> rusqlite::Result<PromptTemplate> {
    let source = require_prompt_template(connection, template_id)?;
    let timestamp = now();
    let id = new_id("prompt");
    connection.execute(
        "INSERT INTO prompt_templates
         (id, name, purpose, template, built_in, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5)",
        params![
            id,
            format!("{} 副本", source.name),
            source.purpose,
            source.template,
            timestamp
        ],
    )?;
    require_prompt_template(connection, &id)
}

pub fn save_prompt_template(
    connection: &Connection,
    draft: PromptTemplateDraft,
) -> rusqlite::Result<PromptTemplate> {
    let timestamp = now();
    let name = normalized_or_default(&draft.name, "未命名模板");
    let purpose = normalized_or_default(&draft.purpose, "custom");

    if let Some(id) = draft.id {
        let affected = connection.execute(
            "UPDATE prompt_templates
             SET name = ?2, purpose = ?3, template = ?4, updated_at = ?5
             WHERE id = ?1 AND built_in = 0",
            params![id, name, purpose, draft.template, timestamp],
        )?;
        ensure_template_was_updated(affected)?;
        require_prompt_template(connection, &id)
    } else {
        let id = new_id("prompt");
        connection.execute(
            "INSERT INTO prompt_templates
             (id, name, purpose, template, built_in, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 0, ?5, ?5)",
            params![id, name, purpose, draft.template, timestamp],
        )?;
        require_prompt_template(connection, &id)
    }
}

pub fn preview_prompt_template(request: PromptTemplatePreviewRequest) -> PromptTemplatePreview {
    let mut prompt = String::with_capacity(request.template.len());
    let mut missing_variables = Vec::new();
    let mut rest = request.template.as_str();

    while let Some(start) = rest.find("{{") {
        prompt.push_str(&rest[..start]);
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let raw_name = &after_open[..end];
            let name = raw_name.trim();
            if let Some(value) = request
                .values
                .iter()
                .find(|item| item.name == name)
                .map(|item| item.value.as_str())
            {
                prompt.push_str(value);
            } else {
                prompt.push_str("{{");
                prompt.push_str(raw_name);
                prompt.push_str("}}");
                push_unique(&mut missing_variables, name);
            }
            rest = &after_open[end + 2..];
        } else {
            prompt.push_str(&rest[start..]);
            rest = "";
        }
    }

    prompt.push_str(rest);
    PromptTemplatePreview {
        prompt,
        missing_variables,
    }
}

pub fn list_prompt_templates(connection: &Connection) -> rusqlite::Result<Vec<PromptTemplate>> {
    ensure_builtin_prompts(connection)?;
    let mut statement = connection.prepare(
        "SELECT id, name, purpose, template, built_in, created_at, updated_at
         FROM prompt_templates
         ORDER BY built_in DESC, name ASC",
    )?;
    let templates = statement.query_map([], map_prompt_template)?.collect();
    templates
}

fn list_builtin_prompt_templates(connection: &Connection) -> rusqlite::Result<Vec<PromptTemplate>> {
    let mut statement = connection.prepare(
        "SELECT id, name, purpose, template, built_in, created_at, updated_at
         FROM prompt_templates
         WHERE built_in = 1
         ORDER BY name ASC",
    )?;
    let templates = statement.query_map([], map_prompt_template)?.collect();
    templates
}

fn insert_builtin_prompt_template(
    connection: &Connection,
    builtin: &BuiltinPromptTemplate,
) -> rusqlite::Result<()> {
    let timestamp = now();
    connection.execute(
        "INSERT INTO prompt_templates
         (id, name, purpose, template, built_in, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5, ?5)",
        params![
            new_id("prompt"),
            builtin.name,
            builtin.purpose,
            builtin.template,
            timestamp
        ],
    )?;
    Ok(())
}

fn require_prompt_template(connection: &Connection, id: &str) -> rusqlite::Result<PromptTemplate> {
    get_prompt_template(connection, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

fn get_prompt_template(
    connection: &Connection,
    id: &str,
) -> rusqlite::Result<Option<PromptTemplate>> {
    let mut statement = connection.prepare(
        "SELECT id, name, purpose, template, built_in, created_at, updated_at
         FROM prompt_templates
         WHERE id = ?1",
    )?;
    let mut rows = statement.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_prompt_template(row)?))
    } else {
        Ok(None)
    }
}

fn map_prompt_template(row: &rusqlite::Row<'_>) -> rusqlite::Result<PromptTemplate> {
    let template: String = row.get(3)?;
    Ok(PromptTemplate {
        id: row.get(0)?,
        name: row.get(1)?,
        purpose: row.get(2)?,
        variables: describe_template_variables(&template),
        template,
        built_in: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn builtin_prompt_templates() -> &'static [BuiltinPromptTemplate] {
    &[
        BuiltinPromptTemplate {
            name: "逻辑审计",
            purpose: "logic_audit",
            template: "请基于项目上下文审计设定逻辑。\n\n项目上下文：\n{{project_context}}\n\n目标对象：{{target_entity}}\n\n审计问题：{{question}}\n\n请输出：冲突点、依据、影响范围、保守修复建议。",
        },
        BuiltinPromptTemplate {
            name: "补完问题",
            purpose: "completion_questions",
            template: "请根据项目上下文，为目标对象生成补完问题。\n\n项目上下文：\n{{project_context}}\n\n目标对象：{{target_entity}}\n\n关注问题：{{question}}\n\n请按世界规则、动机、资源约束、时间线四类输出问题。",
        },
        BuiltinPromptTemplate {
            name: "角色分析",
            purpose: "character_analysis",
            template: "请分析角色在当前世界观中的行为一致性。\n\n项目上下文：\n{{project_context}}\n\n角色或目标：{{target_entity}}\n\n分析问题：{{question}}\n\n请输出：核心欲望、约束、可能行动、风险。",
        },
        BuiltinPromptTemplate {
            name: "角色行为校验",
            purpose: "behavior_audit",
            template: "判断角色行为是否符合画像和世界观。\n\n项目上下文：\n{{project_context}}\n\n目标角色：{{target_entity}}\n\n行为或问题：{{question}}\n\n请输出一致、存疑、冲突三类判断。",
        },
        BuiltinPromptTemplate {
            name: "叙事模拟",
            purpose: "simulation",
            template: "基于世界观和角色画像推演情景。\n\n项目上下文：\n{{project_context}}\n\n模拟情景：{{scenario}}\n\n约束条件：{{constraints}}\n\n请按阶段、触发条件、参与者反应、后果输出。",
        },
    ]
}

fn describe_template_variables(template: &str) -> Vec<PromptTemplateVariable> {
    extract_template_variable_names(template)
        .into_iter()
        .map(|name| describe_variable(&name))
        .collect()
}

fn extract_template_variable_names(template: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let name = after_open[..end].trim();
            if is_variable_name(name) {
                push_unique(&mut names, name);
            }
            rest = &after_open[end + 2..];
        } else {
            break;
        }
    }
    names
}

fn describe_variable(name: &str) -> PromptTemplateVariable {
    let (description, example) = match name {
        "project_context" => (
            "项目或选中实体周边的规则、设定、事件和关系摘要。",
            "北境粮食依赖海运，冬季港口封冻三个月。",
        ),
        "target_entity" => ("本次分析聚焦的词条、角色、事件、地点或阵营。", "银港商会"),
        "question" => (
            "用户提出的具体问题、审计重点或补完方向。",
            "这条贸易规则是否会破坏北境粮价设定？",
        ),
        "scenario" => ("需要模拟的情景或变化。", "北境粮仓在冬季前被焚毁。"),
        "constraints" => (
            "模拟或分析时必须遵守的边界条件。",
            "不得引入新阵营；时间跨度不超过三个月。",
        ),
        _ => ("自定义变量，执行前填写对应文本。", ""),
    };
    PromptTemplateVariable {
        name: name.to_string(),
        description: description.to_string(),
        example: example.to_string(),
    }
}

fn push_unique(items: &mut Vec<String>, name: &str) {
    if !name.is_empty() && !items.iter().any(|item| item == name) {
        items.push(name.to_string());
    }
}

fn is_variable_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn normalized_or_default(value: &str, fallback: &str) -> String {
    let normalized = value.trim();
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized.to_string()
    }
}

fn ensure_template_was_updated(affected: usize) -> rusqlite::Result<()> {
    if affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}
