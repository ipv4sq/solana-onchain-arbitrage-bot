use anyhow::Result;
use solana_client::rpc_response::RpcSimulateTransactionResult;

use crate::arb::chain::meta::TransactionMeta;
use crate::arb::chain::instruction::InnerInstructions;

pub struct SimulationResult {
    pub meta: Option<TransactionMeta>,
    pub err: Option<String>,
    pub units_consumed: Option<u64>,
    pub logs: Vec<String>,
}

impl From<&RpcSimulateTransactionResult> for SimulationResult {
    fn from(result: &RpcSimulateTransactionResult) -> Self {
        let err = result.err.as_ref().map(|e| format!("{:?}", e));
        
        let units_consumed = result.units_consumed;
        
        let logs = result.logs.as_ref().cloned().unwrap_or_default();
        
        let meta = if err.is_none() {
            Some(TransactionMeta {
                fee: 0,
                compute_units_consumed: units_consumed,
                log_messages: logs.clone(),
                inner_instructions: extract_inner_instructions(result),
                pre_balances: Vec::new(),
                post_balances: Vec::new(),
                pre_token_balances: Vec::new(),
                post_token_balances: Vec::new(),
                err: None,
                loaded_writable_addresses: Vec::new(),
                loaded_readonly_addresses: Vec::new(),
            })
        } else {
            None
        };
        
        SimulationResult {
            meta,
            err,
            units_consumed,
            logs,
        }
    }
}

fn extract_inner_instructions(result: &RpcSimulateTransactionResult) -> Vec<InnerInstructions> {
    result.inner_instructions
        .as_ref()
        .map(|instructions| {
            instructions
                .iter()
                .map(|inner_ix| InnerInstructions {
                    parent_index: inner_ix.index,
                    instructions: Vec::new(),
                })
                .collect()
        })
        .unwrap_or_default()
}

impl SimulationResult {
    pub fn is_success(&self) -> bool {
        self.err.is_none()
    }
    
    pub fn compute_units_used(&self) -> u64 {
        self.units_consumed.unwrap_or(0)
    }
    
    pub fn error_message(&self) -> Option<&str> {
        self.err.as_deref()
    }
    
    pub fn has_logs(&self) -> bool {
        !self.logs.is_empty()
    }
    
    pub fn find_log_containing(&self, pattern: &str) -> Option<&str> {
        self.logs.iter()
            .find(|log| log.contains(pattern))
            .map(|s| s.as_str())
    }
    
    pub fn format_logs(&self) -> String {
        self.logs.join("\n")
    }
    
    pub fn program_logs(&self, program_id: &str) -> Vec<&str> {
        let mut collecting = false;
        let mut logs = Vec::new();
        let program_invoke = format!("Program {} invoke", program_id);
        let program_success = format!("Program {} success", program_id);
        let program_failed = format!("Program {} failed", program_id);
        
        for log in &self.logs {
            if log.contains(&program_invoke) {
                collecting = true;
            } else if log.contains(&program_success) || log.contains(&program_failed) {
                collecting = false;
            } else if collecting {
                logs.push(log.as_str());
            }
        }
        
        logs
    }
}