use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SimulationReport {
    pub project_id: String,
    pub scenario: String,
    pub phases: Vec<SimulationPhase>,
    pub risks: Vec<String>,
    pub referenced_entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SimulationPhase {
    pub label: String,
    pub summary: String,
}

pub fn simulate_scenario(
    project_id: &str,
    scenario: &str,
    referenced_entities: Vec<String>,
) -> SimulationReport {
    SimulationReport {
        project_id: project_id.into(),
        scenario: scenario.into(),
        phases: vec![
            SimulationPhase {
                label: "短期".into(),
                summary: format!("{scenario} 会首先改变资源分配和角色优先级。"),
            },
            SimulationPhase {
                label: "中期".into(),
                summary: "阵营会围绕资源、责任和合法性重新站队。".into(),
            },
        ],
        risks: vec!["需要检查资源来源是否足够支撑推演。".into()],
        referenced_entities,
    }
}
