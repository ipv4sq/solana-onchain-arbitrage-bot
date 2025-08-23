use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

pub struct MevBot;
impl MevBot {
    pub const EMV_BOT_PROGRAM: Pubkey = pubkey!("MEViEnscUm6tsQRoGd9h6nLQaQspKj7DB2M5FwM3Xvz");
    // Flashloan
    pub const FLASHLOAN_ACCOUNT: Pubkey = pubkey!("5LFpzqgsxrSfhKwbaFiAEJ2kbc9QyimjKueswsyU4T3o");
    // Fees
    pub const FLASHLOAN_FEE_ACCOUNT: Pubkey =
        pubkey!("6AGB9kqgSp2mQXwYpdrV4QVV8urvCaDS35U1wsLssy6H");
    pub const NON_FLASHLOAN_ACCOUNT_1: Pubkey =
        pubkey!("GPpkDpzCDmYJY5qNhYmM14c7rct1zmkjWc2CjR5g7RZ1");
    pub const NON_FLASHLOAN_ACCOUNT_2: Pubkey =
        pubkey!("J6c7noBHvWju4mMA3wXt3igbBSp2m9ATbA6cjMtAUged");
    pub const NON_FLASHLOAN_ACCOUNT_3: Pubkey =
        pubkey!("BjsfwxDu7GX7RRW6oSRTpMkASdXAgCcHnXEcatqSfuuY");
}
