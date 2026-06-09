use crate::services::simulation::{simulate_scenario, SimulationReport};

#[tauri::command]
pub fn simulation_run(
    project_id: String,
    scenario: String,
    referenced_entities: Vec<String>,
) -> SimulationReport {
    simulate_scenario(&project_id, &scenario, referenced_entities)
}
