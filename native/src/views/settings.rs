//! 设置：外观 / 人工智能设置 / 命令行提供方 / 关于。
//! Tauri 版是带侧栏的设置页；NanaUI 骨架用同一 primary 区内的分区卡片承载。

use nana_ui::runtime::view::{
    column, dynamic, row, text, text_input, widget, AnyView, IntoView, Store,
};
use nana_ui::runtime::{Activate, Card, TextChanged};

use crate::state::{
    AiSettingsDraft, AiSettingsField, AiSettingsFieldKind, AiSettingsStateStoreFields, AppMessage,
    AppState, AppStateStoreFields, DiagnosticsStateStoreFields, SettingsTab, ThemeChoice,
};

pub fn page(app: Store<AppState>) -> impl IntoView {
    column().children((
        text("设置"),
        tab_bar(app.clone()),
        dynamic(app.settings_tab(), move |tab| match tab {
            SettingsTab::Appearance => appearance_section(app.clone()).into_any(),
            SettingsTab::Ai => provider_section(app.clone()).into_any(),
            SettingsTab::CliProviders => cli_section(app.clone()).into_any(),
            SettingsTab::About => about_section(app.clone()).into_any(),
        }),
    ))
}

fn action_button(
    label: &'static str,
    message: AppMessage,
) -> nana_ui::runtime::view::El<nana_ui::runtime::Button> {
    nana_ui::runtime::view::button(label)
        .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(message.clone()))
}

fn field_input(
    kind: &'static str,
    field: AiSettingsFieldKind,
    placeholder: &'static str,
    value: &str,
) -> nana_ui::runtime::view::El<nana_ui::runtime::TextInput> {
    text_input()
        .placeholder(placeholder)
        .value(value.to_string())
        .on_cx(move |_, event: &TextChanged, cx| {
            cx.dispatch_program_all(AppMessage::AiSettingsField(AiSettingsField {
                kind: kind.to_string(),
                field,
                value: event.value.to_string(),
            }))
        })
}

fn tab_button(
    label: &'static str,
    tab: SettingsTab,
    current: SettingsTab,
) -> nana_ui::runtime::view::El<nana_ui::runtime::Button> {
    // Button 没有 selected 态；用「●」前缀标示当前页签（Tauri 版的等价物）。
    let selected = tab == current;
    let label = if selected {
        format!("● {label}")
    } else {
        label.to_string()
    };
    nana_ui::runtime::view::button(label)
        .on_cx(move |_, _: &Activate, cx| cx.dispatch_program_all(AppMessage::SetSettingsTab(tab)))
}

fn tab_bar(app: Store<AppState>) -> impl IntoView {
    let current = app.settings_tab();
    dynamic(current, move |current| {
        row().children((
            tab_button("外观", SettingsTab::Appearance, *current),
            tab_button("人工智能设置", SettingsTab::Ai, *current),
            tab_button("命令行提供方", SettingsTab::CliProviders, *current),
            tab_button("关于", SettingsTab::About, *current),
        ))
    })
}

/// 外观：主题切换（暗色/浅色）。
fn appearance_section(app: Store<AppState>) -> impl IntoView {
    let theme = app.theme();
    let body = dynamic(theme, move |theme| {
        let (label, target) = match theme {
            ThemeChoice::Dark => ("当前主题：暗色（点击切换到浅色）", ThemeChoice::Light),
            ThemeChoice::Light => ("当前主题：浅色（点击切换到暗色）", ThemeChoice::Dark),
        };
        row().children((
            text(label),
            action_button("切换主题", AppMessage::SetTheme(target)),
        ))
    });
    widget(Card::new().title("外观")).children(body)
}

/// 人工智能设置（OpenAI 兼容）：可编辑字段 + 保存/测试。
fn provider_section(app: Store<AppState>) -> impl IntoView {
    let drafts = app.ai_settings().drafts();
    let body = dynamic(drafts, move |drafts| {
        let cards = drafts
            .iter()
            .filter(|draft| draft.kind == "openAiCompatible")
            .map(openai_editor)
            .collect::<Vec<_>>();
        if cards.is_empty() {
            column().children(text("配置尚未载入。")).into_any()
        } else {
            column().children(cards).into_any()
        }
    });
    column().children((
        body,
        row().children((
            action_button("保存设置", AppMessage::AiSettingsSave),
            action_button(
                "测试连接",
                AppMessage::AiSettingsTest("openAiCompatible".to_string()),
            ),
        )),
        settings_feedback(app),
    ))
}

