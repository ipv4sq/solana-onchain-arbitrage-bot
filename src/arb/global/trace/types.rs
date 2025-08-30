#![allow(dead_code)]

use crate::arb::global::enums::step_type::StepType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct WithTrace<T>(pub T, pub Trace);

#[derive(Clone)]
pub struct Trace {
    pub id: String,
    steps: Arc<Mutex<Vec<Step>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub sequence: u32,
    pub step_type: StepType,
    pub attributes: HashMap<String, String>,
    pub happened_at: DateTime<Utc>,
}

impl Trace {
    pub fn new() -> Self {
        let sequence = TRACE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: format!("trace_{}", sequence),
            steps: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn step(&self, step_type: StepType) {
        let mut steps = self.steps.lock().unwrap();
        let sequence = steps.len() as u32;
        steps.push(Step {
            sequence,
            step_type,
            attributes: HashMap::new(),
            happened_at: Utc::now(),
        });
    }
    pub fn step_with_address(
        &self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: Pubkey,
    ) {
        let mut attributes = HashMap::new();
        attributes.insert(attr_name.into(), attr_value.to_string());

        let mut steps = self.steps.lock().unwrap();
        let sequence = steps.len() as u32;
        steps.push(Step {
            sequence,
            step_type,
            attributes,
            happened_at: Utc::now(),
        });
    }

    pub fn step_with(
        &self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: impl Into<String>,
    ) {
        let mut attributes = HashMap::new();
        attributes.insert(attr_name.into(), attr_value.into());

        let mut steps = self.steps.lock().unwrap();
        let sequence = steps.len() as u32;
        steps.push(Step {
            sequence,
            step_type,
            attributes,
            happened_at: Utc::now(),
        });
    }

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
            "trace_id": self.id,
            "total_duration_ms": (steps.last().unwrap().happened_at - first_timestamp).num_milliseconds(),
            "steps": steps_json,
        })
    }

    pub fn dump_pretty(&self) -> String {
        serde_json::to_string_pretty(&self.dump_json()).unwrap()
    }
}
