#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub id: String,
    steps: Vec<Step>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub sequence: u32,
    pub step_type: StepType,
    pub attributes: HashMap<String, String>,
    pub happened_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    AccountUpdateReceived,
    AccountUpdateDebouncing,
    AccountUpdateDebounced,
    TradeStrategyStarted,
    Custom(String),
}

#[derive(Clone)]
pub struct WithTrace<T: Sized> {
    pub param: T,
    pub trace: Arc<Mutex<Trace>>,
}

impl<T: Sized> WithTrace<T> {
    pub fn new(trace: Trace, param: T) -> WithTrace<T> {
        WithTrace {
            param,
            trace: Arc::new(Mutex::new(trace)),
        }
    }

    pub fn new_shared(trace: Arc<Mutex<Trace>>, param: T) -> WithTrace<T> {
        WithTrace { param, trace }
    }

    pub fn with_param<U: Sized>(&self, new_param: U) -> WithTrace<U> {
        WithTrace {
            param: new_param,
            trace: self.trace.clone(),
        }
    }

    pub fn step(&self, step_type: StepType) {
        self.trace.lock().unwrap().step(step_type);
    }

    pub fn step_with_address(
        &self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: Pubkey,
    ) {
        self.trace
            .lock()
            .unwrap()
            .step_with_address(step_type, attr_name, attr_value);
    }

    pub fn step_with(
        &self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: impl Into<String>,
    ) {
        self.trace
            .lock()
            .unwrap()
            .step_with(step_type, attr_name, attr_value);
    }

    pub fn step_with_attrs(&self, step_type: StepType, attributes: HashMap<String, String>) {
        self.trace
            .lock()
            .unwrap()
            .step_with_attrs(step_type, attributes);
    }
}

impl Trace {
    pub fn new() -> Self {
        let sequence = TRACE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: format!("trace_{}", sequence),
            steps: Vec::new(),
        }
    }

    pub fn step(&mut self, step_type: StepType) {
        self.steps.push(Step {
            sequence: self.steps.len() as u32,
            step_type,
            attributes: HashMap::new(),
            happened_at: Utc::now(),
        });
    }
    pub fn step_with_address(
        &mut self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: Pubkey,
    ) {
        let mut attributes = HashMap::new();
        attributes.insert(attr_name.into(), attr_value.to_string());

        self.steps.push(Step {
            sequence: self.steps.len() as u32,
            step_type,
            attributes,
            happened_at: Utc::now(),
        });
    }
    pub fn step_with(
        &mut self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: impl Into<String>,
    ) {
        let mut attributes = HashMap::new();
        attributes.insert(attr_name.into(), attr_value.into());

        self.steps.push(Step {
            sequence: self.steps.len() as u32,
            step_type,
            attributes,
            happened_at: Utc::now(),
        });
    }

    pub fn step_with_attrs(&mut self, step_type: StepType, attributes: HashMap<String, String>) {
        self.steps.push(Step {
            sequence: self.steps.len() as u32,
            step_type,
            attributes,
            happened_at: Utc::now(),
        });
    }

    pub fn dump(&self) {
        if self.steps.is_empty() {
            println!("{{\"trace_id\": \"{}\", \"steps\": []}}", self.id);
            return;
        }

        let first_timestamp = self.steps.first().unwrap().happened_at;

        let steps_json: Vec<_> = self
            .steps
            .iter()
            .map(|step| {
                let relative_ms = (step.happened_at - first_timestamp).num_milliseconds();

                json!({
                    "sequence": step.sequence,
                    "type": match &step.step_type {
                        StepType::AccountUpdateReceived => "AccountUpdateReceived",
                        StepType::AccountUpdateDebouncing => "AccountUpdateDebouncing",
                        StepType::AccountUpdateDebounced => "AccountUpdateDebounced",
                        StepType::TradeStrategyStarted => "TradeStrategyStarted",
                        StepType::Custom(s) => s.as_str(),
                    },
                    "absolute_time": step.happened_at.to_rfc3339(),
                    "relative_ms": relative_ms,
                    "attributes": step.attributes,
                })
            })
            .collect();

        let output = json!({
            "trace_id": self.id,
            "total_duration_ms": (self.steps.last().unwrap().happened_at - first_timestamp).num_milliseconds(),
            "steps": steps_json,
        });

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }
}
