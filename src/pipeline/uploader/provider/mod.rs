use solana_program::instruction::Instruction;

pub mod helius;
pub mod jito;
pub mod sender;

pub enum SenderChannel {
    HeliusSwqos,
    Jito,
    HeliusJito,
    Shyft,
}

impl SenderChannel {
    pub fn tip_ix(&self) -> Vec<Instruction> {
        match self {
            SenderChannel::HeliusSwqos => {}
            SenderChannel::Jito => {}
            SenderChannel::HeliusJito => {}
            SenderChannel::Shyft => {}
        }
        todo!()
    }
}
