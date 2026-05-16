use serde::{Deserialize, Serialize};

use super::conflict::LogicConflict;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepairSuggestion {
    pub title: String,
    pub description: String,
    pub impact: String,
}

pub fn suggest_repairs(conflict: &LogicConflict) -> Vec<RepairSuggestion> {
    vec![
        RepairSuggestion {
            title: "修改其中一条事实".into(),
            description: format!("检查 {:?} 中哪条事实应该更新。", conflict.fact_ids),
            impact: "局部影响，适合新设定录入错误。".into(),
        },
        RepairSuggestion {
            title: "添加例外范围".into(),
            description: "为特殊材料、时间段或地点增加例外公理。".into(),
            impact: "中等影响，需要后续审计例外是否扩散。".into(),
        },
        RepairSuggestion {
            title: "调整全局公理".into(),
            description: "如果冲突来自底层规则过宽，收窄全局规则的适用边界。".into(),
            impact: "高影响，会改变依赖该公理的多个设定。".into(),
        },
    ]
}