fn openai_editor(draft: &AiSettingsDraft) -> AnyView {
    let kind = "openAiCompatible";
    column()
        .children((
            text(format!(
                "API 密钥掩码：{}（输入新值即更新，留空保持不变）",
                if draft.api_key.is_empty() {
                    "见已保存配置"
                } else {
                    "待保存的新密钥"
                }
            )),
            field_input(
                kind,
                AiSettingsFieldKind::BaseUrl,
                "Base URL，例如 https://api.example.com/v1",
                &draft.base_url,
            ),
            field_input(
                kind,
                AiSettingsFieldKind::ApiKey,
                "API 密钥（留空保持不变）",
                &draft.api_key,
            ),
            field_input(
                kind,
                AiSettingsFieldKind::ChatModel,
                "对话模型",
                &draft.chat_model,
            ),
            field_input(
                kind,
                AiSettingsFieldKind::EmbeddingModel,
                "嵌入模型",
                &draft.embedding_model,
            ),
            row().children((
                text(if draft.enabled {
                    "已启用"
                } else {
                    "已停用"
                }),
                action_button(
                    if draft.enabled { "停用" } else { "启用" },
                    AppMessage::AiSettingsToggle(kind.to_string()),
                ),
            )),
        ))
        .into_any()
}

/// 命令行提供方：codex/claude 命令模板编辑 + 测试。
fn cli_section(app: Store<AppState>) -> impl IntoView {
    let drafts = app.ai_settings().drafts();
    let body = dynamic(drafts, move |drafts| {
        let cards = drafts
            .iter()
            .filter(|draft| draft.kind != "openAiCompatible")
            .map(cli_editor)
            .collect::<Vec<_>>();
        if cards.is_empty() {
            column().children(text("配置尚未载入。")).into_any()
        } else {
            column().children(cards).into_any()
        }
    });
    column().children((
        body,
        row().children((
            action_button("保存设置", AppMessage::AiSettingsSave),
            action_button(
                "测试 Codex CLI",
                AppMessage::AiSettingsTest("codexCli".to_string()),
            ),
            action_button(
                "测试 Claude CLI",
                AppMessage::AiSettingsTest("claudeCli".to_string()),
            ),
        )),
        settings_feedback(app),
    ))
}

fn cli_editor(draft: &AiSettingsDraft) -> AnyView {
    let kind: &'static str = if draft.kind == "codexCli" {
        "codexCli"
    } else {
        "claudeCli"
    };
    column()
        .children((
            text(format!(
                "{}（{}）",
                draft.label,
                if draft.enabled {
                    "已启用"
                } else {
                    "已停用"
                }
            )),
            field_input(
                kind,
                AiSettingsFieldKind::CommandTemplate,
                "命令模板，{prompt}/{workspace}/{max_turns} 占位",
                &draft.command_template,
            ),
            row().children((
                text(if draft.enabled {
                    "已启用"
                } else {
                    "已停用"
                }),
                action_button(
                    if draft.enabled { "停用" } else { "启用" },
                    AppMessage::AiSettingsToggle(kind.to_string()),
                ),
            )),
        ))
        .into_any()
}

/// 保存/测试的反馈与测试结果。
fn settings_feedback(app: Store<AppState>) -> AnyView {
    let message = dynamic(app.ai_settings().message(), |message| {
        message.as_ref().map(|message| text(message.clone()))
    });
    let error = dynamic(app.ai_settings().error(), |error| {
        error.as_ref().map(|error| text(error.clone()))
    });
    let tests = dynamic(app.ai_settings().test_results(), |results| {
        let rows = results
            .iter()
            .map(|(kind, outcome)| {
                if outcome.running {
                    text(format!("{kind}：测试中…"))
                } else if outcome.ok {
                    text(format!("{kind}：连接正常"))
                } else {
                    text(format!("{kind}：{}", outcome.message))
                }
            })
            .collect::<Vec<_>>();
        if rows.is_empty() {
            None
        } else {
            Some(column().children(rows))
        }
    });
    column().children((message, error, tests)).into_any()
}

/// 关于：技术栈与数据目录说明。
fn about_section(app: Store<AppState>) -> impl IntoView {
    let body = dynamic(app.diagnostics().app_version(), |version| {
        text(format!(
            "Ameya（LiliaStudio）· 本地优先的世界观创作工作台。版本 {}。Rust + NanaUI 原生界面；数据保存在本机。",
            version.as_deref().unwrap_or("未知")
        ))
    });
    widget(Card::new().title("关于")).children(body)
}
