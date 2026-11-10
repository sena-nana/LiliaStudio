use super::conflict::{Fact, LogicConflict};

pub fn minimal_conflict(
    facts: &[Fact],
    checker: fn(&[Fact]) -> Vec<LogicConflict>,
) -> Option<Vec<Fact>> {
    if checker(facts).is_empty() {
        return None;
    }

    let mut minimal = facts.to_vec();
    let mut changed = true;
    while changed {
        changed = false;
        for index in 0..minimal.len() {
            let mut candidate = minimal.clone();
            candidate.remove(index);
            if !checker(&candidate).is_empty() {
                minimal = candidate;
                changed = true;
                break;
            }
        }
    }
    Some(minimal)
}
