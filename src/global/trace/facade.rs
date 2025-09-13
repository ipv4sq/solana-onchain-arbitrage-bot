#![allow(dead_code)]

use crate::global::enums::step_type::StepType;
use crate::global::trace::types::{Step, Trace};
use chrono::Utc;
use serde::Serialize;
use serde_json::json;
use solana_program::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(0);

impl Trace {
    pub fn new(slot: u64) -> Self {
        let sequence = TRACE_COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: format!("trace_{}", sequence),
            slot,
            steps: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn since_last_step(&self) -> u32 {
        let steps = self.steps.lock().unwrap();
        if let Some(last_step) = steps.last() {
            (Utc::now() - last_step.happened_at).num_milliseconds() as u32
        } else {
            0
        }
    }

    pub fn since_begin(&self) -> u32 {
        let steps = self.steps.lock().unwrap();
        if let Some(first_step) = steps.first() {
            (Utc::now() - first_step.happened_at).num_milliseconds() as u32
        } else {
            0
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

    pub fn step_with_custom(&self, step_type: &str) {
        self.step(StepType::Custom(step_type.to_string()));
    }

    pub fn step_with_address(
        &self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: Pubkey,
    ) {
        let mut attributes = HashMap::new();
        attributes.insert(attr_name.into(), json!(attr_value.to_string()));

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
        attributes.insert(attr_name.into(), json!(attr_value.into()));

        let mut steps = self.steps.lock().unwrap();
        let sequence = steps.len() as u32;
        steps.push(Step {
            sequence,
            step_type,
            attributes,
            happened_at: Utc::now(),
        });
    }

    pub fn step_with_json(
        &self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: serde_json::Value,
    ) {
        let mut attributes = HashMap::new();
        let converted_value = crate::util::serde_pubkey::to_json_value(&attr_value);
        attributes.insert(attr_name.into(), converted_value);

        let mut steps = self.steps.lock().unwrap();
        let sequence = steps.len() as u32;
        steps.push(Step {
            sequence,
            step_type,
            attributes,
            happened_at: Utc::now(),
        });
    }

    pub fn step_with_struct<T: Serialize>(
        &self,
        step_type: StepType,
        attr_name: impl Into<String>,
        attr_value: &T,
    ) {
        let mut attributes = HashMap::new();
        attributes.insert(
            attr_name.into(),
            crate::util::serde_pubkey::to_json_value(attr_value),
        );

        let mut steps = self.steps.lock().unwrap();
        let sequence = steps.len() as u32;
        steps.push(Step {
            sequence,
            step_type,
            attributes,
            happened_at: Utc::now(),
        });
    }
}
