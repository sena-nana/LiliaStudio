use crate::logic::{
    conflict::{detect_conflicts, Fact, LogicConflict},
    repair::{suggest_repairs, RepairSuggestion},
};

#[tauri::command]
pub fn audit_facts(facts: Vec<Fact>) -> Vec<LogicConflict> {
    detect_conflicts(&facts)
}

#[tauri::command]
pub fn repair_suggestions(conflict: LogicConflict) -> Vec<RepairSuggestion> {
    suggest_repairs(&conflict)
}
