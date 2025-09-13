use crate::global::enums::step_type::StepType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct WithTrace<T>(pub T, pub Trace);

#[derive(Clone)]
pub struct Trace {
    pub id: String,
    pub slot: u64,
    pub(crate) steps: Arc<Mutex<Vec<Step>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub sequence: u32,
    pub step_type: StepType,
    pub attributes: HashMap<String, serde_json::Value>,
    pub happened_at: DateTime<Utc>,
}

impl Trace {
    pub fn dump_json(&self) -> serde_json::Value {
        let steps = self.steps.lock().unwrap();
        if steps.is_empty() {
            return json!({
                "trace_id": self.id,
                "steps": []
            });
        }

        let first_timestamp = steps.first().unwrap().happened_at;

        let steps_json: Vec<_> = steps
            .iter()
            .map(|step| {
                let relative_ms = (step.happened_at - first_timestamp).num_milliseconds();

                json!({
                    "sequence": step.sequence,
                    "type": match &step.step_type {
                        StepType::Custom(s) => s.as_str(),
                        other => other.as_ref(),
                    },
                    "absolute_time": step.happened_at.to_rfc3339(),
                    "relative_ms": relative_ms,
                    "attributes": step.attributes,
                })
            })
            .collect();

        json!({
            "slot": self.slot,
            "trace_id": self.id,
            "total_duration_ms": (steps.last().unwrap().happened_at - first_timestamp).num_milliseconds(),
            "steps": steps_json,
        })
    }

    pub fn dump_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.dump_json()).unwrap()
    }
}
