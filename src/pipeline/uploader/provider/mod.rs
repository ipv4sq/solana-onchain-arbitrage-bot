use crate::pipeline::uploader::provider::jito::build_jito_tip_ix;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;

pub mod helius;
pub mod jito;
mod jito_new;
pub mod sender;

pub enum SenderChannel {
    HeliusSwqos,
    Jito,
    HeliusJito,
    Shyft,
}

impl SenderChannel {
    pub fn tip_ix(&self, payer: &Pubkey) -> Vec<Instruction> {
        match self {
            SenderChannel::HeliusSwqos => {
                todo!()
            }
            SenderChannel::Jito => build_jito_tip_ix(payer),
            SenderChannel::HeliusJito => {
                todo!()
            }
            SenderChannel::Shyft => {
                todo!()
            }
        }
    }
}
