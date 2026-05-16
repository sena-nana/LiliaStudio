use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Fact {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub scope_time: String,
    pub scope_location: String,
}

impl Fact {
    pub fn axiom(
        id: &str,
        subject: &str,
        predicate: &str,
        object: &str,
        scope_time: &str,
        scope_location: &str,
    ) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            scope_time: scope_time.into(),
            scope_location: scope_location.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LogicConflict {
    pub conflict_type: String,
    pub fact_ids: Vec<String>,
    pub message: String,
}

pub fn detect_conflicts(facts: &[Fact]) -> Vec<LogicConflict> {
    let mut conflicts = Vec::new();
    for (index, left) in facts.iter().enumerate() {
        for right in facts.iter().skip(index + 1) {
            if left.subject == right.subject
                && left.predicate == right.predicate
                && left.scope_time == right.scope_time
                && left.scope_location == right.scope_location
                && left.object != right.object
            {
                conflicts.push(LogicConflict {
                    conflict_type: "mutually_exclusive_axioms".into(),
                    fact_ids: vec![left.id.clone(), right.id.clone()],
                    message: format!(
                        "{} 在相同范围内同时被定义为 {} 和 {}",
                        left.subject, left.object, right.object
                    ),
                });
            }
        }
    }
    conflicts
}
