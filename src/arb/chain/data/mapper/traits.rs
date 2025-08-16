use anyhow::Result;
use crate::arb::chain::data::UnifiedTransaction;
use crate::arb::chain::data::instruction::Instruction;
use crate::arb::chain::types::SwapInstruction;

pub trait ToUnified {
    fn to_unified(&self) -> Result<UnifiedTransaction>;
}

pub trait InstructionExtractor {
    fn extract_mev_instructions(&self) -> Vec<Instruction>;
    fn extract_swap_instructions(&self) -> Vec<SwapInstruction>;
}