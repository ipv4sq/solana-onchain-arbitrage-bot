use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    XtoY,
    YtoX,
}

pub struct TradeDirection {
    pub from: Pubkey,
    pub to: Pubkey,
}
